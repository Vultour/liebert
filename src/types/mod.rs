use std::sync;
use std::collections::HashMap;

mod message;
pub mod complex;


pub use self::message::Message;
pub use self::message::MessageSender;
pub use self::message::MessageReceiver;


pub type ConfigurationMap       = HashMap<String, String>;
pub type ConfigurationMutex     = sync::Arc<sync::Mutex<ConfigurationMap>>;

pub type NamedSenderHashMap     = HashMap<String, sync::mpsc::Sender<Message>>;
