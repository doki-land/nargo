#![feature(new_range_api)]
#![warn(missing_docs)]
use dashmap::{DashMap, DashSet};
use nargo_types::NargoValue;
use std::collections::HashMap;
use tokio::sync::mpsc as tokio_mpsc;

use crate::types::*;

/// 模块间通信管理器
#[derive(Clone)]
pub struct ModuleCommunicationManager {
    /// 模块注册映射
    modules: DashMap<ModuleId, ModuleInfo>,
    /// 消息通道映射
    message_channels: DashMap<ModuleId, tokio_mpsc::Sender<Message>>,
    /// 事件监听器映射
    event_listeners: DashMap<EventType, Vec<ModuleId>>,
    /// 消息状态跟踪
    message_status: DashMap<MessageId, Message>,
    /// 心跳状态
    heartbeat_status: DashMap<ModuleId, u64>,
    /// 心跳间隔（毫秒）
    heartbeat_interval: u64,
    /// 消息批处理缓冲区
    message_buffers: DashMap<ModuleId, Vec<Message>>,
    /// 批处理大小阈值
    batch_size_threshold: usize,
    /// 批处理间隔（毫秒）
    batch_interval: u64,
    /// 消息缓存
    message_cache: DashMap<String, NargoValue>,
    /// 缓存大小限制
    cache_size_limit: usize,
}

unsafe impl Send for ModuleCommunicationManager {}
unsafe impl Sync for ModuleCommunicationManager {}

impl ModuleCommunicationManager {
    /// 创建新的模块通信管理器
    pub fn new() -> Self {
        Self {
            modules: DashMap::new(),
            message_channels: DashMap::new(),
            event_listeners: DashMap::new(),
            message_status: DashMap::new(),

            heartbeat_status: DashMap::new(),
            heartbeat_interval: 30000,
            message_buffers: DashMap::new(),
            batch_size_threshold: 10,
            batch_interval: 100,
            message_cache: DashMap::new(),
            cache_size_limit: 1000,
        }
    }

    /// 注册模块
    pub fn register_module(
        &self,
        module_id: ModuleId,
        name: String,
        version: String,
    ) -> tokio_mpsc::Receiver<Message> {
        let (tx, rx) = tokio_mpsc::channel(100);

        let name_clone = name.clone();
        let version_clone = version.clone();

        self.modules.insert(
            module_id.clone(),
            ModuleInfo {
                name,
                version,
                status: ModuleStatus::Uninitialized,
            },
        );

        self.message_channels.insert(module_id.clone(), tx);

        self.heartbeat_status.insert(
            module_id.clone(),
            chrono::Utc::now().timestamp_millis() as u64,
        );

        let event_data = NargoValue::Object({
            let mut map = HashMap::new();
            map.insert(
                "module_id".to_string(),
                NargoValue::String(module_id.clone()),
            );
            map.insert("name".to_string(), NargoValue::String(name_clone));
            map.insert("version".to_string(), NargoValue::String(version_clone));
            map
        });
        let manager = self.clone();
        tokio::spawn(async move {
            manager
                .broadcast_event(EventType::ModuleRegistered, event_data, "system".to_string())
                .await;
        });

        rx
    }

    /// 发送消息
    pub async fn send_message(&self, mut message: Message) {
        self.message_status.insert(message.id.clone(), message.clone());

        message.status = MessageStatus::Sent;
        self.message_status.insert(message.id.clone(), message.clone());

        let receiver = message.receiver.clone();

        let tx = self.message_channels.get(&receiver).map(|r| r.value().clone());

        if let Some(tx) = tx {
            if tx.send(message.clone()).await.is_err() {
                self.message_channels.remove(&receiver);
                self.modules.remove(&receiver);
                self.heartbeat_status.remove(&receiver);

                let mut failed_msg = message;
                failed_msg.status = MessageStatus::Failed;
                self.message_status.insert(failed_msg.id.clone(), failed_msg);

                let event_data = NargoValue::Object({
                    let mut map = HashMap::new();
                    map.insert("module_id".to_string(), NargoValue::String(receiver));
                    map
                });
                let event_message = Message::new_event(
                    "system".to_string(),
                    EventType::ModuleUnregistered,
                    event_data,
                );
                self.queue_message(event_message);
            } else {
                let event_data = NargoValue::Object({
                    let mut map = HashMap::new();
                    map.insert(
                        "message_id".to_string(),
                        NargoValue::String(message.id),
                    );
                    map.insert(
                        "sender".to_string(),
                        NargoValue::String(message.sender),
                    );
                    map.insert(
                        "receiver".to_string(),
                        NargoValue::String(message.receiver),
                    );
                    map
                });
                let event_message =
                    Message::new_event("system".to_string(), EventType::MessageSent, event_data);
                self.queue_message(event_message);
            }
        } else {
            message.status = MessageStatus::Failed;
            self.message_status.insert(message.id.clone(), message);
        }
    }

