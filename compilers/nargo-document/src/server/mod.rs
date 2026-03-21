use crate::{config::Config, generator::Generator};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc::channel};
use warp::{Filter, http::Uri};

/// 开发服务器，用于实时预览文档
pub struct DevServer {
    /// 配置信息
    config: Config,
    /// 文档生成器
    generator: Generator,
    /// 服务器端口
    port: u16,
    /// 输出目录
    output_dir: String,
}

impl DevServer {
    /// 创建新的开发服务器
    ///
    /// # 参数
    /// * `config` - 文档配置信息
    /// * `port` - 服务器端口
    /// * `output_dir` - 输出目录
    ///
    /// # 返回值
    /// 返回一个新的开发服务器实例
    pub fn new(config: Config, port: u16, output_dir: &str) -> Self {
        Self { config: config.clone(), generator: Generator::new(config), port, output_dir: output_dir.to_string() }
    }

    /// 启动开发服务器
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    pub async fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting development server on port {}", self.port);

        // 首先生成静态文件
        self.generator.generate(".", &self.output_dir)?;

        // 设置文件监听
        self.setup_file_watcher()?;

        // 设置 HTTP 服务器
        let output_dir = self.output_dir.clone();
        let routes = warp::fs::dir(output_dir).or(warp::path::end().and_then(|| async move { Ok::<_, warp::Rejection>(warp::redirect::temporary("/index.html".parse::<Uri>().unwrap())) }));
        let port = self.port;

        // 启动服务器
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;

        Ok(())
    }

    /// 设置文件监听器
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    fn setup_file_watcher(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| match res {
            Ok(event) => tx.send(event).unwrap(),
            Err(e) => println!("Error: {:?}", e),
        })?;

        // 监听当前目录下的 Markdown 文件
        watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

        let config = self.config.clone();
        let output_dir = self.output_dir.clone();

        // 启动线程处理文件变化
        std::thread::spawn(move || {
            for event in rx {
                if let Event { kind: EventKind::Modify(_), paths, .. } = event {
                    for path in paths {
                        if path.extension().map_or(false, |ext| ext == "md" || ext == "markdown") {
                            println!("File changed: {:?}", path);
                            let mut generator = Generator::new(config.clone());
                            if let Err(e) = generator.generate(".", &output_dir) {
                                eprintln!("Error regenerating documentation: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// 设置路由
    ///
    /// # 返回值
    /// 返回路由过滤器
    fn setup_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // 提供静态文件
        let static_files = warp::fs::dir(&self.output_dir);

        // 根路径重定向到 index.html
        let index = warp::path::end().and_then(|| async move { Ok::<_, warp::Rejection>(warp::redirect::temporary("/index.html".parse::<Uri>().unwrap())) });

        // 组合路由
        index.or(static_files)
    }
}
