#![feature(new_range_api)]
#![warn(missing_docs)]
use nargo_types::NargoValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模块标识符
pub type ModuleId = String;

/// 消息标识符
pub type MessageId = String;

/// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// 普通消息
    Message,
    /// 事件通知
    Event,
    /// 请求消息
    Request,
    /// 响应消息
    Response,
    /// 广播消息
    Broadcast,
    /// 多播消息
    Multicast,
    /// 错误消息
    Error,
    /// 心跳消息
    Ping,
    /// 心跳响应消息
    Pong,
    /// 订阅消息
    Subscribe,
    /// 取消订阅消息
    Unsubscribe,
}

/// 事件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 模块初始化完成
    ModuleInitialized,
    /// 模块状态更新
    ModuleStateUpdated,
    /// 编译开始
    CompileStarted,
    /// 编译完成
    CompileCompleted,
    /// 编译错误
    CompileError,
    /// 文件变更
    FileChanged,
    /// 配置变更
    ConfigurationChanged,
    /// 模块注册
    ModuleRegistered,
    /// 模块注销
    ModuleUnregistered,
    /// 消息发送
    MessageSent,
    /// 消息接收
    MessageReceived,
    /// 心跳超时
    HeartbeatTimeout,
    /// 网络连接断开
    NetworkDisconnected,
    /// 网络连接恢复
    NetworkConnected,
    /// 资源加载开始
    ResourceLoadStarted,
    /// 资源加载完成
    ResourceLoadCompleted,
    /// 资源加载错误
    ResourceLoadError,
    /// 性能警告
    PerformanceWarning,
    /// 系统警告
    SystemWarning,
    /// 系统错误
    SystemError,
}

impl EventType {
    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            EventType::ModuleInitialized => "ModuleInitialized",
            EventType::ModuleStateUpdated => "ModuleStateUpdated",
            EventType::CompileStarted => "CompileStarted",
            EventType::CompileCompleted => "CompileCompleted",
            EventType::CompileError => "CompileError",
            EventType::FileChanged => "FileChanged",
            EventType::ConfigurationChanged => "ConfigurationChanged",
            EventType::ModuleRegistered => "ModuleRegistered",
            EventType::ModuleUnregistered => "ModuleUnregistered",
            EventType::MessageSent => "MessageSent",
            EventType::MessageReceived => "MessageReceived",
            EventType::HeartbeatTimeout => "HeartbeatTimeout",
            EventType::NetworkDisconnected => "NetworkDisconnected",
            EventType::NetworkConnected => "NetworkConnected",
            EventType::ResourceLoadStarted => "ResourceLoadStarted",
            EventType::ResourceLoadCompleted => "ResourceLoadCompleted",
            EventType::ResourceLoadError => "ResourceLoadError",
            EventType::PerformanceWarning => "PerformanceWarning",
            EventType::SystemWarning => "SystemWarning",
            EventType::SystemError => "SystemError",
        }
        .to_string()
    }
}

/// 消息状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    /// 消息已创建
    Created,
    /// 消息已发送
    Sent,
    /// 消息已接收
    Received,
    /// 消息已确认
    Acknowledged,
    /// 消息已超时
    Timeout,
    /// 消息发送失败
    Failed,
}

/// 消息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID
    pub id: MessageId,
    /// 消息类型
    pub message_type: MessageType,
    /// 发送方模块ID
    pub sender: ModuleId,
    /// 接收方模块ID
    pub receiver: ModuleId,
    /// 事件类型（仅当消息类型为Event时有效）
    pub event_type: Option<EventType>,
    /// 消息数据
    pub data: NargoValue,
    /// 时间戳
    pub timestamp: u64,
    /// 消息状态
    pub status: MessageStatus,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
    /// 重试次数
    pub retry_count: u8,
}

