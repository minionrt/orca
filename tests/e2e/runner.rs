use std::{fs, net::SocketAddr};

use bollard::Docker;
use futures::StreamExt;
use tracing::{debug, error, info};

use crate::e2e::{AGENT_TEST_GIT_BRANCH, git};

use super::{
    AgentExpectedAction, AgentTestFinishedRun, Result, ServerContext, TestFailure, TestLog, api,
    api::APIEndpoint,
    container::{attach_outstreams, get_files, start_agent_container},
    environment::AgentTestEnvironment,
    git_proxy, http_server,
    http_server::{HttpResponse, Server},
};

pub struct AgentTestRunner {
    environment: AgentTestEnvironment,
    test_data: AgentTestFinishedRun,
    tokio_runtime: tokio::runtime::Runtime,

    server_process: tokio::task::JoinHandle<http_server::Result<ServerContext>>,
    container_logging_process: tokio::task::JoinHandle<()>,
    docker: Docker,
    container_id: String,

    git_folder: tempfile::TempDir,
}

impl AgentTestRunner {
    /// Construct a runner from an `AgentEnvironment`.
    /// This starts the docker container with the webserver for the API hooks
    pub fn new(
        agent_image: &str,
        agent_environment: AgentTestEnvironment,
        docker: Docker,
    ) -> Result<Self> {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

        let git_folder = Self::initialize_local_git_repo()?;
        let mut new_environment = agent_environment;
        new_environment.git_repo_path = git_folder.path().into();

        let server_process =
            tokio_runtime.block_on(async { Self::setup_api_endpoints(new_environment.clone()) })?;
        let (container_id, logging_process) =
            tokio_runtime.block_on(async { Self::start_container(&docker, agent_image).await })?;

        let runner = Self {
            test_data: Default::default(),
            environment: new_environment,
            tokio_runtime,
            server_process,
            container_logging_process: logging_process,
            docker,
            container_id,
            git_folder,
        };
        Ok(runner)
    }

    /// Block while waiting for the completion of the agent container and all processes around it, like the webserver.
    /// Returns the results of the test run.
    pub fn join(mut self) -> Result<AgentTestFinishedRunner> {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let (server_test_data, own_test_data) = runtime.block_on(async {
            // check for missing post conditions
            let post_conditions = self.environment.post_conditions.clone();
            let uncompleted_post_conditions =
                self.uncompleted_post_conditions(post_conditions).await?;
            if !uncompleted_post_conditions.is_empty() {
                self.test_data.failure = Self::merge_test_failure(
                    self.test_data.failure,
                    Some(TestFailure::ActionsMissing(
                        uncompleted_post_conditions.to_owned(),
                    )),
                )
            }

            // cleanup
            let agent_stop_error = self.join_agent_container().await;
            if let Some(err) = agent_stop_error {
                self.test_data.failure =
                    Self::merge_test_failure(self.test_data.failure, Some(err.clone()));
                self.test_data.log.push(TestLog::CouldNotStopAgent)
            }

            let server_context = self.server_process.await??;
            self.container_logging_process.await?;

            self.tokio_runtime.shutdown_background();

            // check for missing serial actions
            let uncompleted_serial_actions =
                server_context.environment.uncompleted_serial_actions();
            if !uncompleted_serial_actions.is_empty() {
                self.test_data.failure = Self::merge_test_failure(
                    self.test_data.failure,
                    Some(TestFailure::ActionsMissing(
                        uncompleted_serial_actions.to_owned(),
                    )),
                )
            }

            Ok::<(AgentTestFinishedRun, AgentTestFinishedRun), Box<dyn std::error::Error>>((
                server_context.test_data,
                self.test_data,
            ))
        })?;
        Ok(AgentTestFinishedRunner {
            test_data: Self::merge_test_data(server_test_data, own_test_data),
        })
    }

    /// Checks which of the given post conditions were done and deletes them from the queue if yes.
    /// Returns the remaining uncompleted actions
    async fn uncompleted_post_conditions(
        &self,
        actions: Vec<AgentExpectedAction>,
    ) -> Result<Vec<AgentExpectedAction>> {
        let mut action_filter = vec![];
        for action in actions.clone().into_iter() {
            action_filter.push(self.check_post_condition(&action).await);
        }

        if let Some((i, _)) = action_filter.iter().enumerate().find(|(_, v)| v.is_err()) {
            Err(action_filter.into_iter().nth(i).unwrap().unwrap_err())
        } else {
            Ok(action_filter
                .into_iter()
                .zip(actions)
                .filter(|(completed, _)| !completed.as_ref().unwrap())
                .map(|(_, action)| action)
                .collect())
        }
    }

    /// Checks whether the given action is completed
    /// Returns `true` if it is and `false` if not
    async fn check_post_condition(&self, action: &AgentExpectedAction) -> Result<bool> {
        match action {
            AgentExpectedAction::File(name, maybe_content) => {
                Ok(get_files(&self.docker, &self.container_id)
                    .await?
                    .into_iter()
                    .any(|(n, c)| {
                        name == &n && maybe_content.clone().map(|v| v == c).unwrap_or(true)
                    }))
            }

            // This action cannot be checked in parallel
            _ => Ok(true),
        }
    }

