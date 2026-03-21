use crate::{CaaS, CompilerSession, NargoCompiler};
use nargo_types::{errors::Result, NargoContext};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct CompilerOptions {
    pub incremental: bool,
    pub threads: usize,
}

pub struct CompilerDriver {
    pub options: CompilerOptions,
    pub sessions: Arc<RwLock<HashMap<PathBuf, Arc<CompilerSession>>>>,
    pub compiler: Arc<NargoCompiler>,
}

impl CompilerDriver {
    pub fn new(ctx: Arc<NargoContext>, options: CompilerOptions) -> Self {
        Self {
            options,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            compiler: Arc::new(NargoCompiler::new(ctx)),
        }
    }

    pub async fn get_or_create_session(&self, root: &Path) -> Result<Arc<CompilerSession>> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get(root) {
            return Ok(session.clone());
        }

        let session_id = format!("session-{}", sessions.len());
        let mut session = CompilerSession::new(session_id, root.to_path_buf());

        // Initialize rustc interface integration
        session.init_rustc_session()?;

        let session_arc = Arc::new(session);
        sessions.insert(root.to_path_buf(), session_arc.clone());

        Ok(session_arc)
    }
}

#[async_trait::async_trait]
pub trait AsyncCaaS {
    async fn compile(&self, root: PathBuf) -> Result<()>;
    async fn watch(&self, root: PathBuf) -> Result<()>;
}

#[async_trait::async_trait]
impl AsyncCaaS for CompilerDriver {
    async fn compile(&self, root: PathBuf) -> Result<()> {
        info!("🚀 [CaaS] Starting incremental compilation for: {:?}", root);

        let _session = self.get_or_create_session(&root).await?;

        // 1. Discover files
        // 2. Check for changes
        // 3. Compile affected modules
        // 4. Update session cache

        info!("✨ [CaaS] Compilation finished for {:?}", root);
        Ok(())
    }

    async fn watch(&self, root: PathBuf) -> Result<()> {
        info!("👀 [CaaS] Watching for changes in: {:?}", root);
        // Implement watcher logic here, similar to nargo-mono but for CaaS
        Ok(())
    }
}

// Keep the synchronous trait for compatibility if needed, but prefer AsyncCaaS
impl CaaS for CompilerDriver {
    fn compile(&mut self, _root: PathBuf) -> Result<()> {
        Err(nargo_types::errors::Error::external_error(
            "CompilerDriver".to_string(),
            "Use AsyncCaaS::compile instead".to_string(),
            nargo_types::Span::unknown()
        ))
    }

    fn watch(&mut self, _root: PathBuf) -> Result<()> {
        Err(nargo_types::errors::Error::external_error(
            "CompilerDriver".to_string(),
            "Use AsyncCaaS::watch instead".to_string(),
            nargo_types::Span::unknown()
        ))
    }
}
