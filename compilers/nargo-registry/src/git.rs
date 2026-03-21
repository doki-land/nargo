use gix;
use nargo_types::{Error, NargoValue, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::types::{DependencySpec, GitReference, ResolvedDependency, ResolvedSource};

/// Git dependency resolver.
#[derive(Debug)]
pub struct GitResolver {
    /// Cache directory for cloned repositories.
    cache_dir: PathBuf,
}

impl GitResolver {
    /// Creates a new Git resolver with the specified cache directory.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Creates a new Git resolver with default cache directory.
    pub fn with_default_cache() -> Self {
        Self { cache_dir: PathBuf::from(".nargo-cache").join("git") }
    }

    /// Parses a Git dependency URL.
    ///
    /// Supports formats:
    /// - `git+https://github.com/user/repo.git`
    /// - `git+https://github.com/user/repo.git#v1.0.0`
    /// - `git+https://github.com/user/repo.git#branch-name`
    /// - `git+https://github.com/user/repo.git#commit-hash`
    /// - `git+ssh://git@github.com:user/repo.git`
    /// - `github:user/repo` (shorthand)
    /// - `github:user/repo#v1.0.0` (shorthand with reference)
    pub fn parse_git_url(input: &str) -> Result<DependencySpec> {
        if input.starts_with("github:") {
            return Self::parse_github_shorthand(input);
        }

        if !input.starts_with("git+") {
            return Err(Error::external_error("git".to_string(), "Invalid Git URL: must start with git+ or github:".to_string(), nargo_types::Span::unknown()));
        }

        let url_part = &input[4..];

        let (url, reference) = if let Some(hash_pos) = url_part.rfind('#') {
            let (url, ref_str) = url_part.split_at(hash_pos);
            let ref_str = &ref_str[1..];
            (url, Some(Self::parse_git_reference(ref_str)?))
        }
        else {
            (url_part, None)
        };

        Ok(DependencySpec::Git { url: url.to_string(), reference })
    }

    /// Parses GitHub shorthand format.
    fn parse_github_shorthand(input: &str) -> Result<DependencySpec> {
        let input = &input[7..];

        let (repo, reference) = if let Some(hash_pos) = input.rfind('#') {
            let (repo, ref_str) = input.split_at(hash_pos);
            let ref_str = &ref_str[1..];
            (repo, Some(Self::parse_git_reference(ref_str)?))
        }
        else {
            (input, None)
        };

        Ok(DependencySpec::Github { repo: repo.to_string(), reference })
    }

    /// Parses a Git reference string.
    fn parse_git_reference(ref_str: &str) -> Result<GitReference> {
        if ref_str.len() == 40 && ref_str.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(GitReference::Commit(ref_str.to_string()))
        }
        else if ref_str.starts_with("v") || ref_str.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            Ok(GitReference::Tag(ref_str.to_string()))
        }
        else {
            Ok(GitReference::Branch(ref_str.to_string()))
        }
    }

    /// Resolves a Git dependency.
    ///
    /// # Arguments
    /// * `spec` - The Git dependency specification.
    ///
    /// # Returns
    /// The resolved dependency with local path.
    pub async fn resolve(&self, spec: &DependencySpec) -> Result<ResolvedDependency> {
        match spec {
            DependencySpec::Git { url, reference } => self.clone_and_checkout(url, reference, None).await,
            DependencySpec::Github { repo, reference } => {
                let url = format!("https://github.com/{}.git", repo);
                self.clone_and_checkout(&url, reference, Some(repo)).await
            }
            _ => Err(Error::external_error("git".to_string(), "Not a Git dependency".to_string(), nargo_types::Span::unknown())),
        }
    }

    /// Clones a repository and checks out the specified reference.
    async fn clone_and_checkout(&self, url: &str, reference: &Option<GitReference>, repo_name: Option<&str>) -> Result<ResolvedDependency> {
        let repo_dir = self.get_repo_dir(url, repo_name)?;

        let repo = if repo_dir.exists() {
            debug!("Opening existing repository: {}", repo_dir.display());
            gix::open(&repo_dir).map_err(|e| Error::external_error("git".to_string(), format!("Failed to open existing repository: {}", e), nargo_types::Span::unknown()))?
        }
        else {
            debug!("Cloning repository: {}", url);
            self.clone_repository(url, &repo_dir).await?
        };

        let commit = self.checkout_reference(&repo, reference)?;

        let package_name = self.extract_package_name(&repo)?;

        Ok(ResolvedDependency { name: package_name, version: commit.clone(), resolved: ResolvedSource::GitRepo { path: repo_dir, commit }, integrity: None })
    }

    /// Gets the directory for a repository in the cache.
    fn get_repo_dir(&self, url: &str, repo_name: Option<&str>) -> Result<PathBuf> {
        let dir_name = if let Some(name) = repo_name {
            name.replace('/', "_")
        }
        else {
            let url_hash = sha256_hash(url);
            format!("repo_{}", &url_hash[..12])
        };
        Ok(self.cache_dir.join(dir_name))
    }

    /// Clones a repository.
    async fn clone_repository(&self, url: &str, target_dir: &Path) -> Result<gix::Repository> {
        std::fs::create_dir_all(target_dir).map_err(|e| Error::external_error("git".to_string(), format!("Failed to create repo directory: {}", e), nargo_types::Span::unknown()))?;

        let repo = gix::open(target_dir).or_else(|_| {
            use std::sync::atomic::AtomicBool;

            let should_interrupt = AtomicBool::new(false);
            let (_prepare_checkout, _outcome) = gix::prepare_clone(url, target_dir).map_err(|e| Error::external_error("git".to_string(), format!("Failed to prepare clone: {}", e), nargo_types::Span::unknown()))?.fetch_then_checkout(gix::progress::Discard, &should_interrupt).map_err(|e| Error::external_error("git".to_string(), format!("Failed to clone repository: {}", e), nargo_types::Span::unknown()))?;

            let repo = gix::open(target_dir).map_err(|e| Error::external_error("git".to_string(), format!("Failed to open cloned repository: {}", e), nargo_types::Span::unknown()))?;

            Ok::<gix::Repository, Error>(repo)
        })?;

        info!("Cloned repository to: {}", target_dir.display());
        Ok(repo)
    }

    /// Checks out a specific reference in the repository.
    fn checkout_reference(&self, repo: &gix::Repository, reference: &Option<GitReference>) -> Result<String> {
        let commit_id = match reference {
            Some(GitReference::Branch(branch)) => {
                let ref_name = format!("refs/remotes/origin/{}", branch);
                let mut reference = repo.find_reference(&ref_name).map_err(|_| Error::external_error("git".to_string(), format!("Reference not found: {}", branch), nargo_types::Span::unknown()))?;
                let object = reference.peel_to_commit().map_err(|e| Error::external_error("git".to_string(), format!("Failed to peel reference: {}", e), nargo_types::Span::unknown()))?;
                object.id().to_string()
            }
            Some(GitReference::Tag(tag)) => {
                let ref_name = format!("refs/tags/{}", tag);
                let mut reference = repo.find_reference(&ref_name).map_err(|_| Error::external_error("git".to_string(), format!("Reference not found: {}", tag), nargo_types::Span::unknown()))?;
                let object = reference.peel_to_commit().map_err(|e| Error::external_error("git".to_string(), format!("Failed to peel reference: {}", e), nargo_types::Span::unknown()))?;
                object.id().to_string()
            }
            Some(GitReference::Commit(hash)) => hash.clone(),
            None => {
                let mut reference = repo.find_reference("refs/remotes/origin/HEAD").or_else(|_| repo.find_reference("refs/remotes/origin/main")).or_else(|_| repo.find_reference("refs/remotes/origin/master")).map_err(|e| Error::external_error("git".to_string(), format!("Failed to find default branch: {}", e), nargo_types::Span::unknown()))?;
                let object = reference.peel_to_commit().map_err(|e| Error::external_error("git".to_string(), format!("Failed to peel reference: {}", e), nargo_types::Span::unknown()))?;
                object.id().to_string()
            }
        };

        let _commit = repo.find_object(gix::ObjectId::from_hex(commit_id.as_bytes()).map_err(|e| Error::external_error("git".to_string(), format!("Invalid commit hash: {}", e), nargo_types::Span::unknown()))?).map_err(|e| Error::external_error("git".to_string(), format!("Failed to find commit: {}", e), nargo_types::Span::unknown()))?;

        info!("Checked out commit: {}", commit_id);
        Ok(commit_id)
    }

    /// Extracts the package name from the repository.
    fn extract_package_name(&self, repo: &gix::Repository) -> Result<String> {
        let workdir = repo.workdir().ok_or_else(|| Error::external_error("git".to_string(), "Repository has no working directory".to_string(), nargo_types::Span::unknown()))?;

        let package_json_path = workdir.join("package.json");
        if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path).map_err(|e| Error::external_error("git".to_string(), format!("Failed to read package.json: {}", e), nargo_types::Span::unknown()))?;
            let json: NargoValue = serde_json::from_str(&content).map_err(|e| Error::external_error("git".to_string(), format!("Failed to parse package.json: {}", e), nargo_types::Span::unknown()))?;
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                return Ok(name.to_string());
            }
        }

        let dir_name = workdir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        Ok(dir_name.to_string())
    }
}

fn sha256_hash(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
