use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bollard::Docker;

use super::Result;
use super::environment::AgentTestEnvironment;
use super::runner::AgentTestRunner;

pub struct AgentContainerBuilder {
    pub path: PathBuf,
    environment: AgentTestEnvironment,
    docker: Docker,
}

impl AgentContainerBuilder {
    pub(super) fn new<P: AsRef<Path>>(
        path: P,
        environment: AgentTestEnvironment,
        docker: Docker,
    ) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            environment,
            docker,
        })
    }

    pub fn run(self) -> Result<AgentTestRunner> {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let image_name = std::env::var("ORCA_E2E_AGENT_IMAGE").or_else(|_| {
            tokio_runtime.block_on(async { self.build_container_image(&self.path).await })
        })?;

        let runner = AgentTestRunner::new(&image_name, self.environment, self.docker)?;
        Ok(runner)
    }

    /// Build a container image by invoking the `docker` CLI.
    ///
    /// `containerfile` should point to the Containerfile to use.
    /// The build context will be the parent directory of that file.
    pub async fn build_container_image<P: AsRef<Path>>(&self, containerfile: P) -> Result<String> {
        let image_name = "agent-under-test".to_owned();

        // Determine the context dir (parent of the Containerfile).
        let containerfile_path = containerfile.as_ref();
        let context_dir = containerfile_path
            .parent()
            .ok_or("Missing parent directory".to_owned())?;

        // docker build -t agent-under-test -f <containerfile> <context_dir>
        let status = Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(&image_name)
            .arg("-f")
            .arg(containerfile_path)
            .arg(context_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(format!("`docker build` failed with status: {}", status).into());
        }

        Ok(image_name)
    }
}
