use nargo_types::Result;

/// 示例插件，用于演示插件系统的使用
pub struct ExamplePlugin {
    /// 插件配置
    config: Option<super::super::PluginConfig>,
}

impl ExamplePlugin {
    /// 创建新的示例插件
    pub fn new() -> Self {
        Self { config: None }
    }
}

impl super::super::Plugin for ExamplePlugin {
    /// 获取插件名称
    fn name(&self) -> String {
        "example-plugin".to_string()
    }

    /// 获取插件描述
    fn description(&self) -> String {
        "示例插件，用于演示插件系统的使用".to_string()
    }

    /// 初始化插件
    fn init(&mut self, config: &super::super::PluginConfig) -> Result<()> {
        self.config = Some(config.clone());
        println!("ExamplePlugin initialized with config: {:?}", config);
        Ok(())
    }

    /// 执行插件逻辑
    fn run(&mut self, _ir: &mut super::super::IRModule, lifecycle: super::super::PluginLifecycle) -> Result<()> {
        println!("ExamplePlugin running at lifecycle: {:?}", lifecycle);

        // 根据不同的生命周期阶段执行不同的逻辑
        match lifecycle {
            super::super::PluginLifecycle::Init => {
                println!("ExamplePlugin: Init phase");
            }
            super::super::PluginLifecycle::PreTransform => {
                println!("ExamplePlugin: PreTransform phase");
            }
            super::super::PluginLifecycle::Transform => {
                println!("ExamplePlugin: Transform phase");
            }
            super::super::PluginLifecycle::PostTransform => {
                println!("ExamplePlugin: PostTransform phase");
            }
            super::super::PluginLifecycle::Cleanup => {
                println!("ExamplePlugin: Cleanup phase");
            }
        }

        Ok(())
    }

    /// 清理插件资源
    fn cleanup(&mut self) -> Result<()> {
        println!("ExamplePlugin cleaned up");
        Ok(())
    }
}
