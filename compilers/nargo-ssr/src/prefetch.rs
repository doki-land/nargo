use nargo_types::NargoValue;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

/// 数据预取上下文
type PrefetchContext = HashMap<String, NargoValue>;

/// 数据预取函数类型
type PrefetchFn = Arc<dyn Fn(&HashMap<String, String>, &HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<NargoValue, String>>>> + Send + Sync>;

/// 数据预取管理器
pub struct PrefetchManager {
    /// 预取函数映射
    prefetch_functions: HashMap<String, PrefetchFn>,
    /// 缓存的预取结果
    prefetch_cache: HashMap<String, NargoValue>,
}

impl PrefetchManager {
    /// 创建新的数据预取管理器
    pub fn new() -> Self {
        Self { prefetch_functions: HashMap::new(), prefetch_cache: HashMap::new() }
    }

    /// 注册数据预取函数
    ///
    /// # Arguments
    /// * `key` - 预取函数的唯一标识
    /// * `prefetch_fn` - 预取函数
    pub fn register_prefetch<F, Fut>(&mut self, key: &str, prefetch_fn: F)
    where
        F: Fn(&HashMap<String, String>, &HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<NargoValue, String>> + Send + 'static,
    {
        self.prefetch_functions.insert(key.to_string(), Arc::new(move |params, query| Box::pin(prefetch_fn(params, query))));
    }

    /// 执行数据预取
    ///
    /// # Arguments
    /// * `keys` - 需要预取的键列表
    /// * `params` - 路由参数
    /// * `query` - 查询参数
    ///
    /// # Returns
    /// * `PrefetchContext` - 预取的上下文数据
    pub async fn prefetch(&mut self, keys: &[&str], params: &HashMap<String, String>, query: &HashMap<String, String>) -> PrefetchContext {
        let mut context = PrefetchContext::new();

        for key in keys {
            // 生成缓存键
            let cache_key = self.generate_cache_key(key, params, query);

            // 检查缓存
            if let Some(cached) = self.prefetch_cache.get(&cache_key) {
                context.insert(key.to_string(), cached.clone());
                continue;
            }

            // 执行预取
            if let Some(prefetch_fn) = self.prefetch_functions.get(*key) {
                match prefetch_fn(params, query).await {
                    Ok(value) => {
                        context.insert(key.to_string(), value.clone());
                        // 缓存结果
                        self.prefetch_cache.insert(cache_key, value);
                    }
                    Err(err) => {
                        eprintln!("Prefetch error for {}: {}", key, err);
                    }
                }
            }
        }

        context
    }

    /// 生成缓存键
    ///
    /// # Arguments
    /// * `key` - 预取函数的键
    /// * `params` - 路由参数
    /// * `query` - 查询参数
    ///
    /// # Returns
    /// * `String` - 缓存键
    fn generate_cache_key(&self, key: &str, params: &HashMap<String, String>, query: &HashMap<String, String>) -> String {
        let mut cache_key = key.to_string();

        // 添加路由参数
        let mut param_pairs: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        param_pairs.sort();
        for pair in param_pairs {
            cache_key.push_str(&format!("&{}", pair));
        }

        // 添加查询参数
        let mut query_pairs: Vec<String> = query.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        query_pairs.sort();
        for pair in query_pairs {
            cache_key.push_str(&format!("&{}", pair));
        }

        cache_key
    }

    /// 清除预取缓存
    pub fn clear_cache(&mut self) {
        self.prefetch_cache.clear();
    }
}
