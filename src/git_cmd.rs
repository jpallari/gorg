use std::{ffi::OsStr, path::Path, process::Command};

use anyhow::{Result, bail};

pub struct GitCmd {
    git_command: String,
}

impl GitCmd {
    pub fn new(git_command: String) -> Self {
        Self { git_command }
    }

    pub fn init<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let status = Command::new(&self.git_command)
            .args(["init"])
            .current_dir(&dir)
            .spawn()?
            .wait()?;
        if !status.success() {
            bail!(
                "Failed to init Git in {}: exit code = {:?}",
                dir.as_ref().to_string_lossy(),
                status.code()
            );
        }
        Ok(())
    }

    pub fn clone_repo<P: AsRef<OsStr>>(&self, repo_url: &str, dir: P) -> Result<()> {
        let status = Command::new(&self.git_command)
            .args([
                OsStr::new("clone"),
                OsStr::new("--"),
                OsStr::new(repo_url),
                &dir.as_ref(),
            ])
            .spawn()?
            .wait()?;
        if !status.success() {
            bail!(
                "Failed to clone {repo_url} to {}: exit code = {:?}",
                dir.as_ref().to_string_lossy(),
                status.code(),
            );
        }
        Ok(())
    }

    pub fn remote_list<P: AsRef<Path>>(&self, dir: P) -> Result<String> {
        let output = Command::new(&self.git_command)
            .args(["remote"])
            .current_dir(&dir)
            .output()?;
        let remotes = String::from_utf8(output.stdout)?;
        Ok(remotes)
    }

    pub fn remote_add<P: AsRef<Path>>(
        &self,
        remote_name: &str,
        repo_url: &str,
        dir: P,
    ) -> Result<()> {
        let status = Command::new(&self.git_command)
            .args([
                OsStr::new("remote"),
                OsStr::new("add"),
                OsStr::new(remote_name),
                OsStr::new(repo_url),
            ])
            .current_dir(&dir)
            .spawn()?
            .wait()?;
        if !status.success() {
            bail!(
                "Failed to add remote URL {repo_url} for {}: exit code = {:?}",
                dir.as_ref().to_string_lossy(),
                status.code()
            );
        }
        Ok(())
    }

    pub fn remote_set_url<P: AsRef<Path>>(
        &self,
        remote_name: &str,
        repo_url: &str,
        dir: P,
    ) -> Result<()> {
        let status = Command::new(&self.git_command)
            .args([
                OsStr::new("remote"),
                OsStr::new("set-url"),
                OsStr::new(remote_name),
                OsStr::new(repo_url),
            ])
            .current_dir(&dir)
            .spawn()?
            .wait()?;
        if !status.success() {
            bail!(
                "Failed to set remote URL {repo_url} for {}: exit code = {:?}",
                dir.as_ref().to_string_lossy(),
                status.code()
            );
        }
        Ok(())
    }
}
