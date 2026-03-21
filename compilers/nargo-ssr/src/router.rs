use std::{collections::HashMap, sync::Arc};

/// 路由参数类型
type RouteParams = HashMap<String, String>;

/// 路由匹配结果
#[derive(Clone)]
pub struct RouteMatch {
    /// 匹配的路由路径
    pub path: String,
    /// 路由参数
    pub params: RouteParams,
    /// 路由查询参数
    pub query: HashMap<String, String>,
}

/// 路由处理器 trait
pub trait RouteHandler: Send + Sync {
    /// 处理路由请求
    ///
    /// # Arguments
    /// * `params` - 路由参数
    /// * `query` - 查询参数
    ///
    /// # Returns
    /// * `Result<String, String>` - 渲染结果或错误信息
    fn handle(&self, params: &RouteParams, query: &HashMap<String, String>) -> Result<String, String>;
}

/// 路由表
pub struct Router {
    /// 路由规则映射
    routes: HashMap<String, Arc<dyn RouteHandler>>,
    /// 缓存的路由匹配结果
    route_cache: HashMap<String, RouteMatch>,
}

impl Router {
    /// 创建新的路由表
    pub fn new() -> Self {
        Self { routes: HashMap::new(), route_cache: HashMap::new() }
    }

    /// 添加路由
    ///
    /// # Arguments
    /// * `path` - 路由路径
    /// * `handler` - 路由处理器
    pub fn add_route<H: RouteHandler + 'static>(&mut self, path: &str, handler: H) {
        self.routes.insert(path.to_string(), Arc::new(handler));
    }

    /// 匹配路由
    ///
    /// # Arguments
    /// * `path` - 请求路径
    ///
    /// # Returns
    /// * `Option<(Arc<dyn RouteHandler>, RouteMatch)>` - 匹配的处理器和路由匹配结果
    pub fn match_route(&mut self, path: &str) -> Option<(Arc<dyn RouteHandler>, RouteMatch)> {
        // 检查缓存
        if let Some(match_result) = self.route_cache.get(path) {
            // 查找对应的处理器
            if let Some(handler) = self.routes.get(&match_result.path) {
                return Some((handler.clone(), match_result.clone()));
            }
        }

        // 解析路径和查询参数
        let (path_part, query_part) = path.split_once('?').unwrap_or((path, ""));
        let query = self.parse_query(query_part);

        // 匹配路由
        for (route_path, handler) in &self.routes {
            if let Some(params) = self.match_path(route_path, path_part) {
                let match_result = RouteMatch { path: route_path.clone(), params, query };
                // 缓存匹配结果
                self.route_cache.insert(path.to_string(), match_result.clone());
                return Some((handler.clone(), match_result));
            }
        }

        None
    }

    /// 解析查询参数
    ///
    /// # Arguments
    /// * `query_str` - 查询参数字符串
    ///
    /// # Returns
    /// * `HashMap<String, String>` - 解析后的查询参数
    fn parse_query(&self, query_str: &str) -> HashMap<String, String> {
        let mut query = HashMap::new();
        for pair in query_str.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                query.insert(key.to_string(), value.to_string());
            }
        }
        query
    }

    /// 匹配路径
    ///
    /// # Arguments
    /// * `route_path` - 路由路径模板
    /// * `request_path` - 请求路径
    ///
    /// # Returns
    /// * `Option<RouteParams>` - 匹配的路由参数
    fn match_path(&self, route_path: &str, request_path: &str) -> Option<RouteParams> {
        let route_parts: Vec<&str> = route_path.split('/').filter(|p| !p.is_empty()).collect();
        let request_parts: Vec<&str> = request_path.split('/').filter(|p| !p.is_empty()).collect();

        if route_parts.len() != request_parts.len() {
            return None;
        }

        let mut params = RouteParams::new();
        for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
            if route_part.starts_with(':') {
                // 动态参数
                let param_name = route_part.trim_start_matches(':');
                params.insert(param_name.to_string(), request_part.to_string());
            }
            else if route_part != request_part {
                // 路径不匹配
                return None;
            }
        }

        Some(params)
    }

    /// 清除路由缓存
    pub fn clear_cache(&mut self) {
        self.route_cache.clear();
    }
}
