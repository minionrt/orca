use std::process::Command;
use std::io;

pub struct CodeSubmission {
    pub commit_message: Option<String>,
}

impl CodeSubmission {

    pub fn new(commit_message: Option<String>) -> Self{
        CodeSubmission {
            commit_message,
        }
    }

    /// this function will create a command "git add ." which adds every change the agent made to the repo
    pub fn add_changes(&self) -> io::Result<()> {
     println!("git adding changed files...");
        let status = Command::new("git")
            .arg("add")
            .arg(".")
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git add failed"));
        }
        Ok(())
    }

    /// the commit message will be created by the agent (TODO)
    pub fn commit_changes(&self) -> io::Result<()> {

    let c_message = self.commit_message.as_deref().unwrap_or("No message :)");        

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(c_message)
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git commit failed"));
        }
        Ok(())
    }

    /// pushes the previously committed changes
    /// origin is main-connection to the remote repository
    /// HEAD is the symbolic pointer to the current branch
    /// this should work on all branches
    // please create an issue, if this causes an error
    pub fn push_changes(&self) -> io::Result<()> {
        
        let status = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("HEAD")
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git push failed"));
        }
        Ok(())
    }

    pub fn submit_changes(&self) -> io::Result<()> {
        self.add_changes()?;
        self.commit_changes()?;
        self.push_changes()?;
        Ok(())
    }
}