// Small git interface to improve readability in the AgentTestRunner

use std::{fmt, path::Path, process::Command};

#[derive(Debug)]
pub enum GitError {
    /// Error when initializing a Git repository.
    Init,
    /// Error when committing changes to a Git repository.
    Commit,
    /// Error when adding all files to a Git repository.
    AddAll,
    /// Error when creating a new branch in a Git repository.
    CreateBranch,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GitError::Init => write!(f, "Git init failed"),
            GitError::Commit => write!(f, "Git commit failed"),
            GitError::AddAll => write!(f, "Git add failed"),
            GitError::CreateBranch => {
                write!(f, "Git create branch failed")
            }
        }
    }
}

impl std::error::Error for GitError {}

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn error_on_false(err: GitError, result: bool) -> Result {
    if result { Ok(()) } else { Err(err.into()) }
}

/// Enum representing different Git commands that can be executed.
pub enum GitCmd {
    /// Initialize a new Git repository.
    Init,
    /// Commit staged changes to the repository with the given commit message.
    Commit(String),
    /// Add all files in the current directory to the Git repository.
    AddAll,
    /// Create a new branch in the repository with the given name.
    CreateBranch(String),
}

/// Execute multiple Git commands on a repository at the specified path.
///
/// # Arguments
///
/// * `cmds` - A vector of `GitCmd`s to be executed.
/// * `path` - The path to the Git repository.
pub fn execute_git_cmds(cmds: Vec<GitCmd>, path: &Path) -> Result {
    for cmd in cmds.into_iter() {
        match cmd {
            GitCmd::Init => init_repo(path)?,
            GitCmd::Commit(msg) => commit(path, &msg)?,
            GitCmd::AddAll => add_all(path)?,
            GitCmd::CreateBranch(name) => create_branch(path, &name)?,
        }
    }
    Ok(())
}

fn init_repo(path: &Path) -> Result {
    error_on_false(
        GitError::Init,
        Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(path)
            .status()?
            .success(),
    )
}

fn create_branch(path: &Path, name: &str) -> Result {
    error_on_false(
        GitError::CreateBranch,
        Command::new("git")
            .arg("switch")
            .arg("-c")
            .arg(name)
            .current_dir(path)
            .status()?
            .success(),
    )
}

fn commit(path: &Path, msg: &str) -> Result {
    error_on_false(
        GitError::Commit,
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(path)
            .status()?
            .success(),
    )
}

fn add_all(path: &Path) -> Result {
    error_on_false(
        GitError::AddAll,
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(path)
            .status()?
            .success(),
    )
}
