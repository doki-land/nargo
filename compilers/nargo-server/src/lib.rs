//! Nargo development server.
//!
//! This module provides development server, static file serving, and mock server functionality.

#![warn(missing_docs)]

use nargo_types::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use nargo_types::NargoContext;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{info, warn};
use warp::Filter;

/// Server mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    /// Development mode with hot reload.
    Dev,
    /// Static file serving mode.
    Serve,
    /// Production mode.
    Prod,
}

/// Server options.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Server mode.
    pub mode: ServerMode,
    /// Root directory for static files.
    pub root: PathBuf,
    /// Port to listen on.
    pub port: u16,
    /// Host to bind to.
    pub host: String,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self { mode: ServerMode::Dev, root: PathBuf::from("."), port: 3000, host: "127.0.0.1".to_string() }
    }
}

/// Development server.
///
/// 负责启动开发服务器，提供静态文件服务和热重载功能
pub struct DevServer {
    /// 服务器选项
    options: ServerOptions,
    /// Nargo 上下文
    ctx: Arc<NargoContext>,
}

impl DevServer {
    /// 创建新的开发服务器
    ///
    /// # Arguments
    /// * `ctx` - Nargo 上下文
    /// * `options` - 服务器选项
    ///
    /// # Returns
    /// * `DevServer` - 新创建的开发服务器
    pub fn new(ctx: Arc<NargoContext>, options: ServerOptions) -> Self {
        Self { options, ctx }
    }

    /// 启动服务器
    ///
    /// # Returns
    /// * `Result<()>` - 启动结果
    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.options.host, self.options.port);
        info!("Starting dev server on {}", addr);

        // Create a broadcast channel for hot reload notifications
        let (tx, _rx) = broadcast::channel(10);

        // Start file watcher for hot reload
        if self.options.mode == ServerMode::Dev {
            self.start_file_watcher(tx.clone()).await;
        }

        let root = self.options.root.clone();
        let ctx = self.ctx.clone();

        // Define routes
        let static_files = warp::path::tail().and(warp::get()).and_then(move |path| {
            let root = root.clone();
            async move { serve_static_file(&root, path.as_str()).await }
        });

        let api = warp::path("api").and(warp::path::tail()).and(warp::any()).and_then(move |_path| {
            let ctx = ctx.clone();
            async move { handle_api_request(ctx).await }
        });

        let routes = static_files.or(api);

        // Start server
        warp::serve(routes).run(addr.parse()?).await;

        Ok(())
    }

    /// 启动文件监视器以支持热重载
    ///
    /// # Arguments
    /// * `tx` - 用于发送热重载通知的广播通道
    async fn start_file_watcher(&self, tx: broadcast::Sender<()>) {
        let root = self.options.root.clone();
        tokio::spawn(async move {
            let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) => {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        info!("File change detected: {:?}", event);
                        let _ = tx.send(());
                    }
                }
                Err(e) => warn!("File watcher error: {:?}", e),
            })
            .unwrap();

            watcher.watch(&root, notify::RecursiveMode::Recursive).unwrap();

            // Keep watcher running
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("File watcher stopped");
                }
            }
        });
    }
}

/// Run the server with the given options.
pub async fn run(ctx: Arc<NargoContext>, options: ServerOptions) -> Result<()> {
    let server = DevServer::new(ctx, options);
    server.start().await
}

/// Serve static files.
async fn serve_static_file(root: &Path, path: &str) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let file_path = if path.is_empty() { root.join("index.html") } else { root.join(path) };

    // Check if file exists
    if !file_path.is_file() {
        return Err(warp::reject::not_found());
    }

    Ok(warp::fs::file(file_path))
}

/// Handle API requests.
async fn handle_api_request(_ctx: Arc<NargoContext>) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    // TODO: Implement API request handling
    Ok(warp::reply::json(&serde_json::json!({
        "message": "API endpoint"
    })))
}

