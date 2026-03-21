use nargo_workspace::*;
use std::{fs, path::Path};
use tempfile::tempdir;

#[test]
fn test_workspace_loading_performance() {
    // 创建临时目录作为工作区
    let temp_dir = tempdir().unwrap();
    let workspace_dir = temp_dir.path();

    // 创建 Nargo.toml 文件
    fs::write(
        workspace_dir.join("Nargo.toml"),
        r#"
[package]
name = "test-workspace"
version = "0.1.0"

[workspace]
members = ["packages/*"]
"#,
    )
    .unwrap();

    // 创建多个包目录
    let packages_dir = workspace_dir.join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // 创建 10 个测试包
    for i in 0..10 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("Nargo.toml"),
            format!(
                r#"
[package]
name = "pkg-{}"
version = "0.1.0"

dependencies = [
    "pkg-{}",
]
"#,
                i,
                (i + 9) % 10
            ),
        )
        .unwrap();
    }

    // 测试工作区发现性能
    let start = std::time::Instant::now();
    let workspace = Workspace::discover(workspace_dir).unwrap();
    let duration = start.elapsed();

    println!("Workspace discovery time: {:?}", duration);
    assert!(duration < std::time::Duration::from_secs(1), "Workspace discovery should be fast");
    assert_eq!(workspace.members.len(), 1, "Should discover at least one member");
}

#[test]
fn test_dependency_sharing() {
    // 创建临时目录作为工作区
    let temp_dir = tempdir().unwrap();
    let workspace_dir = temp_dir.path();

    // 创建 Nargo.toml 文件
    fs::write(
        workspace_dir.join("Nargo.toml"),
        r#"
[package]
name = "test-workspace"
version = "0.1.0"

[workspace]
members = ["packages/*"]
"#,
    )
    .unwrap();

    // 创建多个包目录
    let packages_dir = workspace_dir.join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // 创建 5 个测试包，都依赖于同一个包
    for i in 0..5 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("Nargo.toml"),
            format!(
                r#"
[package]
name = "pkg-{}"
version = "0.1.0"

dependencies = [
    "shared-dep",
]
"#,
                i
            ),
        )
        .unwrap();
    }

    // 创建共享依赖包
    let shared_dep_dir = packages_dir.join("shared-dep");
    fs::create_dir_all(&shared_dep_dir).unwrap();
    fs::write(
        shared_dep_dir.join("Nargo.toml"),
        r#"
[package]
name = "shared-dep"
version = "0.1.0"
"#,
    )
    .unwrap();

    // 测试依赖共享
    let mut workspace = Workspace::new(workspace_dir);

    // 手动添加所有包到工作区
    for i in 0..5 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        let mut member = WorkspaceMember::new(format!("pkg-{}", i), pkg_dir);
        member.add_dependency("shared-dep");
        workspace.members.insert(member.name.clone(), member);
    }

    let shared_dep_dir = packages_dir.join("shared-dep");
    let shared_dep_member = WorkspaceMember::new("shared-dep", shared_dep_dir);
    workspace.members.insert(shared_dep_member.name.clone(), shared_dep_member);

    workspace.compute_shared_dependencies().unwrap();

    let shared_deps = workspace.shared_dependencies().unwrap();
    println!("Shared dependencies: {:?}", shared_deps);
    assert!(shared_deps.contains_key("shared-dep"), "Should share common dependency");
}

#[test]
fn test_lazy_loading() {
    // 创建临时目录作为工作区
    let temp_dir = tempdir().unwrap();
    let workspace_dir = temp_dir.path();

    // 创建 Nargo.toml 文件
    fs::write(
        workspace_dir.join("Nargo.toml"),
        r#"
[package]
name = "test-workspace"
version = "0.1.0"

[workspace]
members = ["packages/*"]
"#,
    )
    .unwrap();

    // 创建多个包目录
    let packages_dir = workspace_dir.join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // 创建 20 个测试包
    for i in 0..20 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("Nargo.toml"),
            format!(
                r#"
[package]
name = "pkg-{}"
version = "0.1.0"
"#,
                i
            ),
        )
        .unwrap();
    }

    // 测试惰性加载
    let mut workspace = Workspace::new(workspace_dir);
    workspace.enable_lazy_loading();

    // 添加所有包到待加载列表
    for i in 0..20 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        workspace.add_pending_member(pkg_dir);
    }

    // 分批加载
    let start = std::time::Instant::now();
    let mut total_loaded = 0;
    while total_loaded < 20 {
        let loaded = workspace.load_pending_members(5).unwrap();
        total_loaded += loaded;
        println!("Loaded {} members, total: {}", loaded, total_loaded);
    }
    let duration = start.elapsed();

    println!("Lazy loading time: {:?}", duration);
    assert!(duration < std::time::Duration::from_secs(2), "Lazy loading should be fast");
    assert_eq!(workspace.members.len(), 20, "Should load all members");
}

#[test]
fn test_batch_processing() {
    // 创建临时目录作为工作区
    let temp_dir = tempdir().unwrap();
    let workspace_dir = temp_dir.path();

    // 创建 Nargo.toml 文件
    fs::write(
        workspace_dir.join("Nargo.toml"),
        r#"
[package]
name = "test-workspace"
version = "0.1.0"

[workspace]
members = ["packages/*"]
"#,
    )
    .unwrap();

    // 创建多个包目录
    let packages_dir = workspace_dir.join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // 创建 15 个测试包
    for i in 0..15 {
        let pkg_dir = packages_dir.join(format!("pkg-{}", i));
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("Nargo.toml"),
            format!(
                r#"
[package]
name = "pkg-{}"
version = "0.1.0"
"#,
                i
            ),
        )
        .unwrap();
    }

    // 测试批量处理
    let mut workspace = Workspace::discover(workspace_dir).unwrap();

    let start = std::time::Instant::now();
    let mut processed_count = 0;

    workspace
        .process_members_in_batches(5, |batch| {
            println!("Processing batch of {} members", batch.len());
            processed_count += batch.len();
            Ok(())
        })
        .unwrap();

    let duration = start.elapsed();
    println!("Batch processing time: {:?}", duration);
    assert_eq!(processed_count, workspace.members.len(), "Should process all members");
}
