use std::sync;
use std::thread;
use std::collections::HashMap;


mod complex;
mod format;


pub use self::complex::Configuration;
pub use self::format::MetricFormat;


pub type ConfigurationMap       = HashMap<String, String>;
pub type ConfigurationMutex     = sync::Arc<sync::Mutex<ConfigurationMap>>; // TODO: Change Mutex to RwLock
