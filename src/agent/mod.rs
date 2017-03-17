use std::sync;


mod message;

pub mod connector;
pub mod plugins;


pub use self::message::Message;


pub type MessageSender = sync::mpsc::Sender<Message>;