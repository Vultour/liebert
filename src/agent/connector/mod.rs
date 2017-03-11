use std::net;
use std::time;
use std::sync;
use std::thread;

use std::str::FromStr;

use ::util;
use ::types;


pub struct Connector{
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: types::MessageSender
}


impl Connector{
    pub fn new(conf: types::complex::Configuration, tx: types::MessageSender) -> Result<Connector, String>{
        let (pipe_tx, pipe_rx) = sync::mpsc::channel();
        match thread::Builder::new().name(String::from("connector")).spawn(
            move || {
                let connector_rx = pipe_rx;
                let control_tx = tx;
                let mut s: net::TcpStream;
                let mut retries: i32 = -1;
                loop {
                    retries += 1;
                    let host = &conf.get_unsafe("controller.host");
                    let port = i32::from_str(&conf.get_unsafe("controller.port")).expect("FATAL ERROR: Couldn't convert 'controller.port' to an integer");
                    let timeout: i64 = i64::from_str(&conf.get_unsafe("controller.retry_timeout")).expect("FATAL ERROR: Couldn't convert 'controller.retry_timeout to an integer'");
                    match net::TcpStream::connect(format!("{}:{}", host, port).as_str()){
                        Ok(stream)   => {
                            s = stream;
                            break;
                        }
                        Err(e)  => {
                            let max_retries: i32 = i32::from_str(&conf.get_unsafe("controller.max_retries")).expect("FATAL ERROR: Coulnd't convert 'controller.max_retries to an integer'");
                            warn!("Couldn't connect to controller instance at '{}:{}', retry number {} - {}", host, port, retries,  e);

                            if (retries >= max_retries) && (max_retries > 0){
                                error!("[connector] Controller connection could not be established in configured number of retries, shutting down");
                                control_tx.send(types::Message::Shutdown(format!("[connector] Couldn't establish a connection to controller, giving up after {} retries", retries))).expect("FATAL ERROR: [BUG] Control channel hung up");
                                return;
                            }
                        }
                    }

                    match util::wait_exec_result(
                        time::Duration::from_millis(
                            u64::from_str(
                                &conf.get_unsafe("controller.retry_timeout")
                            ).expect("FATAL ERROR: Couldn't convert 'controller.retry_timeout to an integer'")
                        ),
                        &|| { should_abort(connector_rx.try_recv()) }
                    ){
                        Ok(_)   => {
                            debug!("Connector received shutdown command before initial connection was established");
                            return;
                        },
                        Err(_)  => { }
                    }
                }

                info!("Controller connection established");
            }
        ){
            Ok(t)   => { return Ok(Connector{ thread_handle: t, channel_in: pipe_tx }); }
            Err(e)  => { return Err(format!("{}", e)); }
        }
    }
}


fn should_abort(message: Result<types::Message, sync::mpsc::TryRecvError>) -> bool{
    match message{
        Err(e)  => {
            match e{
                sync::mpsc::TryRecvError::Disconnected  => { panic!("FATAL ERROR: [BUG] Connector sender disconnected"); },
                sync::mpsc::TryRecvError::Empty         => { }
            }
        },
        Ok(msg) =>{
            match msg{
                types::Message::Shutdown(m) => { return true; }
                _                           => { warn!("Connector received a message before it finished initialization, it will be ignored"); }
            }
        }
    }

    false
}
