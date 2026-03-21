#![warn(missing_docs)]

use crate::types::{BuildOutput, BuildOutputs, OutputFormat};
use nargo_ir::IRModule;

use crate::analyzer::is_route_component;

/// 生成单文件打包输出
pub fn split_single_file(runtime_code: &str, hmr_code: &str, compiled_modules: &[(String, String)], modules: &[IRModule]) -> BuildOutputs {
    let mut output = String::new();
    output.push_str(runtime_code);
    output.push_str(hmr_code);
    output.push_str("\n// --- Components --\n");

    for (name, code) in compiled_modules {
        output.push_str(&format!("\n// --- Component: {} --\n{}\n", name, code));
    }

    if let Some(main) = modules.first() {
        output.push_str(&format!("\n// --- Mount --\n"));
        output.push_str(&format!("render(h({}, null), document.getElementById('app') || document.body);\n", main.name));
    }

    let build_output = BuildOutput { code: output.into_bytes(), format: OutputFormat::JavaScript, filename: "bundle.js".to_string() };

    BuildOutputs { outputs: vec![build_output], entry_file: "bundle.js".to_string() }
}

/// 按组件分割打包输出
pub fn split_by_component(runtime_code: &str, hmr_code: &str, compiled_modules: &[(String, String)], modules: &[IRModule]) -> BuildOutputs {
    let mut outputs = Vec::new();

    // 生成运行时文件
    let runtime_output = BuildOutput { code: runtime_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "runtime.js".to_string() };
    outputs.push(runtime_output);

    // 生成 HMR 客户端文件
    if !hmr_code.is_empty() {
        let hmr_output = BuildOutput { code: hmr_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "hmr.js".to_string() };
        outputs.push(hmr_output);
    }

    // 为每个组件生成单独的文件
    for (name, code) in compiled_modules {
        let component_output = BuildOutput { code: code.clone().into_bytes(), format: OutputFormat::JavaScript, filename: format!("{}.js", name) };
        outputs.push(component_output);
    }

    // 生成入口文件
    let mut entry_code = String::new();
    entry_code.push_str("// Entry Point\n");
    entry_code.push_str("import './runtime.js';\n");
    if !hmr_code.is_empty() {
        entry_code.push_str("import './hmr.js';\n");
    }

    for module in modules {
        entry_code.push_str(&format!("import {} from './{}.js';\n", module.name, module.name));
    }

    if let Some(main) = modules.first() {
        entry_code.push_str(&format!("\n// Mount\n"));
        entry_code.push_str(&format!("render(h({}, null), document.getElementById('app') || document.body);\n", main.name));
    }

    let entry_output = BuildOutput { code: entry_code.into_bytes(), format: OutputFormat::JavaScript, filename: "index.js".to_string() };
    outputs.push(entry_output);

    BuildOutputs { outputs, entry_file: "index.js".to_string() }
}

/// 按路由分割打包输出
pub fn split_by_route(runtime_code: &str, hmr_code: &str, compiled_modules: &[(String, String)], modules: &[IRModule]) -> BuildOutputs {
    let mut outputs = Vec::new();
    let mut route_components = Vec::new();
    let mut regular_components = Vec::new();

    // 生成运行时文件
    let runtime_output = BuildOutput { code: runtime_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "runtime.js".to_string() };
    outputs.push(runtime_output);

    // 生成 HMR 客户端文件
    if !hmr_code.is_empty() {
        let hmr_output = BuildOutput { code: hmr_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "hmr.js".to_string() };
        outputs.push(hmr_output);
    }

    // 分离路由组件和普通组件
    for (i, module) in modules.iter().enumerate() {
        if is_route_component(module) {
            route_components.push((i, module));
        }
        else {
            regular_components.push((i, module));
        }
    }

    // 生成普通组件文件（共享 chunk）
    let mut shared_code = String::new();
    for (i, module) in &regular_components {
        if let Some((name, code)) = compiled_modules.get(*i) {
            shared_code.push_str(&format!("// Component: {}\n{}\n", name, code));
        }
    }

    if !shared_code.is_empty() {
        let shared_output = BuildOutput { code: shared_code.into_bytes(), format: OutputFormat::JavaScript, filename: "shared.js".to_string() };
        outputs.push(shared_output);
    }

    // 为每个路由组件生成单独的文件
    for (i, module) in &route_components {
        if let Some((name, code)) = compiled_modules.get(*i) {
            let route_output = BuildOutput { code: code.clone().into_bytes(), format: OutputFormat::JavaScript, filename: format!("route-{}.js", name) };
            outputs.push(route_output);
        }
    }

    // 生成入口文件
    let mut entry_code = String::new();
    entry_code.push_str("// Entry Point\n");
    entry_code.push_str("import './runtime.js';\n");
    if !hmr_code.is_empty() {
        entry_code.push_str("import './hmr.js';\n");
    }
    if !regular_components.is_empty() {
        entry_code.push_str("import './shared.js';\n");
    }

    // 动态导入路由组件
    for (i, module) in &route_components {
        if let Some((name, _)) = compiled_modules.get(*i) {
            entry_code.push_str(&format!("const {} = () => import('./route-{}.js');\n", name, name));
        }
    }

    // 导入普通组件
    for (i, module) in &regular_components {
        if let Some((name, _)) = compiled_modules.get(*i) {
            entry_code.push_str(&format!("import {} from './shared.js';\n", name));
        }
    }

    if let Some(main) = modules.first() {
        entry_code.push_str(&format!("\n// Mount\n"));
        entry_code.push_str(&format!("render(h({}, null), document.getElementById('app') || document.body);\n", main.name));
    }

    let entry_output = BuildOutput { code: entry_code.into_bytes(), format: OutputFormat::JavaScript, filename: "index.js".to_string() };
    outputs.push(entry_output);

    BuildOutputs { outputs, entry_file: "index.js".to_string() }
}

