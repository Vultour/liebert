use std::sync;
use std::thread;
use std::collections::HashMap;


mod complex;


pub use self::complex::Configuration;


pub type ConfigurationMap       = HashMap<String, String>;
pub type ConfigurationMutex     = sync::Arc<sync::Mutex<ConfigurationMap>>; // TODO: Change Mutex to RwLock
