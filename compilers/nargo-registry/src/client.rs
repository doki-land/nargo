//! NPM Registry client implementation.

use crate::signature::{KeyPair, PublicKey, sign_package, verify_package};
use base64::{Engine as _, engine::general_purpose};
use nargo_types::{Error, NargoValue, Result, Span};
use sha2::{Digest, Sha512};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::Arc,
};
use tar::Builder;
use tempfile::{tempdir, tempfile};
use tokio::{fs, io::AsyncWriteExt};
use wae_request::{HttpClient, HttpClientConfig, RequestBuilder};

/// Error type for registry operations.
pub type RegistryError = nargo_types::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{semver::resolve_version, types::*};

/// NPM Registry API client.
#[derive(Debug, Clone)]
pub struct RegistryClient {
    /// HTTP client.
    client: HttpClient,
    /// Registry configuration.
    config: RegistryConfig,
    /// Metadata cache.
    metadata_cache: Arc<RwLock<MetadataCache>>,
}

/// In-memory cache for package metadata.
#[derive(Debug, Default)]
struct MetadataCache {
    /// Cached metadata entries.
    entries: HashMap<String, (PackageMetadata, std::time::Instant)>,
    /// Cache TTL in seconds.
    ttl_secs: u64,
}

impl MetadataCache {
    fn new(ttl_secs: u64) -> Self {
        Self { entries: HashMap::new(), ttl_secs }
    }

    fn get(&self, key: &str) -> Option<&PackageMetadata> {
        self.entries.get(key).and_then(|(meta, time)| if time.elapsed().as_secs() < self.ttl_secs { Some(meta) } else { None })
    }

    fn insert(&mut self, key: String, meta: PackageMetadata) {
        self.entries.insert(key, (meta, std::time::Instant::now()));
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

use std::collections::HashMap;

impl RegistryClient {
    /// Creates a new registry client with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(RegistryConfig::default())
    }

    /// Creates a new registry client with custom configuration.
    pub fn with_config(config: RegistryConfig) -> Result<Self> {
        let http_config = HttpClientConfig { timeout: std::time::Duration::from_secs(config.timeout_secs), connect_timeout: std::time::Duration::from_secs(10), user_agent: format!("nargo-registry/{}", env!("CARGO_PKG_VERSION")), max_retries: 3, retry_delay: std::time::Duration::from_millis(1000), default_headers: HashMap::new() };

        let client = HttpClient::new(http_config);

        let metadata_cache = Arc::new(RwLock::new(MetadataCache::new(config.cache_ttl_secs)));

        Ok(Self { client, config, metadata_cache })
    }

    /// Creates a client for a specific registry.
    pub fn for_registry(registry_url: &str) -> Result<Self> {
        let mut config = RegistryConfig::default();
        config.registry_url = registry_url.to_string();
        Self::with_config(config)
    }

    /// Sets the authentication token.
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.config.auth_token = Some(token.into());
        self
    }

    /// Creates a request builder with appropriate headers.
    fn create_request(&self, url: &str) -> RequestBuilder {
        let builder = wae_request::get(url);

        if let Some(ref token) = self.config.auth_token { builder.bearer_auth(token) } else { builder }
    }

    /// Fetches package metadata from the registry.
    pub async fn get_package_metadata(&self, package_name: &str) -> Result<PackageMetadata> {
        let cache_key = format!("{}/{}", self.config.registry_url, package_name);

        {
            let cache = self.metadata_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                debug!("Using cached metadata for {}", package_name);
                return Ok(cached.clone());
            }
        }

        let url = format!("{}/{}", self.config.registry_url, package_name);
        debug!("Fetching package metadata: {}", url);

        let response = self.create_request(&url).send().await.map_err(|e| Error::external_error("registry".to_string(), format!("Network error: {}", e), Span::unknown()))?;

