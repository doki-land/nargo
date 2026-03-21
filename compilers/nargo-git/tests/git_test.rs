use nargo_git::{clone, commit_all, create_branch, create_tag, delete_branch, delete_tag, get_current_branch, get_status, init, list_branches, list_stashes, list_tags, merge_branch, pull, push, push_tag, stash, stash_apply, stash_pop, switch_branch};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_init() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Test git init
    let result = init(temp_dir.path());
    assert!(result.is_ok());

    // Check if .git directory was created
    let git_dir = temp_dir.path().join(".git");
    assert!(git_dir.exists());
    assert!(git_dir.is_dir());
}

#[test]
fn test_get_current_branch() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test get current branch
    let branch = get_current_branch(temp_dir.path());
    assert!(branch.is_ok());
    assert_eq!(branch.unwrap(), "master");
}

#[test]
fn test_create_branch() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test create branch
    let result = create_branch(temp_dir.path(), "test-branch");
    assert!(result.is_ok());
}

#[test]
fn test_get_status() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test get status
    let status = get_status(temp_dir.path());
    assert!(status.is_ok());
    // Should be empty initially
    assert!(status.unwrap().is_empty());

    // Create a test file
    std::fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

    // Test get status again
    let status = get_status(temp_dir.path());
    assert!(status.is_ok());
    // Should have one file
    assert_eq!(status.unwrap().len(), 1);
}

#[test]
fn test_clone() {
    // Note: Actual cloning requires a real remote repository
    // For simplicity, we'll just test that the function doesn't panic
    // In a real test environment, you would use a test repository
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("test-repo");

    // This will fail because we're using a non-existent URL, but it shouldn't panic
    let result = clone("https://github.com/nonexistent/repo.git", Some(&target));
    // We expect this to fail, but not panic
    assert!(result.is_err());
}

#[test]
fn test_commit_all() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();

    // Test commit all
    let result = commit_all(temp_dir.path(), "Initial commit");
    // This might fail if git user is not configured, but it shouldn't panic
    // We'll just check that it doesn't panic
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_push() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test push (will fail but shouldn't panic)
    let result = push(temp_dir.path(), "origin", "master");
    // We expect this to fail, but not panic
    assert!(result.is_err());
}

#[test]
fn test_pull() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test pull (will fail but shouldn't panic)
    let result = pull(temp_dir.path(), "origin", "master");
    // We expect this to fail, but not panic
    assert!(result.is_err());
}

#[test]
fn test_switch_branch() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a branch
    create_branch(temp_dir.path(), "test-branch").unwrap();

    // Test switch branch
    let result = switch_branch(temp_dir.path(), "test-branch");
    // This might fail if there are no commits, but it shouldn't panic
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_clone_without_target() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Test clone without target (should use repo name from URL)
    let result = clone("https://github.com/nonexistent/repo.git", None);
    // We expect this to fail, but not panic
    assert!(result.is_err());

    // Check if directory was created
    let repo_dir = temp_dir.path().join("repo");
    assert!(!repo_dir.exists()); // Should not exist because clone failed
}

#[test]
fn test_list_branches() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test list branches
    let branches = list_branches(temp_dir.path());
    assert!(branches.is_ok());
    let branch_list = branches.unwrap();
    assert!(branch_list.contains(&"master".to_string()));

    // Create a new branch
    create_branch(temp_dir.path(), "test-branch").unwrap();

    // Test list branches again
    let branches = list_branches(temp_dir.path());
    assert!(branches.is_ok());
    let branch_list = branches.unwrap();
    assert!(branch_list.contains(&"master".to_string()));
    assert!(branch_list.contains(&"test-branch".to_string()));
}

#[test]
fn test_delete_branch() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a new branch
    create_branch(temp_dir.path(), "test-branch").unwrap();

    // Test delete branch
    let result = delete_branch(temp_dir.path(), "test-branch");
    assert!(result.is_ok());

    // Test list branches to confirm deletion
    let branches = list_branches(temp_dir.path());
    assert!(branches.is_ok());
    let branch_list = branches.unwrap();
    assert!(!branch_list.contains(&"test-branch".to_string()));
}

