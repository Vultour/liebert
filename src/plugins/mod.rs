use std::thread;
use std::sync;
use std::time;

use super::types;


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

                let (data_tx, data_rx) = sync::mpsc::channel();

                start_default_plugins(conf.clone(), data_tx.clone());
                start_extra_plugins(conf.clone(), data_tx.clone());

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


fn default_start_cpu(pipe_tx: types::MessageSender) -> Result<types::MessageSender, String>{
    let (pipe_plugin_tx, pipe_plugin_rx) = sync::mpsc::channel();
    match thread::Builder::new().name(String::from("plugins_controller")).spawn(
        move || {
            
        }
    ){
        Ok(t)   => { return Ok(pipe_plugin_tx); }
        Err(e)  => { return Err( format!("{}", e)); }
    }
}
