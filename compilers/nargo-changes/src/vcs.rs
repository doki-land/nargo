#![warn(missing_docs)]

use nargo_types::{Error, Result, Span};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::types::VcsType;

/// Version control system integration.
pub struct VcsIntegration {
    /// The base directory of the repository.
    pub repo_dir: PathBuf,
    /// The detected version control system type.
    pub vcs_type: VcsType,
}

impl VcsIntegration {
    /// Creates a new VCS integration.
    pub fn new(repo_dir: &Path) -> Self {
        let vcs_type = Self::detect_vcs(repo_dir);
        Self { repo_dir: repo_dir.to_path_buf(), vcs_type }
    }

    /// Detects the version control system type in the given directory.
    pub fn detect_vcs(repo_dir: &Path) -> VcsType {
        if repo_dir.join(".git").exists() {
            VcsType::Git
        }
        else if repo_dir.join(".svn").exists() {
            VcsType::Svn
        }
        else if repo_dir.join(".hg").exists() {
            VcsType::Mercurial
        }
        else {
            VcsType::None
        }
    }

    /// Gets the detected version control system type.
    pub fn get_vcs_type(&self) -> VcsType {
        self.vcs_type.clone()
    }

    /// Checks if the directory is a git repository.
    pub fn is_git_repo(&self) -> bool {
        self.repo_dir.join(".git").exists()
    }

    /// Checks if the directory is an SVN repository.
    pub fn is_svn_repo(&self) -> bool {
        self.repo_dir.join(".svn").exists()
    }

    /// Checks if the directory is a Mercurial repository.
    pub fn is_hg_repo(&self) -> bool {
        self.repo_dir.join(".hg").exists()
    }

    /// Gets the current branch for the detected VCS.
    pub fn get_current_branch(&self) -> Result<String> {
        match self.vcs_type {
            VcsType::Git => self.get_git_branch(),
            VcsType::Svn => self.get_svn_branch(),
            VcsType::Mercurial => self.get_hg_branch(),
            VcsType::None => Err(Error::external_error("vcs".to_string(), "No version control system detected".to_string(), Span::unknown())),
        }
    }

