use std::collections::HashMap;

use bollard::Docker;
use bollard::container::LogOutput;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptions;
use bollard::query_parameters::{
    AttachContainerOptionsBuilder, DownloadFromContainerOptionsBuilder, StartContainerOptions,
};
use futures_util::stream::StreamExt;
use std::io::Read;
use tar::Archive;
use tracing::{debug, error, info, warn};

use super::Result;

const DOCKER_BASE_URL: &str = "http://host.docker.internal:3000/api/";
const MINION_API_TOKEN: &str = "42";

pub(super) fn get_docker() -> Result<Docker> {
    Ok(Docker::connect_with_local_defaults()?)
}

/// starts the agent container from the given image
/// returns the id of the started container
pub(super) async fn start_agent_container(docker: &Docker, image_name: &str) -> Result<String> {
    let env_vars = vec![
        format!("MINION_API_BASE_URL={DOCKER_BASE_URL}"),
        format!("MINION_API_TOKEN={MINION_API_TOKEN}"),
    ];
    let host_config = HostConfig {
        extra_hosts: Some(vec!["host.docker.internal:host-gateway".to_owned()]),
        ..Default::default()
    };

    let container_config = ContainerCreateBody {
        image: Some(image_name.to_owned()),
        host_config: Some(host_config),
        env: Some(env_vars),
        ..Default::default()
    };

    let container = docker
        .create_container(None::<CreateContainerOptions>, container_config)
        .await?;

    let container_id = container.id;

    docker
        .start_container(&container_id, None::<StartContainerOptions>)
        .await?;

    info!("Started agent container with id {container_id}");

    Ok(container_id)
}

pub(super) async fn attach_outstreams(
    docker: &Docker,
    container_id: &str,
) -> Result<tokio::task::JoinHandle<()>> {
    let attach_options = AttachContainerOptionsBuilder::new()
        .stdout(true)
        .stderr(true)
        .stream(true)
        .logs(true)
        .build();

    let mut stream = docker
        .attach_container(container_id, Some(attach_options))
        .await?;

    Ok(tokio::spawn(async move {
        while let Some(log) = stream.output.next().await {
            match log {
                Ok(LogOutput::StdOut { message }) => {
                    let msg = String::from_utf8_lossy(&message).to_string();
                    info!("STDOUT: {}", msg.strip_suffix("\n").unwrap_or(&msg));
                }
                Ok(LogOutput::StdErr { message }) => {
                    let err = String::from_utf8_lossy(&message).to_string();
                    warn!("STDERR: {}", err.strip_suffix("\n").unwrap_or(&err));
                }
                Ok(_) => {}
                Err(e) => error!("Error: {e}"),
            }
        }
    }))
}

/// Extracts all files from the container under `container_id`.
/// Returns a map from the filenames to the file content.
/// Filenames are given with path, so e.g. `dev/my_project/main.rs`
pub(super) async fn get_files(
    docker: &Docker,
    container_id: &str,
) -> Result<HashMap<String, String>> {
    let options = DownloadFromContainerOptionsBuilder::new()
        .path("~/app")
        .build();

    let mut tar_stream = docker.download_from_container(container_id, Some(options));

    let mut archive_data = Vec::new();
    while let Some(Ok(chunk)) = tar_stream.next().await {
        archive_data.extend(chunk);
    }

    let mut files = HashMap::new();

    let mut archive = Archive::new(&archive_data[..]);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_str().unwrap().to_owned();
        let mut file_contents = String::new();
        entry.read_to_string(&mut file_contents)?;

        debug!("found file: {path:?}");
        debug!("file content: {file_contents:?}");

        files.insert(path, file_contents);
    }

    Ok(files)
}
