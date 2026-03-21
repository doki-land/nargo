use nargo_compiler::Compiler;

#[test]
fn test_compiler_pipeline() {
    let mut compiler = Compiler::new();
    let source = r#"
<script>
const [count, setCount] = createSignal(0);
function inc() { setCount(count() + 1); }
</script>
<template>
  <div class="p-4 text-blue">
    <span>Count: {{ count }}</span>
    <button @click="inc">+</button>
  </div>
</template>
"#;
    let js = compiler.compile("Counter", source).unwrap();

    // Check if compilation succeeds
    assert!(!js.code.is_empty());
    assert!(js.code.contains("const Counter = {"));
    assert!(js.code.contains("name: 'Counter'"));
}

#[test]
fn test_compiler_full_cycle() {
    let mut compiler = Compiler::new();
    let source = r#"
<template>
  <div class="container">
    <h1 @click="increment">Count: {{ count }}</h1>
    <button :disabled="isMax">Add</button>
  </div>
</template>

<script>
const [count, setCount] = createSignal(0);
const isMax = createComputed(() => count() >= 10);
function increment() {
    setCount(count() + 1);
}
</script>
"#;
    let js = compiler.compile("App", source).unwrap();
    println!("Generated JS:\n{}", js.code);

    // Check if compilation succeeds
    assert!(!js.code.is_empty());
    assert!(js.code.contains("const App = {"));
    assert!(js.code.contains("name: 'App'"));
}

#[test]
fn test_compiler_tailwind() {
    let mut compiler = Compiler::new();
    let source = r#"
<script>
addStyle("p-6 m-4");
</script>
<template>
  <div class="p-4 m-2 flex items-center bg-blue text-white rounded-lg shadow-sm">
    <span :class="'font-bold text-2xl'">Tailwind Test</span>
    <p :class="['text-center', 'mx-4']">Dynamic array</p>
  </div>
</template>
"#;
    let res = compiler.compile("TailwindComp", source).unwrap();

    // Check if compilation succeeds
    assert!(!res.code.is_empty());
    assert!(res.code.contains("const TailwindComp = {"));
    assert!(res.code.contains("name: 'TailwindComp'"));
}

#[test]
fn test_compiler_pug() {
    let mut compiler = Compiler::new();
    let source = r#"
<template lang="pug">
div.container
  h1 Hello Pug
  p(id="desc", :class="activeClass") This is a pug template
</template>
"#;
    let res = compiler.compile("PugComp", source).unwrap();
    println!("Generated Pug JS:\n{}", res.code);

    // Check if the output contains the correct h calls (simplified test)
    assert!(res.code.contains("const PugComp = {"));
    assert!(res.code.contains("name: 'PugComp'"));
}

#[test]
fn test_compiler_typescript() {
    use nargo_compiler::{CompileOptions, prelude::OutputFormat};

    let mut compiler = Compiler::new();
    let source = r#"
<template>
  <div>
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
  </div>
</template>

<script>
export default {
  data() {
    return {
      title: 'Hello',
      message: 'World'
    };
  }
};
</script>

<style>
h1 {
  color: blue;
}
</style>
"#;

    // 测试 TypeScript 类型定义生成
    let options = CompileOptions::default();
    let result = compiler.compile_with_format("TestComponent", source, OutputFormat::TypeScript, options).unwrap();

    println!("Generated TypeScript definitions:\n{}", String::from_utf8_lossy(&result.outputs[0].code));

    // 检查生成的 TypeScript 类型定义
    assert!(!result.outputs[0].code.is_empty());

    // 测试生成所有格式的代码
    let all_results = compiler.compile_all_formats("TestComponent", source, CompileOptions::default()).unwrap();

    // 检查是否生成了所有支持的格式
    assert!(all_results.contains_key(&OutputFormat::JavaScript));
    assert!(all_results.contains_key(&OutputFormat::CSS));
    assert!(all_results.contains_key(&OutputFormat::HTML));
    assert!(all_results.contains_key(&OutputFormat::TypeScript));
    assert!(all_results.contains_key(&OutputFormat::WebAssembly));

    // 检查每种格式的代码是否生成成功
    for (format, output) in all_results {
        println!("\nFormat: {:?}", format);
        println!("File name: {}", output.entry_file);
        println!("Code length: {} bytes", output.outputs[0].code.len());
        assert!(!output.outputs[0].code.is_empty());
    }
}
