#![feature(new_range_api)]
#![warn(missing_docs)]

pub mod communication;
pub mod language_service;
pub mod server;
/// 模块间通信和语言服务
pub mod types;

pub use communication::ModuleCommunicationManager;
pub use language_service::NargoLanguageService;
pub use server::{run_http_server, run_stdio};
pub use types::{EventType, Message, MessageBatch, MessageId, MessageStatus, MessageType, ModuleId, ModuleInfo, ModuleStatus};
