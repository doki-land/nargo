use nargo_transformer::{
    Transformer,
    passes::{constant_folding::ConstantFoldingPass, dead_code_elimination::DeadCodeEliminationPass},
};

#[test]
fn test_transformer_initialization() {
    let transformer = Transformer::new();
    assert!(!transformer.enable_snapshots);
}

#[test]
fn test_constant_folding() {
    let pass = ConstantFoldingPass;
    // 这里可以添加具体的测试逻辑
    // 由于需要完整的IR结构，暂时只测试初始化
    assert_eq!(pass.name(), "ConstantFolding");
}

#[test]
fn test_dead_code_elimination() {
    let pass = DeadCodeEliminationPass;
    // 这里可以添加具体的测试逻辑
    // 由于需要完整的IR结构，暂时只测试初始化
    assert_eq!(pass.name(), "DeadCodeElimination");
}
