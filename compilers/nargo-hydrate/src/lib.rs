#![warn(missing_docs)]

use nargo_bundler::targets::js::{JsBackend, writer::JsWriter};
use nargo_ir::{IRModule, TemplateNodeIR};
use nargo_types::{CompileMode, Result};
use std::collections::HashSet;

pub struct HydrateBackend {
    pub runtime_path: String,
    pub mode: CompileMode,
}

impl HydrateBackend {
    pub fn new(mode: CompileMode) -> Self {
        let runtime_path = match mode {
            CompileMode::Vue2 => "nargo".to_string(),
            CompileMode::Vue => "vue".to_string(),
        };
        Self { runtime_path, mode }
    }
}

impl Default for HydrateBackend {
    fn default() -> Self {
        Self::new(CompileMode::Vue2)
    }
}

impl HydrateBackend {
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();

        // 1. Generate Hydrate Function Body
        let mut body_writer = JsWriter::new();
        Self::generate_hydrate_body(ir, &mut body_writer, &mut used_core)?;

        // 2. Generate Imports
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            let import_source = if self.mode == CompileMode::Vue { "vue".to_string() } else { format!("{}/core", self.runtime_path) };
            writer.write_line(&format!("import {{ {} }} from '{}';", imports.join(", "), import_source));
            writer.newline();
        }

        // 3. Append Body
        writer.append(body_writer);

        Ok(writer.finish().0)
    }

    fn generate_hydrate_body(ir: &IRModule, writer: &mut JsWriter, used_core: &mut HashSet<String>) -> Result<()> {
        writer.write_block("export function hydrate(root, ctx, options = {})", |writer| {
            // 1. Initialize state from server if available
            writer.write_line("if (window.__Nargo_STATE__) {");
            writer.indent();
            writer.write_line("Object.assign(ctx, window.__Nargo_STATE__);");
            writer.dedent();
            writer.write_line("}");
            writer.newline();

            // 2. Execute normal script and client script to set up reactivity
            let mut used_dom = HashSet::new();
            if let Some(script) = &ir.script {
                for stmt in &script.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false);
                }
            }
            if let Some(script_client) = &ir.script_client {
                for stmt in &script_client.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false);
                }
            }
            writer.newline();

            // 3. Setup DOM nodes mapping
            writer.write_line("const nodes = new Map();");
            writer.write_line("const eventHandlers = new Map();");
            writer.write_line("const selector = options.region ? `[data-nargo-region='${options.region}'] [data-nargo-id]` : '[data-nargo-id]';");
            writer.write_line("const elements = root.querySelectorAll(selector);");
            writer.write_line("for (let i = 0; i < elements.length; i++) {");
            writer.indent();
            writer.write_line("const el = elements[i];");
            writer.write_line("const id = el.getAttribute('data-nargo-id');");
            writer.write_line("if (id) nodes.set(id, el);");
            writer.dedent();
            writer.write_line("}");
            writer.newline();

            // 4. Setup event delegation
            writer.write_line("root.addEventListener('click', (e) => {");
            writer.indent();
            writer.write_line("let target = e.target;");
            writer.write_line("while (target && target !== root) {");
            writer.indent();
            writer.write_line("const id = target.getAttribute('data-nargo-id');");
            writer.write_line("if (id) {");
            writer.indent();
            writer.write_line("const handlers = eventHandlers.get('click');");
            writer.write_line("if (handlers && handlers[id]) {");
            writer.indent();
            writer.write_line("handlers[id](e);");
            writer.dedent();
            writer.write_line("}");
            writer.dedent();
            writer.write_line("}");
            writer.write_line("target = target.parentElement;");
            writer.dedent();
            writer.write_line("}");
            writer.dedent();
            writer.write_line("});");
            writer.newline();

            // 4. Generate hydration logic for template
            if let Some(template) = &ir.template {
                let mut node_index = 0;
                for node in &template.nodes {
                    Self::generate_node_hydrate(node, writer, &mut node_index, used_core)?;
                }
                Ok(())
            }
            else {
                Ok(())
            }
        });
        writer.newline();

        // Generate partial hydrate function
        writer.write_block("export function partialHydrate(root, ctx, region)", |writer| {
            writer.write_line("return hydrate(root, ctx, { region });");
            Ok(())
        });
        Ok(())
    }

    fn generate_node_hydrate(node: &TemplateNodeIR, writer: &mut JsWriter, node_index: &mut usize, used_core: &mut HashSet<String>) -> Result<()> {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                // Only generate hydration code if the element is dynamic
                if !el.is_static {
                    let el_var = format!("el{}", current_index);
                    writer.write_line(&format!("const {} = nodes.get('{}');", el_var, current_index));

                    // Handle event listeners (directives like on:click and @click)
                    for attr in &el.attributes {
                        if (attr.is_directive && attr.name == "on") || attr.name.starts_with('@') {
                            let event_name = if attr.name == "on" {
                                attr.argument.as_deref().unwrap_or("")
                            }
                            else {
                                // Handle @click syntax
                                attr.name.trim_start_matches('@')
                            };
                            if !event_name.is_empty() {
                                if let Some(value) = &attr.value {
                                    writer.write_line(&format!("if (!eventHandlers.has('{}')) eventHandlers.set('{}', {{}});", event_name, event_name));
                                    writer.write_line(&format!("eventHandlers.get('{}')['{}'] = (e) => {};", event_name, current_index, value));
                                }
                            }
                        }
                        else if attr.is_dynamic || attr.name.starts_with(':') {
                            // Handle dynamic attributes (e.g. bind:class, :class, bind:style)
                            let attr_name = if attr.name == "bind" {
                                attr.argument.as_deref().unwrap_or("")
                            }
                            else if attr.name.starts_with(':') {
                                // Handle :class syntax
                                attr.name.trim_start_matches(':')
                            }
                            else {
                                &attr.name
                            };
                            if !attr_name.is_empty() {
                                let value = attr.value.as_deref().unwrap_or("null");
                                // Check if value is likely dynamic (contains variables or expressions)
                                if value.contains('+') || value.contains('(') || value.contains('{') || value.contains('.') || value.contains('[') {
                                    used_core.insert("createEffect".to_string());
                                    let last_value_var = format!("last{}Val{}", attr_name, current_index);
                                    writer.write_line(&format!("let {} = {};", last_value_var, value));
                                    writer.write_block("createEffect(() =>", |writer| {
                                        writer.write_line(&format!("const currentValue = {};", value));
                                        writer.write_line(&format!("if (currentValue !== {}) {{  ", last_value_var));
                                        writer.indent();
                                        writer.write_line(&format!("{}.setAttribute('{}', currentValue);", el_var, attr_name));
                                        writer.write_line(&format!("{} = currentValue;", last_value_var));
                                        writer.dedent();
                                        writer.write_line("}");
                                        Ok(())
                                    });
                                    writer.write_line(");");
                                }
                                else {
                                    // Static value, set once
                                    writer.write_line(&format!("{}.setAttribute('{}', {});", el_var, attr_name, value));
                                }
                            }
                        }
                    }
                }

                // Always recurse into children to keep node_index in sync with SSR
                for child in &el.children {
                    Self::generate_node_hydrate(child, writer, node_index, used_core)?;
                }
                Ok(())
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;

                writer.write_line(&format!("const text{} = nodes.get('{}');", current_index, current_index));
                // Check if expression is likely dynamic
                if expr.code.contains('+') || expr.code.contains('(') || expr.code.contains('{') || expr.code.contains('.') || expr.code.contains('[') {
                    used_core.insert("createEffect".to_string());
                    writer.write_line(&format!("let lastText{} = {};", current_index, expr.code));
                    writer.write_block("createEffect(() =>", |writer| {
                        writer.write_line(&format!("const currentText = {};", expr.code));
                        writer.write_line(&format!("if (currentText !== lastText{}) {{  ", current_index));
                        writer.indent();
                        writer.write_line(&format!("text{}.textContent = currentText;", current_index));
                        writer.write_line(&format!("lastText{} = currentText;", current_index));
                        writer.dedent();
                        writer.write_line("}");
                        Ok(())
                    });
                    writer.write_line(");");
                }
                else {
                    // Static value, set once
                    writer.write_line(&format!("text{}.textContent = {};", current_index, expr.code));
                }
                Ok(())
            }
            _ => {
                // Static text, comments and hoisted nodes still occupy an index
                *node_index += 1;
                Ok(())
            }
        }
    }
}