    /// 广播事件
    pub async fn broadcast_event(
        &self,
        event_type: EventType,
        data: NargoValue,
        sender: ModuleId,
    ) {
        let message = Message::new_event(sender, event_type.clone(), data);

        let listeners = {
            self.event_listeners
                .get(&event_type)
                .map(|l| l.value().clone())
        };

        if let Some(listeners) = listeners {
            for module_id in listeners {
                let mut msg = message.clone();
                msg.receiver = module_id;
                self.send_message(msg).await;
            }
        }
    }

    /// 注册事件监听器
    pub fn register_event_listener(&self, event_type: EventType, module_id: ModuleId) {
        self.event_listeners
            .entry(event_type)
            .and_modify(|listeners| {
                if !listeners.contains(&module_id) {
                    listeners.push(module_id.clone());
                }
            })
            .or_insert_with(|| vec![module_id]);
    }

    /// 移除事件监听器
    pub fn remove_event_listener(&self, event_type: EventType, module_id: &ModuleId) {
        if let Some(mut listeners) = self.event_listeners.get_mut(&event_type) {
            listeners.retain(|id| id != module_id);
        }
    }

    /// 更新模块状态
    pub fn update_module_status(&self, module_id: &ModuleId, status: ModuleStatus) {
        if let Some(mut module) = self.modules.get_mut(module_id) {
            module.status = status;
        }
    }

    /// 获取模块状态
    pub fn get_module_status(&self, module_id: &ModuleId) -> Option<ModuleStatus> {
        self.modules
            .get(module_id)
            .map(|module| module.status.clone())
    }

