use bytes::Bytes;
use http::HeaderMap;
use reqwest::Client;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

use crate::e2e::environment::AgentTestEnvironment;
use crate::e2e::git_proxy::compression;
use crate::e2e::git_proxy::packet_line::advertisement::create_git_advertisement;
use crate::e2e::git_proxy::packet_line::errors::create_git_error_message;
use crate::e2e::git_proxy::packet_line::parse_commands::{RefModification, parse_update_requests};
use crate::e2e::git_proxy::{Forward, ForwardToLocal, ForwardToRemote, ProxyBehaivor};
use crate::e2e::http_server::{HttpRequest, HttpResponse};
use crate::e2e::{ServerContext, http_server};

/// GET /info/refs?service=<service>
///
/// This endpoint is used by Git clients to discover available refs. In protocol v2,
/// the handshake is initiated here. We forward both push (git-receive-pack)
/// and fetch (git-upload-pack) info requests.
pub async fn info_refs_handler(
    req: HttpRequest,
    context: Arc<Mutex<ServerContext>>,
) -> HttpResponse {
    let query = req.endpoint.query().unwrap_or("");
    if !query.contains("service=git-receive-pack") && !query.contains("service=git-upload-pack") {
        return HttpResponse::bad_request().text("Unsupported or missing service");
    }

    let behaivor = proxy_behaivor(&req, context).await;
    match behaivor.forward {
        Forward::ForwardToRemote(ref forward) => remote_info_refs(forward, query).await,
        Forward::ForwardToLocal(ref local) => local_info_refs(local, query).await,
    }
}

async fn remote_info_refs(forward: &ForwardToRemote, query: &str) -> HttpResponse {
    let mut forward_url = forward.url.clone();
    {
        let mut segments = forward_url
            .path_segments_mut()
            .expect("Cannot modify URL segments");
        segments.push("info");
        segments.push("refs");
    }
    forward_url.set_query(Some(query));

    let client = Client::new();
    match client
        .get(forward_url)
        .basic_auth(
            forward.basic_auth_user.clone(),
            Some(forward.basic_auth_pass.clone()),
        )
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let body = resp.bytes().await.unwrap_or_else(|_| Bytes::new());
            HttpResponse::from_status(status)
                .content_type(&content_type)
                .body(body)
        }
        Err(err) => {
            error!("Error forwarding info/refs: {:?}", err);
            HttpResponse::internal_server_error().text("Error forwarding request")
        }
    }
}

async fn local_info_refs(forward: &ForwardToLocal, query: &str) -> HttpResponse {
    let (service, content_type) = if query.contains("service=git-receive-pack") {
        (
            "git-receive-pack",
            "application/x-git-receive-pack-advertisement",
        )
    } else if query.contains("service=git-upload-pack") {
        (
            "git-upload-pack",
            "application/x-git-upload-pack-advertisement",
        )
    } else {
        return HttpResponse::bad_request().text("Unsupported or missing service");
    };

    let output = match Command::new(service)
        .arg("--advertise-refs")
        .arg(&forward.path)
        .output()
        .await
    {
        Ok(o) => o,
        Err(err) => {
            error!("Error spawning {}: {:?}", service, err);
            return HttpResponse::internal_server_error()
                .text(&format!("Error spawning {service}"));
        }
    };

    if !output.status.success() {
        error!("Command {} exited with non-zero status", service);
        return HttpResponse::internal_server_error().text("Error processing info/refs");
    }

    // Prepend the Git advertisement header.
    let advertisement = create_git_advertisement(service, &output.stdout);

    HttpResponse::ok()
        .content_type(content_type)
        .body(advertisement)
}

