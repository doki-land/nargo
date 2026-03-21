use nargo_hooks::NargoHooks;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

#[tokio::test]
async fn test_hooks_install_and_uninstall() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create .git directory to simulate a git repo
    fs::create_dir(root.join(".git")).unwrap();

    // Install hooks
    NargoHooks::install(root).unwrap();

    let hooks_dir = root.join(".git").join("hooks");
    assert!(hooks_dir.join("pre-commit").exists());
    assert!(hooks_dir.join("commit-msg").exists());

    let content = fs::read_to_string(hooks_dir.join("pre-commit")).unwrap();
    assert!(content.contains("# Nargo Native Hook"));

    // Uninstall hooks
    NargoHooks::uninstall(root).unwrap();
    assert!(!hooks_dir.join("pre-commit").exists());
    assert!(!hooks_dir.join("commit-msg").exists());
}

#[tokio::test]
async fn test_hooks_run_with_config() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a dummy config file
    let config_path = root.join("nargo.config.json");
    let config_content = r#"{
        "root": ".",
        "out_dir": "dist",
        "dev_server": {"port": 3000, "host": "127.0.0.1"},
        "test": {"coverage": false, "include": []},
        "plugins": [],
        "tasks": {},
        "hooks": {
            "pre-commit": "cargo --version"
        }
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the hook
    let result = NargoHooks::run("pre-commit", vec![], Some(config_path)).await;
    assert!(result.is_ok());
}
