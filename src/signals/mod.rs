extern crate chan_signal;
extern crate chan;

use std::thread;
use std::sync;

use types;


pub fn handle<T: 'static>(signal_handler: chan::Receiver<chan_signal::Signal>, exec: T) where T: FnOnce() + Send {
    match thread::Builder::new().name(String::from("signal_handler")).spawn(
        move || {
            signal_handler.recv().unwrap();

            debug!("Received an INT or TERM signal");
            exec();

            //match tx_control.send(types::Message::Shutdown(String::from("INT or TERM signal //received"))) {
            //    Ok(_)  => { }
            //    Err(_) => { panic!("FATAL ERROR: [Bug] Control channel was already closed on manual shutdown") }
            //};
        }
    ){
        Err(e)  => { error!("FATAL ERROR: Couldn't spawn a thread for signal handler - {}", e); },
        _       => { }
    }
}
