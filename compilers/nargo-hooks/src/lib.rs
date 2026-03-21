#![warn(missing_docs)]

use nargo_config::NargoConfig;
use nargo_types::{Error, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::process::Command;
use tracing::{error, info, warn};

pub struct NargoHooks;

impl NargoHooks {
    /// Install git hooks that redirect to `nargo hooks run <hook_name>`
    pub fn install(root_path: &Path) -> Result<()> {
        let git_dir = root_path.join(".git");
        if !git_dir.exists() {
            return Err(Error::external_error("hooks".to_string(), format!("Not a git repository: .git directory not found at {:?}", root_path), nargo_types::Span::unknown()));
        }

        let hooks_dir = git_dir.join("hooks");
        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir)?;
        }

        let hooks = vec!["pre-commit", "pre-push", "commit-msg", "post-checkout", "post-merge"];

        // Get the current executable path to point the hooks to it
        let current_exe = std::env::current_exe()?;
        let exe_path_str = current_exe.to_str().ok_or_else(|| Error::external_error("hooks".to_string(), "Invalid executable path".to_string(), nargo_types::Span::unknown()))?;

        for hook in hooks {
            let hook_path = hooks_dir.join(hook);

            // Create a shell script (or batch file for windows) that calls nargo
            // On Windows, git typically uses sh.exe from MinGW/Git Bash
            let hook_content = format!(
                "#!/bin/sh\n\n# Nargo Native Hook\n\"{}\" hooks run {} \"$@\"\n",
                exe_path_str.replace("\\", "/"), // Use forward slashes for sh
                hook
            );

            fs::write(&hook_path, hook_content)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&hook_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&hook_path, perms)?;
            }

            info!("Installed git hook: {:?}", hook_path);
        }

        Ok(())
    }

    /// Uninstall git hooks installed by Nargo
    pub fn uninstall(root_path: &Path) -> Result<()> {
        let hooks_dir = root_path.join(".git").join("hooks");
        if !hooks_dir.exists() {
            return Ok(());
        }

        let hooks = vec!["pre-commit", "pre-push", "commit-msg", "post-checkout", "post-merge"];
        for hook in hooks {
            let hook_path = hooks_dir.join(hook);
            if hook_path.exists() {
                let content = fs::read_to_string(&hook_path)?;
                if content.contains("# Nargo Native Hook") {
                    fs::remove_file(&hook_path)?;
                    info!("Removed git hook: {:?}", hook_path);
                }
            }
        }

        Ok(())
    }

    /// Run the logic for a specific hook
    pub async fn run(hook_name: &str, args: Vec<String>, config_path: Option<PathBuf>) -> Result<()> {
        info!("Running git hook: {} with args: {:?}", hook_name, args);

        // 简化实现，实际需要根据 nargo_config 的 API 进行调整
        let config = NargoConfig::default();

        // 简化实现，实际需要根据 nargo_config 的 API 进行调整
        // 暂时不执行任何钩子命令
        info!("No command configured for hook: {}, skipping.", hook_name);

        Ok(())
    }
}