impl Message {
    /// 创建普通消息
    pub fn new_message(sender: ModuleId, receiver: ModuleId, data: NargoValue) -> Self {
        Self {
            id: format!("msg-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Message,
            sender,
            receiver,
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(5000),
            retry_count: 0,
        }
    }

    /// 创建事件消息
    pub fn new_event(sender: ModuleId, event_type: EventType, data: NargoValue) -> Self {
        Self {
            id: format!(
                "event-{}-{}",
                event_type.to_string(),
                chrono::Utc::now().timestamp_millis()
            ),
            message_type: MessageType::Event,
            sender,
            receiver: "broadcast".to_string(),
            event_type: Some(event_type),
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(3000),
            retry_count: 0,
        }
    }

    /// 创建请求消息
    pub fn new_request(sender: ModuleId, receiver: ModuleId, data: NargoValue) -> Self {
        Self {
            id: format!("req-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Request,
            sender,
            receiver,
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(10000),
            retry_count: 3,
        }
    }

    /// 创建响应消息
    pub fn new_response(
        request_id: MessageId,
        sender: ModuleId,
        receiver: ModuleId,
        data: NargoValue,
    ) -> Self {
        Self {
            id: format!("resp-{}", request_id),
            message_type: MessageType::Response,
            sender,
            receiver,
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(5000),
            retry_count: 0,
        }
    }

    /// 创建广播消息
    pub fn new_broadcast(sender: ModuleId, data: NargoValue) -> Self {
        Self {
            id: format!("broadcast-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Broadcast,
            sender,
            receiver: "broadcast".to_string(),
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(3000),
            retry_count: 0,
        }
    }

    /// 创建心跳消息
    pub fn new_ping(sender: ModuleId, receiver: ModuleId) -> Self {
        Self {
            id: format!("ping-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Ping,
            sender,
            receiver,
            event_type: None,
            data: NargoValue::Null,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(2000),
            retry_count: 1,
        }
    }

    /// 创建心跳响应消息
    pub fn new_pong(sender: ModuleId, receiver: ModuleId, ping_id: MessageId) -> Self {
        let data = NargoValue::Object({
            let mut map = HashMap::new();
            map.insert("ping_id".to_string(), NargoValue::String(ping_id));
            map
        });
        Self {
            id: format!("pong-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Pong,
            sender,
            receiver,
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(1000),
            retry_count: 0,
        }
    }

    /// 创建订阅消息
    pub fn new_subscribe(sender: ModuleId, event_type: EventType) -> Self {
        Self {
            id: format!(
                "subscribe-{}-{}",
                event_type.to_string(),
                chrono::Utc::now().timestamp_millis()
            ),
            message_type: MessageType::Subscribe,
            sender,
            receiver: "system".to_string(),
            event_type: Some(event_type),
            data: NargoValue::Null,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(3000),
            retry_count: 0,
        }
    }

    /// 创建取消订阅消息
    pub fn new_unsubscribe(sender: ModuleId, event_type: EventType) -> Self {
        Self {
            id: format!(
                "unsubscribe-{}-{}",
                event_type.to_string(),
                chrono::Utc::now().timestamp_millis()
            ),
            message_type: MessageType::Unsubscribe,
            sender,
            receiver: "system".to_string(),
            event_type: Some(event_type),
            data: NargoValue::Null,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(3000),
            retry_count: 0,
        }
    }

    /// 创建错误消息
    pub fn new_error(sender: ModuleId, receiver: ModuleId, error: String) -> Self {
        let data = NargoValue::Object({
            let mut map = HashMap::new();
            map.insert("error".to_string(), NargoValue::String(error));
            map
        });
        Self {
            id: format!("error-{}", chrono::Utc::now().timestamp_millis()),
            message_type: MessageType::Error,
            sender,
            receiver,
            event_type: None,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            status: MessageStatus::Created,
            timeout: Some(3000),
            retry_count: 0,
        }
    }
}

/// 消息批处理结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// 批次ID
    pub batch_id: String,
    /// 消息列表
    pub messages: Vec<Message>,
    /// 批次大小
    pub size: usize,
    /// 时间戳
    pub timestamp: u64,
}

/// 模块状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleStatus {
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 已初始化
    Initialized,
    /// 运行中
    Running,
    /// 出错
    Error,
}

/// 模块信息结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// 模块名称
    pub name: String,
    /// 模块版本
    pub version: String,
    /// 模块状态
    pub status: ModuleStatus,
}

/// 补全上下文枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionContext {
    Tag,
    Attribute(String),
    Expression,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入模式
    pub input_schema: ToolInputSchema,
}

/// 工具输入模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    /// 模式类型
    pub schema_type: String,
    /// 属性定义
    pub properties: Option<HashMap<String, PropertySchema>>,
    /// 必需属性
    pub required: Option<Vec<String>>,
}

/// 属性模式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PropertySchema {
    /// 属性类型
    pub prop_type: Option<String>,
    /// 属性描述
    pub description: Option<String>,
    /// 数组项类型
    pub items: Option<Box<PropertySchema>>,
    /// 枚举值
    pub enum_values: Option<Vec<String>>,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// 结果内容
    pub content: Vec<Content>,
    /// 是否为错误
    pub is_error: Option<bool>,
}

/// 内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    /// 文本内容
    Text {
        /// 文本内容
        text: String,
    },
    /// 图像内容
    Image {
        /// 图像数据
        data: String,
        /// MIME 类型
        mime_type: String,
    },
    /// 资源内容
    Resource {
        /// 资源 URI
        uri: String,
        /// MIME 类型
        mime_type: Option<String>,
        /// 文本内容
        text: Option<String>,
    },
}
