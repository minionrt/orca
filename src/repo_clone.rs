use std::process::{Command, Stdio};
use std::fs;
use std::io;

pub struct GitRepository {

    pub repo_url: String,
    pub branch: String,
    pub user_name: String,
    pub user_email: String,
    pub target_dir: String,
}

impl GitRepository {

    /// Initializes the GitRepository struct
    pub fn new(repo_url: &str, branch: &str, user_name: &str, user_email: &str, target_dir: &str) -> Self {
        GitRepository {
            repo_url: repo_url.to_string(),
            branch: branch.to_string(),
            user_name: user_name.to_string(),
            user_email: user_email.to_string(),
            target_dir: target_dir.to_string(),
        }
    }

    /// Sets the git user name and email
    pub fn configure_git_user(&self) -> io::Result<()> {

        let config_name = Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg(&self.user_name)
            .current_dir(&self.target_dir)
            .status()?;

        let config_email = Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg(&self.user_email)
            .current_dir(&self.target_dir)
            .status()?;

        if !config_name.success() || !config_email.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Git config failed"));
        }

        Ok(())
    }

    /// Clones the repository to the target directory
    pub fn clone_repo(&self) -> io::Result<()> {
        println!("Cloning repo: {} into {}", self.repo_url, self.target_dir);
        // The git command is build
        let status = Command::new("git")
            .arg("clone")
            .arg(&self.repo_url)
            .arg(&self.target_dir)
            .status()?;
        
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Git clone failed"));
        }

        Ok(())
    }

    /// Checks out the specified branch
    pub fn checkout_branch(&self) -> io::Result<()> {

        let status = Command::new("git")
            .arg("checkout")
            .arg(&self.branch)
            .current_dir(&self.target_dir)
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Git checkout failed"));
        }

        Ok(())
    }

    /// Executes the complete setup flow: clone, configure, checkout
    pub fn prepare_repository(&self) -> io::Result<()> {

        self.clone_repo()?;
        self.configure_git_user()?;
        self.checkout_branch()?;
        Ok(())
    }
}
