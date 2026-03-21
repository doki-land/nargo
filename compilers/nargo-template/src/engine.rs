#![warn(missing_docs)]

/// 模板引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateEngine {
    /// 自定义模板引擎
    Custom,

    #[cfg(feature = "handlebars")]
    /// Handlebars 模板引擎
    Handlebars,

    #[cfg(feature = "dejavu")]
    /// DejaVu 模板引擎
    DejaVu,

    #[cfg(feature = "jinja")]
    /// Jinja2 模板引擎
    Jinja2,

    #[cfg(feature = "liquid")]
    /// Liquid 模板引擎
    Liquid,
}

impl TemplateEngine {
    /// 获取所有可用的模板引擎
    pub fn all() -> &'static [Self] {
        #[cfg(all(not(feature = "handlebars"), not(feature = "dejavu"), not(feature = "jinja"), not(feature = "liquid")))]
        {
            &[Self::Custom]
        }

        #[cfg(any(feature = "handlebars", feature = "dejavu", feature = "jinja", feature = "liquid"))]
        {
            &[
                Self::Custom,
                #[cfg(feature = "handlebars")]
                Self::Handlebars,
                #[cfg(feature = "dejavu")]
                Self::DejaVu,
                #[cfg(feature = "jinja")]
                Self::Jinja2,
                #[cfg(feature = "liquid")]
                Self::Liquid,
            ]
        }
    }

    /// 获取模板引擎的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Custom => "自定义",
            #[cfg(feature = "handlebars")]
            Self::Handlebars => "Handlebars",
            #[cfg(feature = "dejavu")]
            Self::DejaVu => "DejaVu",
            #[cfg(feature = "jinja")]
            Self::Jinja2 => "Jinja2",
            #[cfg(feature = "liquid")]
            Self::Liquid => "Liquid",
        }
    }

    /// 获取模板引擎的描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Custom => "自定义模板引擎",
            #[cfg(feature = "handlebars")]
            Self::Handlebars => "Handlebars 模板引擎，流行的 JavaScript 模板语言",
            #[cfg(feature = "dejavu")]
            Self::DejaVu => "DejaVu 模板引擎，编译期零成本抽象",
            #[cfg(feature = "jinja")]
            Self::Jinja2 => "Jinja2 模板引擎，Python 生态系统中流行的模板语言",
            #[cfg(feature = "liquid")]
            Self::Liquid => "Liquid 模板引擎，Shopify 开发的模板语言",
        }
    }

    /// 获取默认的文件扩展名
    pub fn default_extension(&self) -> &'static str {
        match self {
            Self::Custom => "tpl",
            #[cfg(feature = "handlebars")]
            Self::Handlebars => "hbs",
            #[cfg(feature = "dejavu")]
            Self::DejaVu => "dejavu",
            #[cfg(feature = "jinja")]
            Self::Jinja2 => "jinja2",
            #[cfg(feature = "liquid")]
            Self::Liquid => "liquid",
        }
    }
}