/// 自定义分割策略打包输出
pub fn split_custom(runtime_code: &str, hmr_code: &str, compiled_modules: &[(String, String)], modules: &[IRModule]) -> BuildOutputs {
    let mut outputs = Vec::new();
    let mut large_modules = Vec::new();
    let mut small_modules = Vec::new();

    // 生成运行时文件
    let runtime_output = BuildOutput { code: runtime_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "runtime.js".to_string() };
    outputs.push(runtime_output);

    // 生成 HMR 客户端文件
    if !hmr_code.is_empty() {
        let hmr_output = BuildOutput { code: hmr_code.as_bytes().to_vec(), format: OutputFormat::JavaScript, filename: "hmr.js".to_string() };
        outputs.push(hmr_output);
    }

    // 根据模块大小分离模块
    for (i, module) in modules.iter().enumerate() {
        let code_size = if let Some((_, code)) = compiled_modules.get(i) { code.len() } else { 0 };

        // 大于10KB的模块作为单独chunk
        if code_size > 10 * 1024 {
            large_modules.push((i, module));
        }
        else {
            small_modules.push((i, module));
        }
    }

    // 生成小模块的共享chunk
    let mut shared_code = String::new();
    for (i, module) in &small_modules {
        if let Some((name, code)) = compiled_modules.get(*i) {
            shared_code.push_str(&format!("// Component: {}\n{}\n", name, code));
        }
    }

    if !shared_code.is_empty() {
        let shared_output = BuildOutput { code: shared_code.into_bytes(), format: OutputFormat::JavaScript, filename: "shared.js".to_string() };
        outputs.push(shared_output);
    }

    // 为每个大模块生成单独的文件
    for (i, module) in &large_modules {
        if let Some((name, code)) = compiled_modules.get(*i) {
            let large_output = BuildOutput { code: code.clone().into_bytes(), format: OutputFormat::JavaScript, filename: format!("chunk-{}.js", name) };
            outputs.push(large_output);
        }
    }

    // 生成入口文件
    let mut entry_code = String::new();
    entry_code.push_str("// Entry Point\n");
    entry_code.push_str("import './runtime.js';\n");
    if !hmr_code.is_empty() {
        entry_code.push_str("import './hmr.js';\n");
    }
    if !small_modules.is_empty() {
        entry_code.push_str("import './shared.js';\n");
    }

    // 动态导入大模块
    for (i, module) in &large_modules {
        if let Some((name, _)) = compiled_modules.get(*i) {
            entry_code.push_str(&format!("const {} = () => import('./chunk-{}.js');\n", name, name));
        }
    }

    // 导入小模块
    for (i, module) in &small_modules {
        if let Some((name, _)) = compiled_modules.get(*i) {
            entry_code.push_str(&format!("import {} from './shared.js';\n", name));
        }
    }

    if let Some(main) = modules.first() {
        entry_code.push_str(&format!("\n// Mount\n"));
        entry_code.push_str(&format!("render(h({}, null), document.getElementById('app') || document.body);\n", main.name));
    }

    let entry_output = BuildOutput { code: entry_code.into_bytes(), format: OutputFormat::JavaScript, filename: "index.js".to_string() };
    outputs.push(entry_output);

    BuildOutputs { outputs, entry_file: "index.js".to_string() }
}
