use nargo_release::ReleaseManager;
use semver::Version;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_topology_sort() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a mock workspace
    let cargo_toml = r#"
[workspace]
members = ["a", "b", "c"]
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create members
    let create_member = |name: &str, deps: Vec<&str>| {
        let member_dir = root.join(name);
        fs::create_dir(&member_dir).unwrap();
        let deps_str = deps.iter().map(|d| format!("{} = \"*\"", d)).collect::<Vec<_>>().join("\n");
        let member_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"

[dependencies]
{}
"#,
            name, deps_str
        );
        fs::write(member_dir.join("Cargo.toml"), member_toml).unwrap();
    };

    create_member("a", vec!["b", "c"]);
    create_member("b", vec!["c"]);
    create_member("c", vec![]);

    let manager = ReleaseManager::new(root).unwrap();
    let plan = manager.build_release_plan().unwrap();

    // Expected order: c, b, a
    assert_eq!(plan, vec!["c", "b", "a"]);
}

#[test]
fn test_bump_version() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let cargo_toml = r#"
[workspace]
members = ["pkg-a"]
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();

    let member_dir = root.join("pkg-a");
    fs::create_dir(&member_dir).unwrap();
    let member_toml = r#"
[package]
name = "pkg-a"
version = "0.1.0"
"#;
    fs::write(member_dir.join("Cargo.toml"), member_toml).unwrap();

    let mut manager = ReleaseManager::new(root).unwrap();
    let new_version = Version::parse("0.2.0").unwrap();
    manager.bump_version("pkg-a", new_version.clone()).unwrap();

    // Check if file was updated
    let updated_toml = fs::read_to_string(member_dir.join("Cargo.toml")).unwrap();
    assert!(updated_toml.contains("version = \"0.2.0\""));
}