/// POST /git-receive-pack
///
/// This endpoint is used by Git clients to push updates.
/// We first inspect the push commands to ensure that they only affect the allowed ref,
/// and if so we forward the entire request.
pub async fn git_receive_pack_handler(
    req: HttpRequest,
    context: Arc<Mutex<ServerContext>>,
) -> HttpResponse {
    let body_bytes = decompress_if_gzip(req.body.clone(), &req.headers).await;
    let body_bytes = match body_bytes {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Failed to decompress request body: {}", err);
            return HttpResponse::bad_request().text("Could not decompress request body");
        }
    };

    let behaivor = proxy_behaivor(&req, context.clone()).await;

    match parse_update_requests(&body_bytes) {
        Ok(refs) => {
            for r in refs {
                if r.ref_name() != behaivor.allowed_ref {
                    warn!("Push attempted to disallowed ref: {}", r.ref_name());
                    let error_body =
                        create_git_error_message("Push not allowed to modify this ref");
                    return HttpResponse::ok()
                        .content_type("application/x-git-receive-pack-result")
                        .body(error_body);
                }
                match r {
                    RefModification::Create { .. } => {
                        warn!("Push attempted to create ref: {}", r.ref_name());
                        let error_body =
                            create_git_error_message("Push not allowed to create this ref");
                        return HttpResponse::ok()
                            .content_type("application/x-git-receive-pack-result")
                            .body(error_body);
                    }
                    RefModification::Delete { .. } => {
                        warn!("Push attempted to delete ref: {}", r.ref_name());
                        let error_body =
                            create_git_error_message("Push not allowed to delete this ref");
                        return HttpResponse::ok()
                            .content_type("application/x-git-receive-pack-result")
                            .body(error_body);
                    }
                    RefModification::Update { .. } => {}
                }
            }
        }
        Err(e) => {
            error!("Error parsing push commands: {:?}", e);
            let error_body = create_git_error_message("Invalid push data");
            return HttpResponse::ok()
                .content_type("application/x-git-receive-pack-result")
                .body(error_body);
        }
    }

    AgentTestEnvironment::shutdown_on_invalid_action(
        &mut context.lock().await,
        crate::e2e::AgentExpectedAction::GitCmd(crate::e2e::GitCmd::Push),
        crate::e2e::TestFailure::Git(crate::e2e::GitCmd::Push),
    )
    .await;

    match behaivor.forward {
        Forward::ForwardToRemote(ref forward) => remote_git_receive_pack(forward, body_bytes).await,
        Forward::ForwardToLocal(ref local) => local_git_receive_pack(local, body_bytes).await,
    }
}

async fn remote_git_receive_pack(forward: &ForwardToRemote, body_bytes: Bytes) -> HttpResponse {
    let mut forward_url = forward.url.clone();
    {
        let mut segments = forward_url
            .path_segments_mut()
            .expect("Cannot modify URL segments");
        segments.push("git-receive-pack");
    }
    let client = Client::new();
    match client
        .post(forward_url)
        .basic_auth(
            forward.basic_auth_user.clone(),
            Some(forward.basic_auth_pass.clone()),
        )
        .header("Content-Type", "application/x-git-receive-pack-request")
        .body(body_bytes.clone())
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let resp_body = resp.bytes().await.unwrap_or_else(|_| Bytes::new());
            HttpResponse::from_status(status)
                .content_type(&content_type)
                .body(resp_body)
        }
        Err(err) => {
            error!("Error forwarding git-receive-pack: {:?}", err);
            let error_body = create_git_error_message("Error forwarding push");
            HttpResponse::ok()
                .content_type("application/x-git-receive-pack-result")
                .body(error_body)
        }
    }
}

async fn local_git_receive_pack(forward: &ForwardToLocal, body_bytes: Bytes) -> HttpResponse {
    debug!("local git receive pack on path {:?}", forward.path);

    let mut child = match Command::new("git-receive-pack")
        .arg("--stateless-rpc")
        .arg(&forward.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            error!("Error spawning git-receive-pack: {:?}", err);
            let error_body = create_git_error_message("Error spawning git-receive-pack");
            return HttpResponse::ok()
                .content_type("application/x-git-receive-pack-result")
                .body(error_body);
        }
    };

    // Write the request body to the child’s stdin.
    if let Some(mut stdin) = child.stdin.take()
        && let Err(err) = stdin.write_all(&body_bytes).await
    {
        error!("Error writing to git-receive-pack stdin: {:?}", err);
    }

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(err) => {
            error!("Error waiting for git-receive-pack: {:?}", err);
            let error_body = create_git_error_message("Error processing push");
            return HttpResponse::ok()
                .content_type("application/x-git-receive-pack-result")
                .body(error_body);
        }
    };

    if !output.status.success() {
        error!("git-receive-pack exited with non-zero status");
        let error_body = create_git_error_message("git-receive-pack failed");
        return HttpResponse::ok()
            .content_type("application/x-git-receive-pack-result")
            .body(error_body);
    }

    HttpResponse::ok()
        .content_type("application/x-git-receive-pack-result")
        .body(Bytes::from(output.stdout))
}