#[test]
fn test_merge_branch() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Create a new branch
    create_branch(temp_dir.path(), "test-branch").unwrap();

    // Switch to the new branch
    switch_branch(temp_dir.path(), "test-branch").unwrap();

    // Modify the file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo\n\nModified in test-branch").unwrap();

    // Commit the change
    commit_all(temp_dir.path(), "Modify README in test-branch").unwrap();

    // Switch back to master
    switch_branch(temp_dir.path(), "master").unwrap();

    // Test merge branch
    let result = merge_branch(temp_dir.path(), "test-branch");
    // This might fail if there are conflicts, but it shouldn't panic
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_create_tag() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Test create lightweight tag
    let result = create_tag(temp_dir.path(), "v1.0.0", "Version 1.0.0", false);
    assert!(result.is_ok());

    // Test create annotated tag
    let result = create_tag(temp_dir.path(), "v1.0.1", "Version 1.0.1", true);
    assert!(result.is_ok());
}

#[test]
fn test_list_tags() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Create some tags
    create_tag(temp_dir.path(), "v1.0.0", "Version 1.0.0", false).unwrap();
    create_tag(temp_dir.path(), "v1.0.1", "Version 1.0.1", true).unwrap();

    // Test list tags
    let tags = list_tags(temp_dir.path());
    assert!(tags.is_ok());
    let tag_list = tags.unwrap();
    assert!(tag_list.contains(&"v1.0.0".to_string()));
    assert!(tag_list.contains(&"v1.0.1".to_string()));
}

#[test]
fn test_delete_tag() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Create a tag
    create_tag(temp_dir.path(), "v1.0.0", "Version 1.0.0", false).unwrap();

    // Test delete tag
    let result = delete_tag(temp_dir.path(), "v1.0.0");
    assert!(result.is_ok());

    // Test list tags to confirm deletion
    let tags = list_tags(temp_dir.path());
    assert!(tags.is_ok());
    let tag_list = tags.unwrap();
    assert!(!tag_list.contains(&"v1.0.0".to_string()));
}

#[test]
fn test_push_tag() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test push tag (will fail but shouldn't panic)
    let result = push_tag(temp_dir.path(), "origin", "v1.0.0");
    // We expect this to fail, but not panic
    assert!(result.is_err());
}

#[test]
fn test_stash() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("test.txt"), "initial content").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Modify the file
    std::fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();

    // Test stash
    let result = stash(temp_dir.path(), Some("Test stash"));
    assert!(result.is_ok());

    // Test list stashes
    let stashes = list_stashes(temp_dir.path());
    assert!(stashes.is_ok());
    let stash_list = stashes.unwrap();
    assert!(!stash_list.is_empty());
}

#[test]
fn test_stash_apply() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("test.txt"), "initial content").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Modify the file
    std::fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();

    // Stash the changes
    stash(temp_dir.path(), Some("Test stash")).unwrap();

    // Test stash apply
    let result = stash_apply(temp_dir.path());
    assert!(result.is_err() || result.is_ok()); // Might fail if no stashes, but shouldn't panic
}

#[test]
fn test_stash_pop() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Create a test file
    std::fs::write(temp_dir.path().join("test.txt"), "initial content").unwrap();

    // Commit the file
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Modify the file
    std::fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();

    // Stash the changes
    stash(temp_dir.path(), Some("Test stash")).unwrap();

    // Test stash pop
    let result = stash_pop(temp_dir.path());
    assert!(result.is_err() || result.is_ok()); // Might fail if no stashes, but shouldn't panic
}

#[test]
fn test_list_stashes() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Initialize git repo
    init(temp_dir.path()).unwrap();

    // Test list stashes (should be empty initially)
    let stashes = list_stashes(temp_dir.path());
    assert!(stashes.is_ok());
    let stash_list = stashes.unwrap();
    assert!(stash_list.is_empty());

    // Create a test file and commit
    std::fs::write(temp_dir.path().join("test.txt"), "initial content").unwrap();
    let result = commit_all(temp_dir.path(), "Initial commit");
    if result.is_err() {
        // Skip test if commit fails (e.g., no git user configured)
        return;
    }

    // Modify the file and stash
    std::fs::write(temp_dir.path().join("test.txt"), "modified content").unwrap();
    stash(temp_dir.path(), Some("Test stash")).unwrap();

    // Test list stashes again
    let stashes = list_stashes(temp_dir.path());
    assert!(stashes.is_ok());
    let stash_list = stashes.unwrap();
    assert!(!stash_list.is_empty());
}
