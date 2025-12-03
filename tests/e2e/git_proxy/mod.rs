//! A simple Git proxy integration that forwards Git requests to a Git server.
//! It supports the Git v2 wire protocol via the smart HTTP transfer protocol.
//! In other words, most modern Git clients should work with this proxy over HTTP.
//! For authentication, currently only HTTP Basic Authentication is supported, both for the proxy itself and for the upstream Git server.
//!
//! # Usage Example
//!
//! For a basic usage example, see the `basic` example in the `examples` directory.
//!
//! # How it Works
//!
//! 1. Client requests (e.g. `git clone`, `git push`, `git fetch`) are sent to
//!    your server at the path defined in [`scope`].
//! 2. An optional Basic Authentication check (the validator you provide) runs,
//!    ensuring the request is authorized to access the proxy.
//!    This check needs to supply a [`ProxyBehaivor`] instance to the request extensions
//!    which will tell the proxy how to forward the Git requests.
//! 3. The proxy inspects the request body of push requests to apply any configured restrictions.
//!    Currently, push requests are restricted to a single specific ref (e.g. branch) configured by `allowed_ref`.
//!    deletion and creation of refs is forbidden.
//! 4. The proxy routes the request to the corresponding Git endpoints (`info/refs`,
//!    `git-receive-pack`, `git-upload-pack`) and relays the response from the upstream server back to the client.
//!
//! # References
//!
//! For more details on the Git HTTP protocol and the wire protocol v2, see:
//!
//! - [Git HTTP protocol documentation](https://git-scm.com/docs/http-protocol)
//! - [Git wire protocol v2 documentation](https://git-scm.com/docs/protocol-v2)

use std::path::PathBuf;
use url::Url;

mod compression;
mod packet_line;
pub(crate) mod routes;

/// What the proxy should do with the request.
///
/// # Usage
///
/// In your authentication validator function, supply an instance of this struct to the request extensions.
/// For usage examples, see the `examples` directory.
#[derive(Clone)]
pub struct ProxyBehaivor {
    /// A reference (e.g., "refs/heads/main") indicating which ref/branch is allowed to be updated during a push operation.
    /// Pushes to other refs will be denied.
    pub allowed_ref: String,
    /// How to forward the request.
    pub forward: Forward,
}

/// How to forward the request.
#[derive(Clone)]
pub enum Forward {
    /// Forward the request to another Git server.
    ForwardToRemote(ForwardToRemote),
    /// Forward the request to a local Git repository.
    ForwardToLocal(ForwardToLocal),
}

impl From<ForwardToRemote> for Forward {
    fn from(f: ForwardToRemote) -> Self {
        Forward::ForwardToRemote(f)
    }
}

impl From<ForwardToLocal> for Forward {
    fn from(f: ForwardToLocal) -> Self {
        Forward::ForwardToLocal(f)
    }
}

/// Forward Git requests to another server.
#[derive(Clone)]
pub struct ForwardToRemote {
    /// The upstream Git server's URL to which Git commands are forwarded.
    pub url: Url,
    /// The username used for Basic Authentication with the upstream server.
    pub basic_auth_user: String,
    /// The password used for Basic Authentication with the upstream server.
    pub basic_auth_pass: String,
}

/// Forward Git requests to a local Git repository.
#[derive(Clone)]
pub struct ForwardToLocal {
    pub path: PathBuf,
}