        match response.status {
            200 => {
                let metadata: PackageMetadata = response.json().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to parse package metadata for {}: {}", package_name, e), Span::unknown()))?;

                {
                    let mut cache = self.metadata_cache.write().await;
                    cache.insert(cache_key, metadata.clone());
                }

                Ok(metadata)
            }
            404 => Err(Error::external_error("registry".to_string(), format!("Package not found: {}", package_name), Span::unknown()).into()),
            401 => Err(Error::external_error("registry".to_string(), format!("Authentication failed for registry: {}", self.config.registry_url), Span::unknown()).into()),
            429 => {
                let retry_after = response.headers.get("retry-after").and_then(|v| v.parse::<u64>().ok()).unwrap_or(60);
                Err(Error::external_error("registry".to_string(), format!("Rate limited by registry, retry after {} seconds", retry_after), Span::unknown()).into())
            }
            status => Err(Error::external_error("registry".to_string(), format!("Unexpected status code {}: for package {}", status, package_name), Span::unknown()).into()),
        }
    }

    /// Fetches a specific version of a package.
    pub async fn get_package_version(&self, package_name: &str, version: &str) -> Result<PackageVersion> {
        let metadata = self.get_package_metadata(package_name).await?;

        metadata.versions.get(version).cloned().ok_or_else(|| Error::external_error("registry".to_string(), format!("Version {} not found for package {}", version, package_name), Span::unknown()).into())
    }

    /// Searches for packages in the registry.
    pub async fn search_packages(&self, query: &str, size: u32) -> Result<SearchResult> {
        let url = format!("{}/-/v1/search?text={}&size={}", self.config.registry_url, urlencoding::encode(query), size);
        debug!("Searching packages: {}", url);

        let response = self.create_request(&url).send().await.map_err(|e| Error::external_error("registry".to_string(), format!("Network error: {}", e), Span::unknown()))?;

        match response.status {
            200 => {
                let result: SearchResult = response.json().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to parse search results: {}", e), Span::unknown()))?;
                Ok(result)
            }
            status => Err(Error::external_error("registry".to_string(), format!("Search failed with status code {}", status), Span::unknown()).into()),
        }
    }

    /// Resolves a package version constraint to a specific version.
    pub async fn resolve_package(&self, package_name: &str, constraint: &str) -> Result<(String, PackageVersion)> {
        let metadata = self.get_package_metadata(package_name).await?;

        let versions: Vec<String> = metadata.versions.keys().cloned().collect();
        let resolved_version = resolve_version(constraint, &versions, &metadata.dist_tags).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to resolve version {} for {}: {}", constraint, package_name, e), Span::unknown()))?;

        let version_info = metadata.versions.get(&resolved_version).cloned().ok_or_else(|| Error::external_error("registry".to_string(), format!("Version {} not found for package {}", resolved_version, package_name), Span::unknown()))?;

        Ok((resolved_version, version_info))
    }

    /// Gets all versions of a package.
    pub async fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let metadata = self.get_package_metadata(package_name).await?;
        Ok(metadata.versions.keys().cloned().collect())
    }

    /// Gets the latest version of a package.
    pub async fn get_latest_version(&self, package_name: &str) -> Result<String> {
        let metadata = self.get_package_metadata(package_name).await?;
        metadata.dist_tags.get("latest").cloned().ok_or_else(|| Error::external_error("registry".to_string(), format!("No latest version found for {}", package_name), Span::unknown()).into())
    }

    /// Gets dist-tags for a package.
    pub async fn get_dist_tags(&self, package_name: &str) -> Result<HashMap<String, String>> {
        let metadata = self.get_package_metadata(package_name).await?;
        Ok(metadata.dist_tags)
    }

    /// Downloads a tarball to the cache directory.
    pub async fn download_tarball(&self, tarball_url: &str, package_name: &str, version: &str, integrity: Option<&str>) -> Result<PathBuf> {
        self.download_tarball_with_progress(tarball_url, package_name, version, integrity, None).await
    }

    /// Downloads a tarball with progress callback.
    pub async fn download_tarball_with_progress(&self, tarball_url: &str, package_name: &str, version: &str, integrity: Option<&str>, progress_callback: Option<AcceptedCallback>) -> Result<PathBuf> {
        let cache_dir = self.config.cache_dir.join("packages").join(package_name);
        fs::create_dir_all(&cache_dir).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create cache directory: {}", e), Span::unknown()))?;

        let tarball_name = format!("{}-{}.tgz", package_name.replace('/', "_"), version);
        let tarball_path = cache_dir.join(&tarball_name);

        if tarball_path.exists() {
            info!("Tarball already cached: {}", tarball_path.display());
            return Ok(tarball_path);
        }

        info!("Downloading tarball: {}", tarball_url);
        let response = self.create_request(tarball_url).send().await.map_err(|e| Error::external_error("registry".to_string(), format!("Network error: {}", e), Span::unknown()))?;

        if !response.is_success() {
            return Err(Error::external_error("registry".to_string(), format!("Failed to download tarball: HTTP {}", response.status), Span::unknown()).into());
        }

        let total_size = response.headers.get("content-length").and_then(|v| v.parse::<u64>().ok());
        let temp_path = tarball_path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create temp file: {}", e), Span::unknown()))?;

        let mut hasher = Sha512::new();
        let mut downloaded: u64 = 0;

        // Write the entire body at once since we have it in memory
        hasher.update(&response.body);
        file.write_all(&response.body).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to write file: {}", e), Span::unknown()))?;
        downloaded = response.body.len() as u64;

        if let Some(ref callback) = progress_callback {
            let progress = DownloadProgress { total: total_size, downloaded };
            callback(progress);
        }

        file.flush().await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to flush file: {}", e), Span::unknown()))?;
        drop(file);

        let actual_hash = format!("sha512-{}", hex::encode(hasher.finalize()));

        if let Some(expected) = integrity {
            if expected != actual_hash {
                fs::remove_file(&temp_path).await.ok();
                return Err(Error::external_error("registry".to_string(), format!("Integrity check failed for {}@{}, expected: {}, got: {}", package_name, version, expected, actual_hash), Span::unknown()).into());
            }
        }

        fs::rename(&temp_path, &tarball_path).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to rename temp file: {}", e), Span::unknown()))?;

        info!("Tarball downloaded: {}", tarball_path.display());
        Ok(tarball_path)
    }

    /// Downloads multiple packages in parallel.
    pub async fn download_packages_parallel(&self, packages: &[(&str, &str)], max_concurrent: usize) -> Result<Vec<ResolvedDependency>> {
        use futures::stream::{self, StreamExt};

        let results: Vec<Result<ResolvedDependency>> = stream::iter(packages).map(|(name, constraint)| async move { self.download_package(name, constraint).await }).buffer_unordered(max_concurrent).collect().await;

        let mut resolved = Vec::new();
        for result in results {
            resolved.push(result?);
        }

        Ok(resolved)
    }

    /// Downloads a package and returns the resolved dependency.
    pub async fn download_package(&self, package_name: &str, constraint: &str) -> Result<ResolvedDependency> {
        let (version, version_info) = self.resolve_package(package_name, constraint).await?;

        let tarball_path = self.download_tarball(&version_info.dist.tarball, package_name, &version, version_info.dist.integrity.as_deref()).await?;

        // Verify package signature if verification key is configured
        if let Some(verification_key_path) = &self.config.verification_key_path {
            // Check if the package has a signature in dist info
            if let Some(signature_str) = &version_info.dist.npm_signature {
                let public_key = PublicKey::load(verification_key_path).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to load verification key: {}", e), Span::unknown()))?;
                let signature = general_purpose::STANDARD.decode(signature_str).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to decode signature: {}", e), Span::unknown()))?;
                let verified = verify_package(&tarball_path, &signature, &public_key).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to verify package signature: {}", e), Span::unknown()))?;
                if !verified {
                    return Err(Error::external_error("registry".to_string(), format!("Invalid signature for package {}@{}", package_name, version), Span::unknown()).into());
                }
            }
        }

        Ok(ResolvedDependency { name: package_name.to_string(), version: version.clone(), resolved: ResolvedSource::Tarball(tarball_path.to_string_lossy().to_string()), integrity: version_info.dist.integrity })
    }

    /// Extracts a tarball to a directory.
    pub async fn extract_tarball(&self, tarball_path: &PathBuf, target_dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(target_dir).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create target directory: {}", e), Span::unknown()))?;

        let tarball = fs::read(tarball_path).await.map_err(|e| Error::external_error("registry".to_string(), format!("Failed to read tarball: {}", e), Span::unknown()))?;

        let cursor = std::io::Cursor::new(tarball);
        let gz_decoder = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(gz_decoder);

        archive.unpack(target_dir).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to extract tarball: {}", e), Span::unknown()))?;

        info!("Extracted tarball to: {}", target_dir.display());
        Ok(())
    }

    /// Clears the metadata cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.metadata_cache.write().await;
        cache.clear();
    }

    /// Returns the registry URL.
    pub fn registry_url(&self) -> &str {
        &self.config.registry_url
    }

    /// Returns the cache directory.
    pub fn cache_dir(&self) -> &PathBuf {
        &self.config.cache_dir
    }

    /// Publishes a package to the registry.
    pub async fn publish_package(&self, package_dir: &PathBuf) -> Result<()> {
        use flate2::Compression;
        use std::{fs::File, io::Read};
        use tar::Builder;

        // Read package.json
        let package_json_path = package_dir.join("package.json");
        let mut package_json_file = File::open(&package_json_path).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to open package.json: {}", e), Span::unknown()))?;
        let mut package_json_content = String::new();
        package_json_file.read_to_string(&mut package_json_content).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to read package.json: {}", e), Span::unknown()))?;

        // Parse package.json
        let package_info: NargoValue = serde_json::from_str(&package_json_content).map_err(|e| Error::external_error("registry".to_string(), format!("Invalid package.json: {}", e), Span::unknown()))?;

        // Create tarball
        let temp_tarball = tempfile::tempfile().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create temp file: {}", e), Span::unknown()))?;
        let enc = flate2::write::GzEncoder::new(temp_tarball, Compression::default());
        let mut tar_builder = Builder::new(enc);

        // Add files to tarball
        self.add_files_to_tarball(&mut tar_builder, package_dir).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create tarball: {}", e), Span::unknown()))?;

        let enc = tar_builder.into_inner().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to finish tarball: {}", e), Span::unknown()))?;
        let mut temp_tarball = enc.finish().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to finish compression: {}", e), Span::unknown()))?;

        // Reset cursor to start
        temp_tarball.seek(std::io::SeekFrom::Start(0)).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to seek in temp file: {}", e), Span::unknown()))?;

        // Read tarball content
        let mut tarball_content = Vec::new();
        temp_tarball.read_to_end(&mut tarball_content).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to read tarball: {}", e), Span::unknown()))?;

        // Create publish request
        let url = format!("{}/{}", self.config.registry_url, package_info.get("name").and_then(|v| v.as_str()).unwrap_or_default());
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/octet-stream".to_string());

        if let Some(ref token) = self.config.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        // Sign the package if signing key is configured
        if let Some(signing_key_path) = &self.config.signing_key_path {
            let temp_tarball_path = tempfile::tempdir().map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create temp directory: {}", e), Span::unknown()))?.path().join("package.tgz");
            let mut temp_file = File::create(&temp_tarball_path).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to create temp file: {}", e), Span::unknown()))?;
            temp_file.write_all(&tarball_content).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to write temp file: {}", e), Span::unknown()))?;

            let key_pair = KeyPair::load(signing_key_path).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to load signing key: {}", e), Span::unknown()))?;
            let signature = sign_package(&temp_tarball_path, &key_pair).map_err(|e| Error::external_error("registry".to_string(), format!("Failed to sign package: {}", e), Span::unknown()))?;

            headers.insert("X-Package-Signature".to_string(), general_purpose::STANDARD.encode(&signature));
        }

        // Send publish request
        let mut request = RequestBuilder::new(self.client.clone(), "PUT", &url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        let response = request.body(tarball_content).send().await.map_err(|e| Error::external_error("registry".to_string(), format!("Network error: {}", e), Span::unknown()))?;

        match response.status {
            201 => {
                info!("Package published successfully");
                Ok(())
            }
            401 => Err(Error::external_error("registry".to_string(), format!("Authentication failed for registry: {}", self.config.registry_url), Span::unknown()).into()),
            403 => Err(Error::external_error("registry".to_string(), format!("Permission denied: {}", response.text().unwrap_or_default()), Span::unknown()).into()),
            409 => Err(Error::external_error("registry".to_string(), format!("Package version already exists"), Span::unknown()).into()),
            status => Err(Error::external_error("registry".to_string(), format!("Publish failed with status code {}: {}", status, response.text().unwrap_or_default()), Span::unknown()).into()),
        }
    }

    /// Adds files to tarball, excluding node_modules and other ignored files.
    fn add_files_to_tarball(&self, tar_builder: &mut Builder<impl std::io::Write>, package_dir: &PathBuf) -> std::io::Result<()> {
        let ignored_patterns = ["node_modules", ".git", ".nargo-cache", "*.tgz"];

        for entry in walkdir::WalkDir::new(package_dir) {
            let entry = entry?;
            let path = entry.path();

            // Skip ignored patterns
            if ignored_patterns.iter().any(|pattern| path.to_string_lossy().contains(pattern)) {
                continue;
            }

            if path.is_file() {
                let relative_path = path.strip_prefix(package_dir).unwrap_or(path);
                let mut file = std::fs::File::open(path)?;
                let mut file_content = Vec::new();
                file.read_to_end(&mut file_content)?;

                let mut header = tar::Header::new_gnu();
                header.set_size(file_content.len() as u64);
                header.set_mode(0o644);
                header.set_path(relative_path)?;

                tar_builder.append(&header, std::io::Cursor::new(file_content))?;
            }
        }

        Ok(())
    }

    /// Unpublishes a package or specific version from the registry.
    pub async fn unpublish_package(&self, package_name: &str, version: Option<&str>) -> Result<()> {
        let url = if let Some(version) = version { format!("{}/{}/-rev/{}", self.config.registry_url, package_name, version) } else { format!("{}/{}", self.config.registry_url, package_name) };

        let mut headers = HashMap::new();
        if let Some(ref token) = self.config.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let mut request = RequestBuilder::new(self.client.clone(), "DELETE", &url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        let response = request.send().await.map_err(|e| Error::external_error("registry".to_string(), format!("Network error: {}", e), Span::unknown()))?;

        match response.status {
            200 => {
                info!("Package unpublished successfully");
                Ok(())
            }
            401 => Err(Error::external_error("registry".to_string(), format!("Authentication failed for registry: {}", self.config.registry_url), Span::unknown()).into()),
            403 => Err(Error::external_error("registry".to_string(), format!("Permission denied: {}", response.text().unwrap_or_default()), Span::unknown()).into()),
            404 => Err(Error::external_error("registry".to_string(), format!("Package or version not found: {}", package_name), Span::unknown()).into()),
            status => Err(Error::external_error("registry".to_string(), format!("Unpublish failed with status code {}: {}", status, response.text().unwrap_or_default()), Span::unknown()).into()),
        }
    }
}

/// Type alias for progress callback.
type AcceptedCallback = std::sync::Arc<dyn Fn(DownloadProgress) + Send + Sync>;

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default registry client")
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