    /// Gets the current git branch.
    pub fn get_git_branch(&self) -> Result<String> {
        let output = Command::new("git").arg("branch").arg("--show-current").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get current branch: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the current SVN branch.
    pub fn get_svn_branch(&self) -> Result<String> {
        let output = Command::new("svn").arg("info").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get SVN info: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        for line in output_str.lines() {
            if line.starts_with("URL:") {
                let url = line.split(": ").nth(1).unwrap_or("");
                // Extract branch name from URL (assuming standard SVN layout)
                if let Some(branch_part) = url.split("/branches/").nth(1) {
                    return Ok(branch_part.split("/").next().unwrap_or("").to_string());
                }
                else if url.contains("/trunk/") {
                    return Ok("trunk".to_string());
                }
                else if url.contains("/tags/") {
                    return Ok("tags".to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    /// Gets the current Mercurial branch.
    pub fn get_hg_branch(&self) -> Result<String> {
        let output = Command::new("hg").arg("branch").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get Mercurial branch: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the latest commit hash for the detected VCS.
    pub fn get_latest_commit(&self) -> Result<String> {
        match self.vcs_type {
            VcsType::Git => self.get_git_commit(),
            VcsType::Svn => self.get_svn_commit(),
            VcsType::Mercurial => self.get_hg_commit(),
            VcsType::None => Err(Error::external_error("vcs".to_string(), "No version control system detected".to_string(), Span::unknown())),
        }
    }

    /// Gets the latest git commit hash.
    pub fn get_git_commit(&self) -> Result<String> {
        let output = Command::new("git").arg("rev-parse").arg("HEAD").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get latest commit: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the latest SVN revision.
    pub fn get_svn_commit(&self) -> Result<String> {
        let output = Command::new("svn").arg("info").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get SVN info: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        for line in output_str.lines() {
            if line.starts_with("Revision:") {
                return Ok(line.split(": ").nth(1).unwrap_or("").to_string());
            }
        }

        Err(Error::external_error("vcs".to_string(), "Failed to extract SVN revision".to_string(), Span::unknown()))
    }

    /// Gets the latest Mercurial commit hash.
    pub fn get_hg_commit(&self) -> Result<String> {
        let output = Command::new("hg").arg("identify").arg("--id").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get Mercurial commit: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the status for the detected VCS.
    pub fn get_status(&self) -> Result<String> {
        match self.vcs_type {
            VcsType::Git => self.get_git_status(),
            VcsType::Svn => self.get_svn_status(),
            VcsType::Mercurial => self.get_hg_status(),
            VcsType::None => Err(Error::external_error("vcs".to_string(), "No version control system detected".to_string(), Span::unknown())),
        }
    }

    /// Gets the git status.
    pub fn get_git_status(&self) -> Result<String> {
        let output = Command::new("git").arg("status").arg("--porcelain").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get git status: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Gets the SVN status.
    pub fn get_svn_status(&self) -> Result<String> {
        let output = Command::new("svn").arg("status").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get SVN status: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Gets the Mercurial status.
    pub fn get_hg_status(&self) -> Result<String> {
        let output = Command::new("hg").arg("status").current_dir(&self.repo_dir).output()?;

        if !output.status.success() {
            return Err(Error::external_error("vcs".to_string(), format!("Failed to get Mercurial status: {}", String::from_utf8_lossy(&output.stderr)), Span::unknown()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Generates a change summary for the detected VCS.
    pub fn generate_change_summary(&self) -> Result<String> {
        match self.vcs_type {
            VcsType::Git => self.generate_git_change_summary(),
            VcsType::Svn => self.generate_svn_change_summary(),
            VcsType::Mercurial => self.generate_hg_change_summary(),
            VcsType::None => Err(Error::external_error("vcs".to_string(), "No version control system detected".to_string(), Span::unknown())),
        }
    }

    /// Generates a change summary from git.
    pub fn generate_git_change_summary(&self) -> Result<String> {
        let status = self.get_git_status()?;
        let mut summary = String::new();
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut renamed = 0;

        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_code = &line[0..2];
            match status_code {
                "A " => added += 1,
                "M " => modified += 1,
                "D " => deleted += 1,
                "R " => renamed += 1,
                _ => {}
            }
        }

        summary.push_str(&format!("Git change summary:\n"));
        summary.push_str(&format!("- Added: {}\n", added));
        summary.push_str(&format!("- Modified: {}\n", modified));
        summary.push_str(&format!("- Deleted: {}\n", deleted));
        summary.push_str(&format!("- Renamed: {}\n", renamed));

        if !status.is_empty() {
            summary.push_str("\nDetailed changes:\n");
            summary.push_str(&status);
        }

        Ok(summary)
    }

    /// Generates a change summary from SVN.
    pub fn generate_svn_change_summary(&self) -> Result<String> {
        let status = self.get_svn_status()?;
        let mut summary = String::new();
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut other = 0;

        for line in status.lines() {
            if line.is_empty() {
                continue;
            }

            let status_char = line.chars().next().unwrap_or(' ');
            match status_char {
                'A' => added += 1,
                'M' => modified += 1,
                'D' => deleted += 1,
                _ => other += 1,
            }
        }

        summary.push_str(&format!("SVN change summary:\n"));
        summary.push_str(&format!("- Added: {}\n", added));
        summary.push_str(&format!("- Modified: {}\n", modified));
        summary.push_str(&format!("- Deleted: {}\n", deleted));
        if other > 0 {
            summary.push_str(&format!("- Other: {}\n", other));
        }

        if !status.is_empty() {
            summary.push_str("\nDetailed changes:\n");
            summary.push_str(&status);
        }

        Ok(summary)
    }

    /// Generates a change summary from Mercurial.
    pub fn generate_hg_change_summary(&self) -> Result<String> {
        let status = self.get_hg_status()?;
        let mut summary = String::new();
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut renamed = 0;
        let mut other = 0;

        for line in status.lines() {
            if line.is_empty() {
                continue;
            }

            let status_char = line.chars().next().unwrap_or(' ');
            match status_char {
                'A' => added += 1,
                'M' => modified += 1,
                'D' => deleted += 1,
                'R' => renamed += 1,
                _ => other += 1,
            }
        }

        summary.push_str(&format!("Mercurial change summary:\n"));
        summary.push_str(&format!("- Added: {}\n", added));
        summary.push_str(&format!("- Modified: {}\n", modified));
        summary.push_str(&format!("- Deleted: {}\n", deleted));
        if renamed > 0 {
            summary.push_str(&format!("- Renamed: {}\n", renamed));
        }
        if other > 0 {
            summary.push_str(&format!("- Other: {}\n", other));
        }

        if !status.is_empty() {
            summary.push_str("\nDetailed changes:\n");
            summary.push_str(&status);
        }

        Ok(summary)
    }
}
