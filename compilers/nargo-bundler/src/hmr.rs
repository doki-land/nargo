#![warn(missing_docs)]

use std::{
    sync::{Arc, Mutex},
    thread,
};
use ws::{Handler, Handshake, Message, Result as WsResult, listen};

use nargo_types::Result;

/// HMR 服务器 WebSocket 处理器
pub struct HmrHandler {
    /// 连接列表
    pub connections: Arc<Mutex<Vec<ws::Sender>>>,
}

impl Handler for HmrHandler {
    fn on_open(&mut self, _out: Handshake) -> WsResult<()> {
        println!("HMR: Client connected");
        // 将新连接添加到连接列表
        // 简化实现，暂时不存储连接
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        println!("HMR: Received message: {:?}", msg);
        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        println!("HMR: Client disconnected");
        // 连接会自动关闭，不需要手动从列表中移除
    }
}

/// 启动 HMR 服务器
pub fn start_hmr_server(connections: Arc<Mutex<Vec<ws::Sender>>>, port: u16) -> Result<()> {
    thread::spawn(move || {
        println!("HMR: Starting server on port {}", port);
        if let Err(e) = listen(format!("127.0.0.1:{}", port), |_| HmrHandler { connections: connections.clone() }) {
            println!("HMR: Server error: {:?}", e);
        }
    });

    Ok(())
}

/// 发送热更新通知
pub fn send_hmr_update(connections: &Arc<Mutex<Vec<ws::Sender>>>, module_name: &str) -> Result<()> {
    let message = serde_json::json!({ "type": "update", "module": module_name });
    let message_str = message.to_string();

    let mut connections = connections.lock().unwrap();
    connections.retain(|conn| match conn.send(Message::text(message_str.clone())) {
        Ok(_) => true,
        Err(e) => {
            println!("HMR: Failed to send update: {:?}", e);
            false
        }
    });

    Ok(())
}

/// 生成 HMR 客户端代码
pub fn generate_hmr_client(port: u16) -> String {
    let mut hmr_code = String::new();
    hmr_code.push_str("// HMR Client\n\n");
    hmr_code.push_str("// HMR 模块缓存\n");
    hmr_code.push_str("window.__HMR_MODULES__ = window.__HMR_MODULES__ || {};\n");
    hmr_code.push_str("window.__HMR_CLIENTS__ = window.__HMR_CLIENTS__ || [];\n");
    hmr_code.push_str("\n");

    hmr_code.push_str(&format!("const ws = new WebSocket('ws://localhost:{}');\n", port));
    hmr_code.push_str("\n");
    hmr_code.push_str("// 注册模块更新回调\n");
    hmr_code.push_str("window.hmrRegisterModule = (moduleName, updateCallback) => {\n");
    hmr_code.push_str("  window.__HMR_MODULES__[moduleName] = updateCallback;\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("// 发送模块注册信息\n");
    hmr_code.push_str("window.hmrSendModuleInfo = () => {\n");
    hmr_code.push_str("  if (ws.readyState === WebSocket.OPEN) {\n");
    hmr_code.push_str("    ws.send(JSON.stringify({\n");
    hmr_code.push_str("      type: 'register',\n");
    hmr_code.push_str("      modules: Object.keys(window.__HMR_MODULES__)\n");
    hmr_code.push_str("    }));\n");
    hmr_code.push_str("  }\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("// 处理模块更新\n");
    hmr_code.push_str("const handleModuleUpdate = async (moduleName, updateData) => {\n");
    hmr_code.push_str("  console.log('HMR: Updating module', moduleName);\n");
    hmr_code.push_str("  \n");
    hmr_code.push_str("  try {\n");
    hmr_code.push_str("    // 动态加载更新后的模块\n");
    hmr_code.push_str("    const response = await fetch(`${moduleName}.js?_t=${Date.now()}`);\n");
    hmr_code.push_str("    const code = await response.text();\n");
    hmr_code.push_str("    \n");
    hmr_code.push_str("    // 创建临时模块执行环境\n");
    hmr_code.push_str("    const module = { exports: {} };\n");
    hmr_code.push_str("    const require = (dep) => {\n");
    hmr_code.push_str("      if (window.__HMR_MODULES__[dep]) {\n");
    hmr_code.push_str("        return window.__HMR_MODULES__[dep];\n");
    hmr_code.push_str("      }\n");
    hmr_code.push_str("      throw new Error(`Module ${dep} not found`);\n");
    hmr_code.push_str("    };\n");
    hmr_code.push_str("    \n");
    hmr_code.push_str("    // 执行模块代码\n");
    hmr_code.push_str("    const exec = new Function('module', 'exports', 'require', code);\n");
    hmr_code.push_str("    exec(module, module.exports, require);\n");
    hmr_code.push_str("    \n");
    hmr_code.push_str("    // 调用模块的更新回调\n");
    hmr_code.push_str("    if (window.__HMR_MODULES__[moduleName]) {\n");
    hmr_code.push_str("      window.__HMR_MODULES__[moduleName](module.exports);\n");
    hmr_code.push_str("      console.log('HMR: Module updated successfully', moduleName);\n");
    hmr_code.push_str("    }\n");
    hmr_code.push_str("  } catch (error) {\n");
    hmr_code.push_str("    console.error('HMR: Failed to update module', moduleName, error);\n");
    hmr_code.push_str("    // 如果更新失败，回退到页面刷新\n");
    hmr_code.push_str("    window.location.reload();\n");
    hmr_code.push_str("  }\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("ws.onmessage = (event) => {\n");
    hmr_code.push_str("  try {\n");
    hmr_code.push_str("    const data = JSON.parse(event.data);\n");
    hmr_code.push_str("    \n");
    hmr_code.push_str("    switch (data.type) {\n");
    hmr_code.push_str("      case 'update':\n");
    hmr_code.push_str("        // 处理模块更新\n");
    hmr_code.push_str("        handleModuleUpdate(data.module, data);\n");
    hmr_code.push_str("        break;\n");
    hmr_code.push_str("      case 'reload':\n");
    hmr_code.push_str("        // 强制页面刷新\n");
    hmr_code.push_str("        console.log('HMR: Reloading page');\n");
    hmr_code.push_str("        window.location.reload();\n");
    hmr_code.push_str("        break;\n");
    hmr_code.push_str("      default:\n");
    hmr_code.push_str("        console.log('HMR: Unknown message type', data.type);\n");
    hmr_code.push_str("    }\n");
    hmr_code.push_str("  } catch (error) {\n");
    hmr_code.push_str("    console.error('HMR: Failed to process message', error);\n");
    hmr_code.push_str("  }\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("ws.onopen = () => {\n");
    hmr_code.push_str("  console.log('HMR: Connected');\n");
    hmr_code.push_str("  // 发送模块注册信息\n");
    hmr_code.push_str("  window.hmrSendModuleInfo();\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("ws.onclose = () => {\n");
    hmr_code.push_str("  console.log('HMR: Connection closed');\n");
    hmr_code.push_str("  // 尝试重连\n");
    hmr_code.push_str("  setTimeout(() => {\n");
    hmr_code.push_str("    console.log('HMR: Attempting to reconnect');\n");
    hmr_code.push_str("    window.location.reload();\n");
    hmr_code.push_str("  }, 1000);\n");
    hmr_code.push_str("};\n");
    hmr_code.push_str("\n");
    hmr_code.push_str("ws.onerror = (error) => {\n");
    hmr_code.push_str("  console.error('HMR: Connection error', error);\n");
    hmr_code.push_str("};\n");
    hmr_code
}
