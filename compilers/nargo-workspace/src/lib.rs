//! Workspace management for Nargo package manager.
//!
//! This crate provides workspace discovery, member management, and
//! task orchestration for multi-package projects.

#![warn(missing_docs)]

use nargo_types::{Error, Result, Span};
use oak_toml;
use oak_json;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

/// Represents a workspace member package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// Member package name.
    pub name: String,
    /// Path to the member package.
    pub path: PathBuf,
    /// Member package version.
    pub version: String,
    /// Whether this is the root package.
    pub is_root: bool,
    /// Dependencies of this member.
    pub dependencies: Vec<String>,
    /// Dev dependencies of this member.
    pub dev_dependencies: Vec<String>,
}

impl WorkspaceMember {
    /// Creates a new workspace member.
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self { name: name.into(), path: path.into(), version: "0.0.0".to_string(), is_root: false, dependencies: Vec::new(), dev_dependencies: Vec::new() }
    }

    /// Sets the version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Marks as root package.
    pub fn as_root(mut self) -> Self {
        self.is_root = true;
        self
    }

    /// Adds a dependency.
    pub fn add_dependency(&mut self, dep: impl Into<String>) {
        self.dependencies.push(dep.into());
    }

    /// Adds a dev dependency.
    pub fn add_dev_dependency(&mut self, dep: impl Into<String>) {
        self.dev_dependencies.push(dep.into());
    }
}

/// Cache entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached value.
    pub value: T,
    /// Time when the entry was created (as duration since UNIX epoch).
    pub created_at: Duration,
    /// Time when the entry was last accessed (as duration since UNIX epoch).
    pub last_accessed: Duration,
    /// Size of the entry in bytes (approximate).
    pub size: usize,
    /// Dependencies that this entry depends on.
    pub dependencies: Vec<PathBuf>,
    /// Hash of the dependencies for quick validation.
    pub dependency_hash: u64,
}

/// Workspace cache for performance optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCache {
    /// Cached workspace members.
    pub members: HashMap<String, CacheEntry<WorkspaceMember>>,
    /// Cached member order.
    pub member_order: CacheEntry<Vec<String>>,
    /// Cached workspace configuration.
    pub config: Option<CacheEntry<SharedConfig>>,
    /// LRU queue for cache eviction.
    pub lru_queue: VecDeque<String>,
    /// Current memory usage.
    pub memory_usage: usize,
    /// Maximum memory usage (in bytes).
    pub max_memory_usage: usize,
    /// Disk cache path.
    pub disk_cache_path: Option<PathBuf>,
    /// Last disk sync time (as duration since UNIX epoch).
    pub last_disk_sync: Option<Duration>,
}

/// Dependency cache for shared dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCache {
    /// Shared dependencies across workspace members.
    pub shared_dependencies: CacheEntry<HashMap<String, String>>,
    /// Shared dev dependencies across workspace members.
    pub shared_dev_dependencies: CacheEntry<HashMap<String, String>>,
    /// Dependency resolution cache.
    pub resolved_dependencies: HashMap<String, CacheEntry<HashMap<String, String>>>,
    /// LRU queue for cache eviction.
    pub lru_queue: VecDeque<String>,
    /// Current memory usage.
    pub memory_usage: usize,
    /// Maximum memory usage (in bytes).
    pub max_memory_usage: usize,
}

/// Workspace configuration and state.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Root directory of the workspace.
    pub root: PathBuf,
    /// All member packages.
    pub members: HashMap<String, WorkspaceMember>,
    /// Member discovery order (for builds).
    pub member_order: Vec<String>,
    /// Shared workspace configuration.
    pub shared_config: Option<SharedConfig>,
    /// Cache for performance optimization.
    pub cache: Option<WorkspaceCache>,
    /// Dependency cache for shared dependencies.
    pub dependency_cache: Option<DependencyCache>,
    /// Whether members are lazily loaded.
    pub lazy_loaded: bool,
    /// Pending members to load (for lazy loading).
    pub pending_members: Vec<PathBuf>,
    /// Cache statistics.
    pub cache_stats: CacheStats,
}

