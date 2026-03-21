use nargo_compiler::{CompileMode, CompileOptions, CompileResult, Compiler, compile};
use nargo_ir::IRModule;

#[test]
fn test_basic_compilation() {
    let source = r#"<template>
  <div>Hello, World!</div>
</template>
<script>
  console.log('Hello from script');
</script>
<style>
  div { color: red; }
</style>"#;

    let result = compile("test.hxo", source).unwrap();
    assert!(!result.code.is_empty());
}

#[test]
fn test_compile_with_options() {
    let source = r#"<template>
  <div>Hello, World!</div>
</template>
<script>
  console.log('Hello from script');
</script>"#;

    let mut compiler = Compiler::new();
    let options = CompileOptions { mode: CompileMode::Vue, ssr: false, hydrate: false, minify: true, is_prod: true, target: Some("es2015".to_string()), scope_id: None, i18n_locale: None, vue_reactivity_transform: false, vue_define_model: false, vue_props_destructure: false };

    let result = compiler.compile_with_options("test.hxo", source, options).unwrap();
    assert!(!result.code.is_empty());
}

#[test]
fn test_vue2_compilation() {
    let source = r#"<template>
  <div>Hello, Vue 2!</div>
</template>
<script>
export default {
  data() {
    return {
      message: 'Hello'
    };
  }
};
</script>"#;

    let mut compiler = Compiler::new();
    let options = CompileOptions { mode: CompileMode::Vue2, ssr: false, hydrate: false, minify: false, is_prod: false, target: Some("es2015".to_string()), scope_id: None, i18n_locale: None, vue_reactivity_transform: false, vue_define_model: false, vue_props_destructure: false };

    let result = compiler.compile_with_options("test-vue2.hxo", source, options).unwrap();
    assert!(!result.code.is_empty());
}
