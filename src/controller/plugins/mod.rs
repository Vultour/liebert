use std::thread;
use std::sync;
use std::time;
use std::collections::HashMap;

use ::types;
use super::Message;


mod router;
mod builtin_rrd;


type NamedPluginTuple   = (String, super::MessageSender, thread::JoinHandle<()>);
type PluginTuple        = (super::MessageSender, thread::JoinHandle<()>);
type NamedSenderHashMap = HashMap<String, PluginTuple>;


pub enum Format {
    Counter(String, u32, i64, i64),
    Gauge(String, u32, i64, i64)
}


pub struct Controller{
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: super::MessageSender
}


impl Controller{
    pub fn new(conf: types::Configuration, control_tx: super::MessageSender) -> Result<Controller, String>{
        let (pipe_tx, pipe_rx) = sync::mpsc::channel();
        match thread::Builder::new().name(String::from("plg-controller")).spawn(
            move || {
                let control_tx = control_tx;
                let plugin_rx = pipe_rx;

                let builtin_plugins = start_default_plugins(conf.clone(), control_tx.clone());
                let extra_plugins   = start_extra_plugins(conf.clone(), control_tx.clone());

                let mut router = router::Router::new();

                for (key, value) in builtin_plugins {
                    router.add("builtin.cpu".into(), value.0.clone());
                    router.add("builtin.memory".into(), value.0.clone());
                }

                for message in plugin_rx{
                    match message{
                        Message::Format(ref h, ref m, ref f) => {
                            match router.get_channels(&m) {
                                None    => { },
                                Some(c) => {
                                    for route in c {
                                        route.send(message.clone()).expect("Tried to send format message to a dead plugin");
                                    }
                                }
                            }
                        },
                        Message::Data(ref h, ref m, ref t, ref d) => {
                            match router.get_channels(&m) {
                                None    => { },
                                Some(c) => {
                                    for route in c {
                                        route.send(message.clone()).expect("Tried to send data message to a dead plugin");
                                    }
                                }
                            }
                        },
                        Message::Shutdown(m) => {
                            debug!("Plugins thread received a shutdown command: {}", m);
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

    match builtin_rrd::start_builtin_rrd(c.clone(), control_tx.clone()) {
        Ok(pt) => { plugin_channels.insert(pt.0, (pt.1, pt.2)); },
        Err(e) => { error!("Couldn't start builtin RRD plugin: {}", e); }
    }

    plugin_channels
}

fn start_extra_plugins(c: types::Configuration, control_tx: super::MessageSender) -> NamedSenderHashMap{
    let mut plugin_channels = NamedSenderHashMap::new();

    plugin_channels
}


fn is_shutdown(message: Result<Message, sync::mpsc::TryRecvError>) -> bool{
    match message{
        Err(e)  => {
            match e{
                sync::mpsc::TryRecvError::Disconnected  => { panic!("FATAL ERROR: [BUG] Plugin sender disconnected"); },
                sync::mpsc::TryRecvError::Empty         => { }
            }
        },
        Ok(msg) =>{
            match msg{
                Message::Shutdown(m) => { return true; }
                _                           => { warn!("[BUG] Plugin received an uknown message, it will be ignored"); }
            }
        }
    }

    false
}
