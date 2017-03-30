mod builtin_cpu;
mod builtin_memory;

use std::thread;
use std::sync;
use std::time;
use std::collections::HashMap;

use ::types;
use ::util;


pub type NamedPluginTuple   = (String, super::MessageSender, thread::JoinHandle<()>);
pub type PluginTuple        = (super::MessageSender, thread::JoinHandle<()>);
pub type NamedSenderHashMap = HashMap<String, PluginTuple>;


pub struct Controller{
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: super::MessageSender
}


impl Controller{
    pub fn new(conf: types::Configuration, control_tx: super::MessageSender) -> Result<Controller, String>{
        let (pipe_tx, pipe_rx) = sync::mpsc::channel();
        match thread::Builder::new().name(String::from("plugins_controller")).spawn(
            move || {
                let control_tx = control_tx;
                let plugin_rx = pipe_rx;

                let default_plugins = start_default_plugins(conf.clone(), control_tx.clone());
                let extra_plugins   = start_extra_plugins(conf.clone(), control_tx.clone());

                for message in plugin_rx{
                    match message{
                        super::Message::Shutdown(m) => {
                            debug!("Plugins thread received a shutdown command: {}", m);
                            for (name, plugin) in default_plugins {
                                debug!("Trying to shutdown '{}'", name);
                                plugin.0.send(super::Message::Shutdown("Global shutdown".into()));
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


fn start_default_plugins(c: types::Configuration, control_tx: super::MessageSender) -> NamedSenderHashMap{
    let mut plugin_channels = NamedSenderHashMap::new();

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

fn start_extra_plugins(c: types::Configuration, control_tx: super::MessageSender) -> NamedSenderHashMap {
    let mut plugin_channels = NamedSenderHashMap::new();

    plugin_channels
}


fn is_shutdown(message: Result<super::Message, sync::mpsc::TryRecvError>) -> bool{
    match message{
        Err(e)  => {
            match e{
                sync::mpsc::TryRecvError::Disconnected  => { panic!("FATAL ERROR: [BUG] Plugin sender disconnected"); },
                sync::mpsc::TryRecvError::Empty         => { }
            }
        },
        Ok(msg) =>{
            match msg{
                super::Message::Shutdown(m) => { return true; }
                _                           => { warn!("[BUG] Plugin received an uknown message, it will be ignored"); }
            }
        }
    }

    false
}
