mod builtin_cpu;
mod builtin_memory;

use std::thread;
use std::sync;
use std::time;

use ::types;


pub type PluginTuple = (String, types::MessageSender, thread::JoinHandle<()>);


pub enum Format {
    Gauge(String, u64, Option<i32>, Option<i32>),  // Name, heartbeat, min, max
    Counter(String, u64, Option<i32>, Option<i32>) // Name, heartbeat, min, max
}


pub struct Controller{
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: types::MessageSender
}


impl Controller{
    pub fn new(conf: types::complex::Configuration, control_tx: types::MessageSender) -> Result<Controller, String>{
        let (pipe_tx, pipe_rx) = sync::mpsc::channel();
        match thread::Builder::new().name(String::from("plugins_controller")).spawn(
            move || {
                let control_tx = control_tx;
                let plugin_rx = pipe_rx;

                let default_plugins = start_default_plugins(conf.clone(), control_tx.clone());
                start_extra_plugins(conf.clone(), control_tx.clone());

                for message in plugin_rx{
                    match message{
                        types::Message::Shutdown(m) => {
                            debug!("Plugins thread received a shutdown command: {}", m);
                            for (name, plugin) in default_plugins {
                                debug!("Trying to shutdown '{}'", name);
                                plugin.0.send(types::Message::Shutdown("Global shutdown".into()));
                                plugin.1.join();
                            }
                            return;
                        }
                        _                           => { error!("[BUG] Plugins thread received unknown message: {}", message); }
                    }
                }
            }
        ){
            Ok(h)   => { return Ok(Controller{ thread_handle: h, channel_in: pipe_tx }); }
            Err(e)  => { return Err(format!("{}", e)); }
        }
    }
}


fn start_default_plugins(c: types::complex::Configuration, control_tx: types::MessageSender) -> types::NamedSenderHashMap{
    let mut plugin_channels = types::NamedSenderHashMap::new();

    match builtin_cpu::start_builtin_cpu(c.clone(), control_tx.clone()) {
        Ok(pt) => { plugin_channels.insert(pt.0, (pt.1, pt.2)); },
        Err(e) => { error!("Couldn't start builtin CPU collector: {}", e); }
    }

    match builtin_memory::start_builtin_memory(c.clone(), control_tx.clone()) {
        Ok(pt) => { plugin_channels.insert(pt.0, (pt.1, pt.2)); },
        Err(e) => { error!("Couldn't start builtin memory collector: {}", e); }
    }

    plugin_channels
}

fn start_extra_plugins(c: types::complex::Configuration, control_tx: types::MessageSender) -> types::NamedSenderHashMap{
    let mut plugin_channels = types::NamedSenderHashMap::new();

    plugin_channels
}


fn is_shutdown(message: Result<types::Message, sync::mpsc::TryRecvError>) -> bool{
    match message{
        Err(e)  => {
            match e{
                sync::mpsc::TryRecvError::Disconnected  => { panic!("FATAL ERROR: [BUG] Plugin sender disconnected"); },
                sync::mpsc::TryRecvError::Empty         => { }
            }
        },
        Ok(msg) =>{
            match msg{
                types::Message::Shutdown(m) => { return true; }
                _                           => { warn!("[BUG] Plugin received an uknown message, it will be ignored"); }
            }
        }
    }

    false
}