/// POST /git-upload-pack
///
/// This endpoint is used by Git clients to fetch objects (clone or fetch).
/// Unlike push, no ref restrictions are needed, so we simply forward the request.
pub async fn git_upload_pack_handler(
    req: HttpRequest,
    context: Arc<Mutex<ServerContext>>,
) -> HttpResponse {
    let body_bytes = decompress_if_gzip(req.body.clone(), &req.headers).await;
    let body_bytes = match body_bytes {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Failed to decompress request body: {}", err);
            return HttpResponse::bad_request().text("Could not decompress request body");
        }
    };

    let behaivor = proxy_behaivor(&req, context.clone()).await;

    let mut context = context.lock().await;
    context.test_data.failure = context.environment.invalid_action_to_failure(
        &crate::e2e::AgentExpectedAction::GitCmd(crate::e2e::GitCmd::Push),
        crate::e2e::TestFailure::Git(crate::e2e::GitCmd::Push),
    );
    if context.test_data.failure.is_some() {
        error!(
            "Test failure: {}",
            context.test_data.failure.clone().unwrap()
        );
        context.terminate_server()
    }

    match behaivor.forward {
        Forward::ForwardToRemote(ref forward) => remote_git_upload_pack(forward, body_bytes).await,
        Forward::ForwardToLocal(ref local) => local_git_upload_pack(local, body_bytes).await,
    }
}

async fn remote_git_upload_pack(forward: &ForwardToRemote, body_bytes: Bytes) -> HttpResponse {
    let mut forward_url = forward.url.clone();
    {
        let mut segments = forward_url
            .path_segments_mut()
            .expect("Cannot modify URL segments");
        segments.push("git-upload-pack");
    }
    let client = Client::new();
    match client
        .post(forward_url)
        .basic_auth(
            forward.basic_auth_user.clone(),
            Some(forward.basic_auth_pass.clone()),
        )
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(body_bytes.clone())
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let resp_body = resp.bytes().await.unwrap_or_else(|_| Bytes::new());
            HttpResponse::from_status(status)
                .content_type(&content_type)
                .body(resp_body)
        }
        Err(err) => {
            error!("Error forwarding git-upload-pack: {:?}", err);
            HttpResponse::internal_server_error().text("Error forwarding fetch")
        }
    }
}

async fn local_git_upload_pack(forward: &ForwardToLocal, body_bytes: Bytes) -> HttpResponse {
    debug!("local git upload pack on path {:?}", forward.path);

    let mut child = match Command::new("git-upload-pack")
        .arg("--stateless-rpc")
        .arg(&forward.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            error!("Error spawning git-upload-pack: {:?}", err);
            return HttpResponse::internal_server_error().text("Error spawning git-upload-pack");
        }
    };

    if let Some(mut stdin) = child.stdin.take()
        && let Err(err) = stdin.write_all(&body_bytes).await
    {
        error!("Error writing to git-upload-pack stdin: {:?}", err);
    }

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(err) => {
            error!("Error waiting for git-upload-pack: {:?}", err);
            return HttpResponse::internal_server_error().text("Error processing fetch");
        }
    };

    if !output.status.success() {
        error!("git-upload-pack exited with non-zero status");
        return HttpResponse::internal_server_error().text("git-upload-pack failed");
    }

    HttpResponse::ok()
        .content_type("application/x-git-upload-pack-result")
        .body(Bytes::from(output.stdout))
}

/// Extract a `ProxyBehaivor` from the server context.
async fn proxy_behaivor(_: &HttpRequest, context: Arc<Mutex<ServerContext>>) -> ProxyBehaivor {
    let context = context.lock().await;
    ProxyBehaivor {
        allowed_ref: format!("refs/heads/{}", context.environment.git_branch),
        forward: Forward::ForwardToLocal(ForwardToLocal {
            path: context.environment.git_repo_path.clone(),
        }),
    }
}

/// Decompress the request body if it is encoded with gzip.
async fn decompress_if_gzip(
    body_bytes: Bytes,
    headers: &HeaderMap,
) -> Result<Bytes, http_server::Error> {
    if let Some(encoding) = headers.get("Content-Encoding") {
        let encoding_str = encoding.to_str().unwrap_or("").trim();
        if encoding_str.eq_ignore_ascii_case("gzip") {
            let decompressed = compression::decompress_gzip(&body_bytes)
                .await
                .map_err(|e| {
                    error!("Error decompressing body: {:?}", e);
                    http_server::Error::BadRequest("Decompression failed".to_owned())
                })?;
            return Ok(Bytes::from(decompressed));
        } else {
            error!("Unsupported Content-Encoding: {:?}", encoding);
        }
    }
    Ok(body_bytes)
}
