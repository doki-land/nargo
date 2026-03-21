use nargo_workspace::{Workspace, WorkspaceMember};
use std::{fs, path::Path, time::Instant};

#[test]
fn test_cache_basic() {
    // 创建临时目录用于测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 创建 Nargo.toml 文件
    let nargo_toml = temp_path.join("Nargo.toml");
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.0.0\"\n").unwrap();

    // 第一次发现工作区（应该创建缓存）
    let start = Instant::now();
    let mut workspace = Workspace::discover(temp_path).unwrap();
    let first_time = start.elapsed();

    // 验证缓存统计
    assert_eq!(workspace.cache_stats.hits, 0);
    assert_eq!(workspace.cache_stats.misses, 1);

    // 第二次发现工作区（应该使用缓存）
    let start = Instant::now();
    let workspace2 = Workspace::discover(temp_path).unwrap();
    let second_time = start.elapsed();

    // 验证缓存统计
    assert_eq!(workspace2.cache_stats.hits, 1);
    assert_eq!(workspace2.cache_stats.misses, 1);

    // 验证第二次执行更快
    assert!(second_time < first_time);

    println!("First time: {:?}, Second time: {:?}", first_time, second_time);
    workspace2.print_cache_stats();
}

#[test]
fn test_cache_invalidation() {
    // 创建临时目录用于测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 创建 Nargo.toml 文件
    let nargo_toml = temp_path.join("Nargo.toml");
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.0.0\"\n").unwrap();

    // 第一次发现工作区
    let mut workspace = Workspace::discover(temp_path).unwrap();
    let initial_misses = workspace.cache_stats.misses;

    // 修改 Nargo.toml 文件
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();

    // 再次发现工作区（应该失效并重新创建缓存）
    let workspace2 = Workspace::discover(temp_path).unwrap();

    // 验证缓存失效
    assert_eq!(workspace2.cache_stats.misses, initial_misses + 1);
    assert_eq!(workspace2.cache_stats.invalidations, 1);

    println!("Cache invalidation test passed");
    workspace2.print_cache_stats();
}

#[test]
fn test_cache_cleanup() {
    // 创建临时目录用于测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 创建 Nargo.toml 文件
    let nargo_toml = temp_path.join("Nargo.toml");
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.0.0\"\n").unwrap();

    // 发现工作区
    let mut workspace = Workspace::discover(temp_path).unwrap();

    // 手动设置较小的内存限制以触发清理
    if let Some(cache) = &mut workspace.cache {
        cache.max_memory_usage = 1; // 非常小的限制，强制清理
    }

    // 执行清理
    workspace.cleanup_cache();

    // 验证清理效果
    assert!(workspace.cache_stats.evictions > 0);

    println!("Cache cleanup test passed");
    workspace.print_cache_stats();
}

#[test]
fn test_dependency_cache() {
    // 创建临时目录用于测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 创建 Nargo.toml 文件
    let nargo_toml = temp_path.join("Nargo.toml");
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.0.0\"\n").unwrap();

    // 发现工作区
    let mut workspace = Workspace::discover(temp_path).unwrap();

    // 添加一些依赖
    if let Some(member) = workspace.members.get_mut("default") {
        member.add_dependency("dep1");
        member.add_dependency("dep2");
        member.add_dev_dependency("dev1");
    }

    // 第一次计算共享依赖
    let start = Instant::now();
    workspace.compute_shared_dependencies().unwrap();
    let first_time = start.elapsed();

    // 验证缓存统计
    assert_eq!(workspace.cache_stats.hits, 0);
    assert_eq!(workspace.cache_stats.misses, 2); // 1 for workspace, 1 for dependencies

    // 第二次计算共享依赖（应该使用缓存）
    let start = Instant::now();
    workspace.compute_shared_dependencies().unwrap();
    let second_time = start.elapsed();

    // 验证缓存统计
    assert_eq!(workspace.cache_stats.hits, 1);
    assert_eq!(workspace.cache_stats.misses, 2);

    // 验证第二次执行更快
    assert!(second_time < first_time);

    println!("Dependency cache test passed");
    workspace.print_cache_stats();
}

#[test]
fn test_cache_persistence() {
    // 创建临时目录用于测试
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 创建 Nargo.toml 文件
    let nargo_toml = temp_path.join("Nargo.toml");
    fs::write(&nargo_toml, "[package]\nname = \"test\"\nversion = \"0.0.0\"\n").unwrap();

    // 发现工作区并同步到磁盘
    let mut workspace = Workspace::discover(temp_path).unwrap();
    workspace.sync_cache_to_disk().unwrap();

    // 清除内存缓存
    workspace.clear_cache();

    // 从磁盘加载缓存
    workspace.load_cache_from_disk().unwrap();

    // 验证缓存是否加载成功
    assert!(workspace.cache.is_some());

    println!("Cache persistence test passed");
    workspace.print_cache_stats();
}