    /// Merge two instances of the data accumulated over an agent test run
    fn merge_test_data(
        a: AgentTestFinishedRun,
        mut b: AgentTestFinishedRun,
    ) -> AgentTestFinishedRun {
        let mut log = a.log;
        log.append(&mut b.log);
        AgentTestFinishedRun {
            log,
            failure: Self::merge_test_failure(a.failure, b.failure),
        }
    }

    /// Merge two instances of test results from an agent test run
    fn merge_test_failure(a: Option<TestFailure>, b: Option<TestFailure>) -> Option<TestFailure> {
        // just for fun as oneliner: vec![a, b].into_iter().filter(|t| t.is_some()).reduce(|l, r| Some(TestFailure::Multiple(Box::new(l.unwrap()), Box::new(r.unwrap())))).unwrap_or(None)
        match (a, b) {
            (None, None) => None,
            (Some(f), None) => Some(f),
            (None, Some(f)) => Some(f),
            (Some(a), Some(b)) => Some(TestFailure::Multiple(Box::new(a), Box::new(b))),
        }
    }

    /// Setup docker and start the agent container.
    /// Return the docker instance, the container id and the logging process.
    /// This has to be called from a tokio runtime as a new tokio task is spawned.
    async fn start_container(
        docker: &Docker,
        agent_image: &str,
    ) -> Result<(String, tokio::task::JoinHandle<()>)> {
        let container_id = start_agent_container(docker, agent_image).await?;
        let logging_process = attach_outstreams(docker, &container_id).await?;

        Ok((container_id, logging_process))
    }

    /// Sets up the endpoints the agent can request, e.g. for the Task API or the LLM requests.
    /// Returns a handle to the server process, which, on completion, returns the server context.
    /// This has to be called from a tokio runtime as a new tokio task is spawned.
    fn setup_api_endpoints(
        agent_environment: AgentTestEnvironment,
    ) -> Result<tokio::task::JoinHandle<http_server::Result<ServerContext>>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let server = Server::new(
            addr,
            ServerContext {
                environment: agent_environment,
                test_data: Default::default(),
                terminate_server: false,
            },
        )
        // Task API
        // SUCCESS
        .with_endpoint(APIEndpoint::TaskSuccess.into(), |request, context| {
            Box::pin(api::task_success(request, context))
        })?
        // FAILURE
        .with_endpoint(APIEndpoint::TaskFailure.into(), |request, context| {
            Box::pin(api::task_failure(request, context))
        })?
        // TASK INFO
        .with_endpoint(APIEndpoint::TaskInfo.into(), |request, context| {
            Box::pin(api::task_info(request, context))
        })?
        // LLM
        .with_endpoint(APIEndpoint::LLMRequest.into(), |request, context| {
            Box::pin(api::llm_request(request, context))
        })?
        // GIT
        .with_endpoint(APIEndpoint::GitInfoRefs.into(), |request, context| {
            debug!(
                "info/refs request: {}",
                request.endpoint.query().unwrap_or("empty query")
            );
            Box::pin(git_proxy::routes::info_refs_handler(request, context))
        })?
        .with_endpoint(APIEndpoint::GitReceivePack.into(), |request, context| {
            debug!("git receive pack request");
            Box::pin(git_proxy::routes::git_receive_pack_handler(
                request, context,
            ))
        })?
        .with_endpoint(APIEndpoint::GitUploadPack.into(), |request, context| {
            debug!("git upload pack request");
            Box::pin(git_proxy::routes::git_upload_pack_handler(request, context))
        })?
        // Rest of endpoints
        .with_else_handle(|request| {
            error!("Request on unknown endpoint: >>>{}<<<", request.endpoint);
            HttpResponse::not_found()
        });

        Ok(tokio::spawn(async move { server.run().await }))
    }

    /// Wait for the agent container to stop running and return `None` on success.
    /// If something unexpected happens while stopping, it is treated as a test failure and the cause is returned.
    async fn join_agent_container(&self) -> Option<TestFailure> {
        let mut wait_stream = self.docker.wait_container(
            &self.container_id,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );

        if let Some(Ok(log)) = wait_stream.next().await
            && log.status_code > 0
        {
            return Some(TestFailure::AgentCrashed(format!(
                "Container exited with status code {} and error: {}",
                log.status_code,
                log.error
                    .map(|err| err.message.unwrap_or("No message".to_owned()))
                    .unwrap_or("No error".to_owned())
            )));
        }

        None
    }

    fn initialize_local_git_repo() -> Result<tempfile::TempDir> {
        let temp_dir = tempfile::tempdir()?;
        fs::write(temp_dir.path().join("empty_file_for_git"), "42")?;
        git::execute_git_cmds(
            vec![
                git::GitCmd::Init,
                git::GitCmd::CreateBranch(AGENT_TEST_GIT_BRANCH.to_owned()),
                git::GitCmd::AddAll,
                git::GitCmd::Commit("Initial Commit".to_owned()),
            ],
            temp_dir.path(),
        )?;
        info!("Local git repo initialized in folder {:?}", temp_dir.path());

        Ok(temp_dir)
    }
}

pub struct AgentTestFinishedRunner {
    test_data: AgentTestFinishedRun,
}

impl AgentTestFinishedRunner {
    pub fn test_failure(&self) -> Option<TestFailure> {
        self.test_data.failure.clone()
    }
    pub fn test_logs(&self) -> Vec<TestLog> {
        self.test_data.log.clone()
    }
}