    /// 列出所有模块
    pub fn list_modules(&self) -> Vec<(ModuleId, ModuleInfo)> {
        self.modules
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// 确认消息
    pub async fn acknowledge_message(&self, message_id: &MessageId) {
        if let Some(mut msg) = self.message_status.get_mut(message_id) {
            msg.status = MessageStatus::Acknowledged;

            let event_data = NargoValue::Object({
                let mut map = HashMap::new();
                map.insert(
                    "message_id".to_string(),
                    NargoValue::String(message_id.clone()),
                );
                map
            });
            self.broadcast_event(EventType::MessageReceived, event_data, "system".to_string())
                .await;
        }
    }

    /// 发送消息并等待确认
    pub async fn send_message_with_ack(&self, message: Message) -> Result<Message, String> {
        let timeout = message.timeout.unwrap_or(5000);

        self.send_message(message).await;

        Err(format!("Message acknowledgement not supported, timeout: {}", timeout))
    }

    /// 启动心跳检测
    pub async fn start_heartbeat(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    manager.heartbeat_interval,
                ))
                .await;
                manager.check_heartbeats().await;
            }
        });
    }

    /// 检查心跳状态
    async fn check_heartbeats(&self) {
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut modules_to_remove = Vec::new();

        for entry in self.heartbeat_status.iter() {
            let module_id = entry.key();
            let last_heartbeat = entry.value();

            if current_time - *last_heartbeat > self.heartbeat_interval * 2 {
                modules_to_remove.push(module_id.clone());
            }
        }

        for module_id in modules_to_remove {
            self.message_channels.remove(&module_id);
            self.modules.remove(&module_id);
            self.heartbeat_status.remove(&module_id);
            self.message_buffers.remove(&module_id);

            let event_data = NargoValue::Object({
                let mut map = HashMap::new();
                map.insert("module_id".to_string(), NargoValue::String(module_id));
                map
            });
            let event_message = Message::new_event(
                "system".to_string(),
                EventType::HeartbeatTimeout,
                event_data,
            );
            self.queue_message(event_message);
        }
    }

    /// 发送心跳消息
    pub async fn send_ping(&self, module_id: &ModuleId) {
        let ping_message = Message::new_ping("system".to_string(), module_id.clone());
        self.send_message(ping_message).await;
    }

    /// 处理心跳响应
    pub async fn handle_pong(&self, module_id: &ModuleId) {
        self.heartbeat_status.insert(
            module_id.clone(),
            chrono::Utc::now().timestamp_millis() as u64,
        );
    }

    /// 获取消息状态
    pub fn get_message_status(&self, message_id: &MessageId) -> Option<MessageStatus> {
        self.message_status.get(message_id).map(|msg| msg.status.clone())
    }

    /// 清理过期消息
    pub fn cleanup_expired_messages(&self) {
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut messages_to_remove = Vec::new();

        for entry in self.message_status.iter() {
            let message_id = entry.key();
            let message = entry.value();

            if message.status == MessageStatus::Acknowledged
                || (message.status == MessageStatus::Timeout
                    && current_time - message.timestamp > 300000)
            {
                messages_to_remove.push(message_id.clone());
            }
        }

        for message_id in messages_to_remove {
            self.message_status.remove(&message_id);
        }
    }

    /// 发送消息批次
    pub async fn send_message_batch(&self, batch: MessageBatch) {
        for message in batch.messages {
            self.send_message(message).await;
        }
    }

    /// 将消息加入批处理队列
    pub fn queue_message(&self, message: Message) {
        self.message_buffers
            .entry(message.receiver.clone())
            .and_modify(|buffer| {
                buffer.push(message.clone());
                if buffer.len() >= self.batch_size_threshold {
                    let batch = MessageBatch {
                        batch_id: format!("batch-{}", chrono::Utc::now().timestamp_millis()),
                        messages: buffer.drain(..).collect(),
                        size: buffer.len(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    let manager = self.clone();
                    tokio::spawn(async move {
                        manager.send_message_batch(batch).await;
                    });
                }
            })
            .or_insert_with(|| vec![message]);
    }

    /// 处理消息批次
    pub async fn process_message_batches(&self) {
        let mut batches = Vec::new();

        for mut entry in self.message_buffers.iter_mut() {
            let module_id = entry.key().clone();
            let buffer = entry.value_mut();

            if !buffer.is_empty() {
                let batch = MessageBatch {
                    batch_id: format!(
                        "batch-{}-{}",
                        module_id,
                        chrono::Utc::now().timestamp_millis()
                    ),
                    messages: buffer.drain(..).collect(),
                    size: buffer.len(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                };
                batches.push(batch);
            }
        }

        for batch in batches {
            self.send_message_batch(batch).await;
        }
    }

    /// 启动批处理器
    pub async fn start_batch_processor(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(manager.batch_interval))
                    .await;
                manager.process_message_batches().await;
            }
        });
    }

    /// 缓存消息
    pub fn cache_message(&self, key: String, value: NargoValue) {
        if self.message_cache.len() >= self.cache_size_limit {
            if let Some(first_key) = self.message_cache.iter().next() {
                self.message_cache.remove(first_key.key());
            }
        }
        self.message_cache.insert(key, value);
    }

    /// 获取缓存的消息
    pub fn get_cached_message(&self, key: &str) -> Option<NargoValue> {
        self.message_cache.get(key).and_then(|v| Some(v.clone()))
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        self.message_cache.clear();
    }

    /// 设置批处理大小阈值
    pub fn set_batch_size_threshold(&mut self, threshold: usize) {
        self.batch_size_threshold = threshold;
    }

    /// 设置批处理间隔
    pub fn set_batch_interval(&mut self, interval: u64) {
        self.batch_interval = interval;
    }

    /// 设置缓存大小限制
    pub fn set_cache_size_limit(&mut self, limit: usize) {
        self.cache_size_limit = limit;
    }
}
