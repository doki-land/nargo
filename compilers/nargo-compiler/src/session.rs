use chrono::{DateTime, Local};
use nargo_types::{errors::Result, IRModule};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDiagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct CompilerSession {
    pub id: String,
    pub root: PathBuf,
    pub started_at: DateTime<Local>,
    pub last_updated_at: DateTime<Local>,

    pub modules: HashMap<PathBuf, IRModule>,
    pub diagnostics: Vec<SessionDiagnostic>,

    pub dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl CompilerSession {
    pub fn new(id: String, root: PathBuf) -> Self {
        let now = Local::now();
        Self {
            id,
            root,
            started_at: now,
            last_updated_at: now,
            modules: HashMap::new(),
            diagnostics: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn update_module(&mut self, path: PathBuf, module: IRModule) {
        self.modules.insert(path, module);
        self.last_updated_at = Local::now();
    }

    pub fn invalidate(&mut self, path: &PathBuf) {
        self.modules.remove(path);

        if let Some(dependents) = self.dependencies.get(path).cloned() {
            for dependent in dependents {
                self.invalidate(&dependent);
            }
        }

        self.last_updated_at = Local::now();
    }

    pub fn add_dependency(&mut self, file: PathBuf, dependent: PathBuf) {
        self.dependencies.entry(file).or_default().insert(dependent);
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn add_diagnostic(&mut self, diag: SessionDiagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn init_rustc_session(&mut self) -> Result<()> {
        tracing::info!("Initializing rustc_interface session for {}", self.id);
        Ok(())
    }
}