/// Mock route definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRoute {
    /// Route path pattern.
    pub path: String,
    /// HTTP method (GET, POST, etc.).
    #[serde(default = "default_method")]
    pub method: String,
    /// Response status code.
    #[serde(default = "default_status")]
    pub status: u16,
    /// Response headers.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Response body.
    #[serde(default)]
    pub body: String,
    /// Response body file path (alternative to body).
    #[serde(default)]
    pub body_file: Option<PathBuf>,
    /// Delay in milliseconds before responding.
    #[serde(default)]
    pub delay_ms: u64,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_status() -> u16 {
    200
}

/// Mock server for development.
///
/// 用于开发环境的模拟服务器，支持从 JSON 文件加载路由配置
pub struct MockServer {
    /// Nargo 上下文
    _ctx: Arc<NargoContext>,
    /// 模拟路由列表
    routes: Vec<MockRoute>,
}

impl MockServer {
    /// 创建新的模拟服务器
    ///
    /// # Arguments
    /// * `ctx` - Nargo 上下文
    ///
    /// # Returns
    /// * `MockServer` - 新创建的模拟服务器
    pub fn new(ctx: Arc<NargoContext>) -> Self {
        Self { _ctx: ctx, routes: Vec::new() }
    }

    /// 向模拟服务器添加路由
    ///
    /// # Arguments
    /// * `route` - 模拟路由
    pub fn add_route(&mut self, route: MockRoute) {
        self.routes.push(route);
    }

    /// 从目录加载路由配置
    ///
    /// # Arguments
    /// * `dir` - 包含 JSON 路由配置文件的目录
    ///
    /// # Returns
    /// * `Result<()>` - 加载结果
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir).map_err(|e| nargo_types::Error::external_error("server".to_string(), e.to_string(), nargo_types::Span::unknown()))? {
            let entry = entry.map_err(|e| nargo_types::Error::external_error("server".to_string(), e.to_string(), nargo_types::Span::unknown()))?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = std::fs::read_to_string(&path).map_err(|e| nargo_types::Error::external_error("server".to_string(), e.to_string(), nargo_types::Span::unknown()))?;
                let routes: Vec<MockRoute> = serde_json::from_str(&content).map_err(|e| nargo_types::Error::external_error("server".to_string(), e.to_string(), nargo_types::Span::unknown()))?;
                self.routes.extend(routes);
            }
        }
        Ok(())
    }

    /// 获取路由数量
    ///
    /// # Returns
    /// * `usize` - 路由数量
    pub fn routes_count(&self) -> usize {
        self.routes.len()
    }

    /// 启动模拟服务器
    ///
    /// # Arguments
    /// * `port` - 服务器端口
    ///
    /// # Returns
    /// * `Result<()>` - 启动结果
    pub async fn start(&self, port: u16) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        info!("Mock server running on {}", addr);
        info!("Loaded {} routes", self.routes.len());

        let routes = self.routes.clone();

        // Define mock routes
        let mock_routes = warp::any().and(warp::path::full()).and(warp::method()).and_then(move |path, method| {
            let routes = routes.clone();
            async move { handle_mock_request(routes, path.as_str(), method).await }
        });

        // Start server
        warp::serve(mock_routes).run(addr.parse()?).await;

        Ok(())
    }
}

/// Handle mock requests.
async fn handle_mock_request(routes: Vec<MockRoute>, path: &str, method: warp::http::Method) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let method_str = method.to_string();

    // Find matching route
    for route in &routes {
        if route.path == path && route.method == method_str {
            // Apply delay if specified
            if route.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(route.delay_ms)).await;
            }

            // Build response
            let mut resp = warp::http::Response::builder().status(route.status);

            // Add headers
            for (key, value) in &route.headers {
                resp = resp.header(key, value);
            }

            // Get body
            let body = if let Some(body_file) = &route.body_file {
                match tokio::fs::read(body_file).await {
                    Ok(content) => content,
                    Err(e) => format!("Error reading body file: {:?}", e).into_bytes(),
                }
            }
            else {
                route.body.as_bytes().to_vec()
            };

            return Ok(resp.body(body).unwrap());
        }
    }

    // No matching route
    Err(warp::reject::not_found())
}
