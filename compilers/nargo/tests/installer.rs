use nargo::installer::*;

#[test]
fn test_install_options_default() {
    let options = InstallOptions::default();
    assert!(options.include_dev);
    assert!(options.include_optional);
    assert!(!options.force);
    assert!(!options.frozen);
    assert!(!options.offline);
    assert!(options.parallel_downloads > 0);
}

#[test]
fn test_install_result_default() {
    let result = InstallResult::default();
    assert_eq!(result.installed, 0);
    assert_eq!(result.removed, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.cached, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.packages.is_empty());
    assert!(result.removed_packages.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_install_diff() {
    let mut diff = InstallDiff::default();
    assert!(diff.is_empty());
    assert_eq!(diff.total_changes(), 0);

    diff.to_add.push(("lodash".to_string(), "4.17.21".to_string()));
    assert!(!diff.is_empty());
    assert_eq!(diff.total_changes(), 1);

    diff.to_remove.push(("old-pkg".to_string(), "1.0.0".to_string()));
    assert_eq!(diff.total_changes(), 2);
}

#[test]
fn test_install_progress() {
    let progress = InstallProgress { phase: InstallPhase::Downloading, current_package: Some("lodash".to_string()), total_packages: 10, completed_packages: 5, download_progress: 50 };
    assert_eq!(progress.phase, InstallPhase::Downloading);
    assert_eq!(progress.current_package, Some("lodash".to_string()));
    assert_eq!(progress.total_packages, 10);
    assert_eq!(progress.completed_packages, 5);
    assert_eq!(progress.download_progress, 50);
}
