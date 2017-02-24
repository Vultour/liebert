use std::thread;
use std::sync;
use std::time;

use ::types;


pub type PluginTuple = (types::MessageSender, thread::JoinHandle)


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

                start_default_plugins(conf.clone(), pipe_tx.clone());
                start_extra_plugins(conf.clone(), pipe_tx.clone());

                for message in plugin_rx{
                    match message{
                        types::Message::Shutdown(m) => {
                            debug!("Plugins thread received a shutdown message: {}", m);
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


fn start_default_plugins(c: types::complex::Configuration, data_tx: types::MessageSender) -> types::NamedSenderHashMap{
    let plugin_channels = types::NamedSenderHashMap::new();
    
    

    plugin_channels
}

fn start_extra_plugins(c: types::complex::Configuration, data_tx: types::MessageSender) -> types::NamedSenderHashMap{
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