/// Cache statistics for monitoring and analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of cache evictions.
    pub evictions: u64,
    /// Number of cache invalidations.
    pub invalidations: u64,
    /// Current memory usage (bytes).
    pub memory_usage: usize,
    /// Peak memory usage (bytes).
    pub peak_memory_usage: usize,
    /// Average cache access time (nanoseconds).
    pub avg_access_time: u64,
    /// Total cache access time (nanoseconds).
    pub total_access_time: u64,
    /// Number of accesses.
    pub access_count: u64,
}

/// Shared configuration inherited by all members.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SharedConfig {
    /// Shared version.
    pub version: Option<String>,
    /// Shared authors.
    pub authors: Vec<String>,
    /// Shared license.
    pub license: Option<String>,
    /// Shared repository.
    pub repository: Option<String>,
    /// Shared dependencies.
    pub dependencies: HashMap<String, String>,
    /// Shared dev dependencies.
    pub dev_dependencies: HashMap<String, String>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self { root: PathBuf::from("."), members: HashMap::new(), member_order: Vec::new(), shared_config: None, cache: None, dependency_cache: None, lazy_loaded: false, pending_members: Vec::new(), cache_stats: CacheStats::default() }
    }
}

impl Workspace {
    /// Creates a new empty workspace.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), ..Default::default() }
    }

    /// Computes hash for a file's content.
    fn compute_file_hash(path: &Path) -> Result<u64> {
        let content = fs::read(path)?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(hasher.finish())
    }

    /// Creates a cache entry with metadata.
    fn create_cache_entry<T>(value: T, dependencies: Vec<PathBuf>) -> Result<CacheEntry<T>> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for dep in &dependencies {
            if let Ok(hash) = Self::compute_file_hash(dep) {
                hash.hash(&mut hasher);
            }
        }
        let dependency_hash = hasher.finish();

        // Approximate size calculation (simplified)
        let size = std::mem::size_of::<T>();

        let created_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

        Ok(CacheEntry { value, created_at, last_accessed: created_at, size, dependencies, dependency_hash })
    }

    /// Checks if a cache entry is valid.
    fn is_cache_valid<T>(entry: &CacheEntry<T>) -> bool {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for dep in &entry.dependencies {
            if let Ok(hash) = Self::compute_file_hash(dep) {
                hash.hash(&mut hasher);
            }
        }
        let current_hash = hasher.finish();
        current_hash == entry.dependency_hash
    }

    /// Discovers workspace from a root directory.
    pub fn discover(root: &Path) -> Result<Self> {
        let nargo_toml = root.join("Nargo.toml");

        if !nargo_toml.exists() {
            return Err(Error::external_error("workspace".to_string(), format!("No Nargo.toml found in {:?}", root), Span::unknown()));
        }

        // 简化实现，实际需要根据 nargo_config 的 API 进行调整
        let mut workspace = Self::new(root);
        let start_time = Instant::now();

        // 尝试从缓存加载
        if let Some(cache) = &workspace.cache {
            // 检查缓存是否有效
            if Self::is_cache_valid(&cache.member_order) {
                info!("Loading workspace from cache");
                // 从缓存中恢复数据
                workspace.members = cache.members.iter().map(|(k, v)| (k.clone(), v.value.clone())).collect();
                workspace.member_order = cache.member_order.value.clone();
                workspace.shared_config = cache.config.as_ref().map(|c| c.value.clone());

                // 更新缓存统计
                workspace.cache_stats.hits += 1;
                let access_time = start_time.elapsed().as_nanos() as u64;
                workspace.cache_stats.total_access_time += access_time;
                workspace.cache_stats.access_count += 1;
                workspace.cache_stats.avg_access_time = workspace.cache_stats.total_access_time / workspace.cache_stats.access_count;

                return Ok(workspace);
            }
            else {
                // 缓存无效，增加失效计数
                workspace.cache_stats.invalidations += 1;
            }
        }

        // 缓存未命中
        workspace.cache_stats.misses += 1;

        // 添加一个默认成员
        let member = WorkspaceMember::new("default", root.to_path_buf()).with_version("0.0.0").as_root();

        workspace.members.insert(member.name.clone(), member.clone());
        workspace.member_order.push(member.name);

        // 缓存结果
        let dependencies = vec![nargo_toml.clone()];
        let member_entries: HashMap<String, CacheEntry<WorkspaceMember>> = workspace
            .members
            .iter()
            .map(|(k, v)| {
                let entry = Self::create_cache_entry(v.clone(), dependencies.clone()).unwrap();
                (k.clone(), entry)
            })
            .collect();

        let member_order_entry = Self::create_cache_entry(workspace.member_order.clone(), dependencies.clone())?;
        let config_entry = workspace.shared_config.as_ref().map(|c| Self::create_cache_entry(c.clone(), dependencies.clone()).unwrap());

        // 计算内存使用
        let memory_usage = member_entries.values().map(|e| e.size).sum::<usize>() + member_order_entry.size + config_entry.as_ref().map(|e| e.size).unwrap_or(0);

        let last_disk_sync = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

        workspace.cache = Some(WorkspaceCache {
            members: member_entries,
            member_order: member_order_entry,
            config: config_entry,
            lru_queue: VecDeque::new(),
            memory_usage,
            max_memory_usage: 1024 * 1024 * 100, // 100MB
            disk_cache_path: Some(nargo_types::cache_dir(root)),
            last_disk_sync: Some(last_disk_sync),
        });

        // 更新缓存统计
        let access_time = start_time.elapsed().as_nanos() as u64;
        workspace.cache_stats.total_access_time += access_time;
        workspace.cache_stats.access_count += 1;
        workspace.cache_stats.avg_access_time = workspace.cache_stats.total_access_time / workspace.cache_stats.access_count;
        workspace.cache_stats.memory_usage = memory_usage;
        if memory_usage > workspace.cache_stats.peak_memory_usage {
            workspace.cache_stats.peak_memory_usage = memory_usage;
        }

        info!("Discovered workspace with {} members", workspace.members.len());
        Ok(workspace)
    }

    /// Discovers workspace or returns a single-package workspace.
    pub fn discover_or_single(root: &Path) -> Result<Self> {
        Self::discover(root).or_else(|_| {
            let nargo_toml = root.join("Nargo.toml");
            if nargo_toml.exists() {
                // 简化实现，实际需要根据 nargo_config 的 API 进行调整
                let member = WorkspaceMember::new("default", root.to_path_buf()).with_version("0.0.0").as_root();

                let mut workspace = Self::new(root);
                workspace.members.insert(member.name.clone(), member.clone());
                workspace.member_order.push(member.name);

                Ok(workspace)
            }
            else {
                Err(Error::external_error("workspace".to_string(), format!("No Nargo.toml found in {:?}", root), Span::unknown()))
            }
        })
    }

    fn discover_members(&mut self, root: &Path, pattern: &str) -> Result<()> {
        if pattern.ends_with("/*") {
            let base = root.join(pattern.trim_end_matches("/*"));
            if base.is_dir() {
                // 使用并行加载提高性能
                let entries = fs::read_dir(&base)?;
                let members = Arc::new(Mutex::new(Vec::new()));

                // 创建 tokio 运行时
                let rt = Runtime::new()?;

                rt.block_on(async {
                    let mut handles = Vec::new();

                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();

                            if path.is_dir() && path.join("Nargo.toml").exists() {
                                let members_clone = Arc::clone(&members);
                                let path_clone = path.clone();

                                let handle = tokio::spawn(async move {
                                    if let Ok(member) = Self::load_member_from_path(&path_clone) {
                                        let mut members = members_clone.lock().unwrap();
                                        members.push(member);
                                    }
                                });

                                handles.push(handle);
                            }
                        }
                    }

                    // 等待所有任务完成
                    for handle in handles {
                        let _ = handle.await;
                    }
                });

                // 添加加载的成员
                let loaded_members = members.lock().unwrap();
                for member in loaded_members.iter() {
                    self.members.insert(member.name.clone(), member.clone());
                }
            }
        }
        else {
            let path = root.join(pattern);
            if path.is_dir() && path.join("Nargo.toml").exists() {
                self.add_member_from_path(&path)?;
            }
        }
        Ok(())
    }

    fn add_member_from_path(&mut self, path: &Path) -> Result<()> {
        let member = Self::load_member_from_path(path)?;
        self.members.insert(member.name.clone(), member);
        Ok(())
    }

    /// Loads a workspace member from a path (for parallel loading).
    fn load_member_from_path(path: &Path) -> Result<WorkspaceMember> {
        let nargo_toml = path.join("Nargo.toml");
        let _content = fs::read_to_string(&nargo_toml).map_err(|e| Error::external_error("workspace".to_string(), format!("Failed to read {:?}: {}", nargo_toml, e), Span::unknown()))?;

        // 简化实现，实际需要根据 nargo_config 的 API 进行调整
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let member = WorkspaceMember::new(&name, path).with_version("0.0.0");

        Ok(member)
    }

    fn compute_dependency_order(&mut self) -> Result<()> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_marks = HashSet::new();

        fn visit(name: &str, members: &HashMap<String, WorkspaceMember>, visited: &mut HashSet<String>, temp_marks: &mut HashSet<String>, order: &mut Vec<String>) -> Result<()> {
            if visited.contains(name) {
                return Ok(());
            }
            if temp_marks.contains(name) {
                return Err(Error::external_error("workspace".to_string(), format!("Circular dependency detected involving {}", name), Span::unknown()));
            }

            temp_marks.insert(name.to_string());

            if let Some(member) = members.get(name) {
                for dep in &member.dependencies {
                    if members.contains_key(dep) {
                        visit(dep, members, visited, temp_marks, order)?;
                    }
                }
            }

            temp_marks.remove(name);
            visited.insert(name.to_string());
            order.push(name.to_string());

            Ok(())
        }

        for name in self.members.keys() {
            visit(name, &self.members, &mut visited, &mut temp_marks, &mut order)?;
        }

        self.member_order = order;
        Ok(())
    }

    /// Gets a member by name.
    pub fn get_member(&self, name: &str) -> Option<&WorkspaceMember> {
        self.members.get(name)
    }

    /// Gets a member by path.
    pub fn get_member_by_path(&self, path: &Path) -> Option<&WorkspaceMember> {
        self.members.values().find(|m| m.path == path)
    }

    /// Returns all member names in dependency order.
    pub fn member_names(&self) -> &[String] {
        &self.member_order
    }

    /// Returns the number of members.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Checks if the workspace is empty.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Checks if this is a single-package workspace.
    pub fn is_single(&self) -> bool {
        self.members.len() == 1
    }

    /// Returns the root member if this is a single-package workspace.
    pub fn root_member(&self) -> Option<&WorkspaceMember> {
        self.members.values().find(|m| m.is_root)
    }

    /// Lists all members.
    pub fn list_members(&self) {
        println!("Workspace members:");
        for name in &self.member_order {
            if let Some(member) = self.members.get(name) {
                let rel_path = member.path.strip_prefix(&self.root).unwrap_or(&member.path);
                println!("  {} @ {} ({})", name, member.version, rel_path.display());
            }
        }
    }

    /// Returns members that depend on a given package.
    pub fn dependents_of(&self, package_name: &str) -> Vec<&WorkspaceMember> {
        self.members.values().filter(|m| m.dependencies.contains(&package_name.to_string())).collect()
    }

    /// Returns members that a given package depends on.
    pub fn dependencies_of(&self, package_name: &str) -> Vec<&WorkspaceMember> {
        if let Some(member) = self.members.get(package_name) { member.dependencies.iter().filter_map(|dep| self.members.get(dep)).collect() } else { Vec::new() }
    }

    /// Filters members by a predicate.
    pub fn filter_members<F>(&self, predicate: F) -> Vec<&WorkspaceMember>
    where
        F: Fn(&WorkspaceMember) -> bool,
    {
        self.member_order
            .iter()
            .filter_map(|name| {
                let member = self.members.get(name)?;
                if predicate(member) { Some(member) } else { None }
            })
            .collect()
    }

    /// Returns the workspace root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the target directory for the workspace.
    pub fn target_dir(&self) -> PathBuf {
        self.root.join("target")
    }

    /// Returns the shared configuration.
    pub fn shared_config(&self) -> Option<&SharedConfig> {
        self.shared_config.as_ref()
    }

    /// Merges configuration from multiple sources.
    pub fn merge_config(&mut self, other: &SharedConfig) -> Result<()> {
        if let Some(ref mut config) = self.shared_config {
            // 合并版本（如果未设置）
            if config.version.is_none() && other.version.is_some() {
                config.version = other.version.clone();
            }

            // 合并作者
            config.authors.extend(other.authors.clone());
            // 去重
            let mut unique_authors = HashSet::new();
            config.authors.retain(|author| unique_authors.insert(author.clone()));

            // 合并许可证（如果未设置）
            if config.license.is_none() && other.license.is_some() {
                config.license = other.license.clone();
            }

            // 合并仓库（如果未设置）
            if config.repository.is_none() && other.repository.is_some() {
                config.repository = other.repository.clone();
            }

            // 合并依赖
            for (name, version) in &other.dependencies {
                if !config.dependencies.contains_key(name) {
                    config.dependencies.insert(name.clone(), version.clone());
                }
            }

            // 合并开发依赖
            for (name, version) in &other.dev_dependencies {
                if !config.dev_dependencies.contains_key(name) {
                    config.dev_dependencies.insert(name.clone(), version.clone());
                }
            }
        }
        else {
            // 如果当前没有配置，直接使用其他配置
            self.shared_config = Some(other.clone());
        }

        Ok(())
    }

    /// Loads configuration from a file with caching.
    pub fn load_config_from_file(&mut self, path: &Path) -> Result<()> {
        // 检查文件是否存在
        if !path.exists() {
            return Err(Error::external_error("workspace".to_string(), format!("Config file not found: {:?}", path), Span::unknown()));
        }

        // 读取配置文件
        let content = fs::read_to_string(path).map_err(|e| Error::external_error("workspace".to_string(), format!("Failed to read config file: {}", e), Span::unknown()))?;

        // 解析配置（简化实现）
        let config: SharedConfig = oak_toml::from_str(&content).map_err(|e| Error::external_error("workspace".to_string(), format!("Failed to parse config file: {}", e), Span::unknown()))?;

        // 合并配置
        self.merge_config(&config)?;

        Ok(())
    }

    /// Computes shared dependencies across all workspace members.
    pub fn compute_shared_dependencies(&mut self) -> Result<()> {
        let start_time = Instant::now();

        // 检查依赖缓存是否有效
        if let Some(cache) = &self.dependency_cache {
            // 简单的有效性检查，实际需要更复杂的依赖分析
            if !cache.resolved_dependencies.is_empty() {
                info!("Using cached shared dependencies");
                // 更新缓存统计
                self.cache_stats.hits += 1;
                let access_time = start_time.elapsed().as_nanos() as u64;
                self.cache_stats.total_access_time += access_time;
                self.cache_stats.access_count += 1;
                self.cache_stats.avg_access_time = self.cache_stats.total_access_time / self.cache_stats.access_count;
                return Ok(());
            }
        }

        // 缓存未命中
        self.cache_stats.misses += 1;

        let mut shared_deps = HashMap::new();
        let mut shared_dev_deps = HashMap::new();

        // 统计所有成员的依赖
        let mut member_with_deps_count = 0;
        for member in self.members.values() {
            // 统计依赖
            if !member.dependencies.is_empty() {
                member_with_deps_count += 1;
                for dep in &member.dependencies {
                    *shared_deps.entry(dep.to_string()).or_insert(0) += 1;
                }
            }

            // 统计开发依赖
            if !member.dev_dependencies.is_empty() {
                for dep in &member.dev_dependencies {
                    *shared_dev_deps.entry(dep.to_string()).or_insert(0) += 1;
                }
            }
        }

        // 只保留被大多数成员共享的依赖
        // 使用更合理的阈值：至少被 80% 的成员依赖
        let threshold = if member_with_deps_count > 0 { (member_with_deps_count as f64 * 0.8).ceil() as usize } else { 0 };

        let shared_deps: HashMap<String, String> = shared_deps
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(dep, _)| (dep, "latest".to_string())) // 简化实现，实际需要版本解析
            .collect();

        let shared_dev_deps: HashMap<String, String> = shared_dev_deps
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(dep, _)| (dep, "latest".to_string())) // 简化实现，实际需要版本解析
            .collect();

        // 缓存依赖解析结果
        let dependencies = self.members.values().map(|m| m.path.join("Nargo.toml")).collect::<Vec<_>>();
        let resolved_deps_entries: HashMap<String, CacheEntry<HashMap<String, String>>> = self
            .members
            .keys()
            .map(|member| {
                let entry = Self::create_cache_entry(shared_deps.clone(), dependencies.clone()).unwrap();
                (member.clone(), entry)
            })
            .collect();

        // 创建缓存条目
        let shared_deps_entry = Self::create_cache_entry(shared_deps, dependencies.clone())?;
        let shared_dev_deps_entry = Self::create_cache_entry(shared_dev_deps, dependencies.clone())?;

        // 计算内存使用
        let memory_usage = shared_deps_entry.size + shared_dev_deps_entry.size + resolved_deps_entries.values().map(|e| e.size).sum::<usize>();

        // 更新依赖缓存
        self.dependency_cache = Some(DependencyCache {
            shared_dependencies: shared_deps_entry,
            shared_dev_dependencies: shared_dev_deps_entry,
            resolved_dependencies: resolved_deps_entries,
            lru_queue: VecDeque::new(),
            memory_usage,
            max_memory_usage: 1024 * 1024 * 50, // 50MB
        });

        // 更新缓存统计
        let access_time = start_time.elapsed().as_nanos() as u64;
        self.cache_stats.total_access_time += access_time;
        self.cache_stats.access_count += 1;
        self.cache_stats.avg_access_time = self.cache_stats.total_access_time / self.cache_stats.access_count;
        self.cache_stats.memory_usage += memory_usage;
        if self.cache_stats.memory_usage > self.cache_stats.peak_memory_usage {
            self.cache_stats.peak_memory_usage = self.cache_stats.memory_usage;
        }

        info!("Computed shared dependencies: {} dependencies, {} dev dependencies", self.dependency_cache.as_ref().unwrap().shared_dependencies.value.len(), self.dependency_cache.as_ref().unwrap().shared_dev_dependencies.value.len());

        Ok(())
    }

    /// Gets shared dependencies for the workspace.
    pub fn shared_dependencies(&self) -> Option<&HashMap<String, String>> {
        self.dependency_cache.as_ref().map(|cache| &cache.shared_dependencies.value)
    }

    /// Gets shared dev dependencies for the workspace.
    pub fn shared_dev_dependencies(&self) -> Option<&HashMap<String, String>> {
        self.dependency_cache.as_ref().map(|cache| &cache.shared_dev_dependencies.value)
    }

    /// Resolves dependencies for a specific member (using cache if available).
    pub fn resolve_dependencies(&self, member_name: &str) -> Option<&HashMap<String, String>> {
        self.dependency_cache.as_ref()?.resolved_dependencies.get(member_name).map(|entry| &entry.value)
    }

    /// Enables lazy loading for workspace members.
    pub fn enable_lazy_loading(&mut self) {
        self.lazy_loaded = true;
        info!("Enabled lazy loading for workspace members");
    }

    /// Loads pending members in batches.
    pub fn load_pending_members(&mut self, batch_size: usize) -> Result<usize> {
        let mut loaded_count = 0;
        let mut batch = Vec::new();

        // 收集一批待加载的成员
        while !self.pending_members.is_empty() && batch.len() < batch_size {
            batch.push(self.pending_members.pop().unwrap());
        }

        // 并行加载这批成员
        if !batch.is_empty() {
            let members = Arc::new(Mutex::new(Vec::new()));

            let rt = Runtime::new()?;

            rt.block_on(async {
                let mut handles = Vec::new();

                for path in batch {
                    let members_clone = Arc::clone(&members);
                    let path_clone = path.clone();

                    let handle = tokio::spawn(async move {
                        if let Ok(member) = Self::load_member_from_path(&path_clone) {
                            let mut members = members_clone.lock().unwrap();
                            members.push(member);
                        }
                    });

                    handles.push(handle);
                }

                // 等待所有任务完成
                for handle in handles {
                    let _ = handle.await;
                }
            });

            // 添加加载的成员
            let loaded_members = members.lock().unwrap();
            for member in loaded_members.iter() {
                self.members.insert(member.name.clone(), member.clone());
                self.member_order.push(member.name.clone());
                loaded_count += 1;
            }

            info!("Loaded {} members in batch", loaded_count);
        }

        Ok(loaded_count)
    }

    /// Adds a member to the pending list for lazy loading.
    pub fn add_pending_member(&mut self, path: impl Into<PathBuf>) {
        self.pending_members.push(path.into());
    }

    /// Processes members in batches for efficient handling.
    pub fn process_members_in_batches<F>(&self, batch_size: usize, mut processor: F) -> Result<()>
    where
        F: FnMut(&[&WorkspaceMember]) -> Result<()>,
    {
        let members: Vec<&WorkspaceMember> = self.members.values().collect();

        for batch in members.chunks(batch_size) {
            processor(batch)?;
        }

        Ok(())
    }

    /// Gets members by pattern with lazy loading support.
    pub fn get_members_by_pattern(&mut self, pattern: &str) -> Result<Vec<&WorkspaceMember>> {
        // 如果启用了惰性加载，先加载匹配的成员
        if self.lazy_loaded {
            // 简化实现，实际需要根据 pattern 匹配
            while !self.pending_members.is_empty() {
                self.load_pending_members(10)?;
            }
        }

        // 过滤匹配的成员
        let matching_members: Vec<&WorkspaceMember> = self
            .members
            .values()
            .filter(|member| {
                // 简化实现，实际需要根据 pattern 匹配
                member.name.contains(pattern)
            })
            .collect();

        Ok(matching_members)
    }

    /// Cleans up the cache to stay within memory limits.
    pub fn cleanup_cache(&mut self) {
        // 清理工作区缓存
        if let Some(cache) = &mut self.cache {
            while cache.memory_usage > cache.max_memory_usage && !cache.lru_queue.is_empty() {
                if let Some(key) = cache.lru_queue.pop_front() {
                    if let Some(entry) = cache.members.remove(&key) {
                        cache.memory_usage -= entry.size;
                        self.cache_stats.evictions += 1;
                        debug!("Evicted cache entry: {}", key);
                    }
                }
            }
        }

        // 清理依赖缓存
        if let Some(cache) = &mut self.dependency_cache {
            while cache.memory_usage > cache.max_memory_usage && !cache.lru_queue.is_empty() {
                if let Some(key) = cache.lru_queue.pop_front() {
                    if let Some(entry) = cache.resolved_dependencies.remove(&key) {
                        cache.memory_usage -= entry.size;
                        self.cache_stats.evictions += 1;
                        debug!("Evicted dependency cache entry: {}", key);
                    }
                }
            }
        }

        // 更新缓存统计
        let total_memory = self.cache.as_ref().map(|c| c.memory_usage).unwrap_or(0) + self.dependency_cache.as_ref().map(|c| c.memory_usage).unwrap_or(0);
        self.cache_stats.memory_usage = total_memory;
    }

    /// Syncs cache to disk for persistence.
    pub fn sync_cache_to_disk(&mut self) -> Result<()> {
        if let Some(cache) = &mut self.cache {
            if let Some(disk_path) = &cache.disk_cache_path {
                // 创建缓存目录
                fs::create_dir_all(disk_path)?;

                // 更新最后同步时间
                let last_disk_sync = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
                cache.last_disk_sync = Some(last_disk_sync);

                // 保存缓存到磁盘
                let cache_file = disk_path.join("workspace_cache.json");
                let cache_json = oak_json::to_string(cache).map_err(|e| Error::external_error("oak_json".to_string(), e.to_string(), Span::default()))?;
                fs::write(cache_file, cache_json)?;

                info!("Synced workspace cache to disk");
            }
        }
        Ok(())
    }

    /// Loads cache from disk.
    pub fn load_cache_from_disk(&mut self) -> Result<()> {
        if let Some(cache) = &self.cache {
            if let Some(disk_path) = &cache.disk_cache_path {
                let cache_file = disk_path.join("workspace_cache.json");
                if cache_file.exists() {
                    let cache_json = fs::read_to_string(cache_file)?;
                    let loaded_cache: WorkspaceCache = oak_json::from_str(&cache_json).map_err(|e| Error::external_error("oak_json".to_string(), e.to_string(), Span::default()))?;
                    self.cache = Some(loaded_cache);
                    info!("Loaded workspace cache from disk");
                }
            }
        }
        Ok(())
    }

    /// Clears all cache.
    pub fn clear_cache(&mut self) {
        // 清理工作区缓存
        if let Some(cache) = &mut self.cache {
            self.cache_stats.evictions += cache.members.len() as u64;
            cache.members.clear();
            cache.lru_queue.clear();
            cache.memory_usage = 0;
        }

        // 清理依赖缓存
        if let Some(cache) = &mut self.dependency_cache {
            self.cache_stats.evictions += cache.resolved_dependencies.len() as u64;
            cache.resolved_dependencies.clear();
            cache.lru_queue.clear();
            cache.memory_usage = 0;
        }

        // 更新缓存统计
        self.cache_stats.memory_usage = 0;
        info!("Cleared all cache");
    }

    /// Gets cache statistics.
    pub fn get_cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    /// Prints cache statistics.
    pub fn print_cache_stats(&self) {
        let stats = &self.cache_stats;
        println!("Cache Statistics:");
        println!("  Hits: {}", stats.hits);
        println!("  Misses: {}", stats.misses);
        println!("  Evictions: {}", stats.evictions);
        println!("  Invalidations: {}", stats.invalidations);
        println!("  Memory Usage: {} bytes", stats.memory_usage);
        println!("  Peak Memory Usage: {} bytes", stats.peak_memory_usage);
        println!("  Average Access Time: {} ns", stats.avg_access_time);
        println!("  Total Accesses: {}", stats.access_count);
        println!("  Hit Rate: {:.2}%", if stats.access_count > 0 { (stats.hits as f64 / stats.access_count as f64) * 100.0 } else { 0.0 });
    }
}
