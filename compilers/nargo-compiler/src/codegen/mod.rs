use nargo_ir::IRModule;
use nargo_types::Result;

pub trait Backend {
    type Output;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output>;
}

// 其他后端已迁移到 nargo-bundler
