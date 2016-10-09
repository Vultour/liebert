extern crate chan_signal;
extern crate chan;

use std::thread;
use std::sync;

use types;


pub fn handle(signal_handler: chan::Receiver<chan_signal::Signal>, tx_control: sync::mpsc::Sender<types::Message>) -> Result<thread::JoinHandle<()>, String>{
    match thread::Builder::new().name(String::from("signal_handler")).spawn(
        move || {
            signal_handler.recv().unwrap();

            debug!("Received an INT or TERM signal");
            tx_control.send(types::Message::Shutdown(String::from("INT or TERM signal received")));
        }
    ){
        Ok(h)   => { return Ok(h); },
        Err(e)  => { return Err(format!("FATAL ERROR: Couldn't spawn a thread for signal handler - {}", e)); }
    }
}
