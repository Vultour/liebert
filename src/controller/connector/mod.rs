use std::net;
use std::time;
use std::sync;
use std::thread;
use std::io::BufReader;
use std::io::BufWriter;

use std::io::Write;
use std::io::BufRead;
use std::str::FromStr;
use std::fmt::Display;

use ::util;
use ::types;


mod decoder;


pub type    ConnectorMessageSender   = sync::mpsc::Sender<super::Message>;
type        ConnectorMessageReceiver = sync::mpsc::Receiver<super::Message>;
type        ConnectorThreadTuple     = (ConnectorMessageSender, thread::JoinHandle<()>);
type        ConnectorMutexedReceiver = sync::Arc<sync::Mutex<ConnectorMessageReceiver>>;


pub struct Connector {
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: ConnectorMessageSender
}


impl Connector {
    pub fn new(conf: types::Configuration, tx: super::MessageSender) -> Result<Connector, String> {
        let (pipe_tx, pipe_rx) = sync::mpsc::channel::<super::Message>();
        let pipe_tx_clone = pipe_tx.clone();
        match thread::Builder::new().name(String::from("connector")).spawn(
            move || {
                let connector_rx = pipe_rx;
                let control_tx = tx;
                //let (data_queue_tx, data_queue_rx) = sync::mpsc::channel();
                //let data_queue_rx = sync::Arc::new(sync::Mutex::new(data_queue_rx));
                let host = conf.get_unsafe("controller.host");
                let port = u32::from_str(&conf.get_unsafe("controller.port")).expect("FATAL ERROR: Couldn't convert controller.port to an integer");
                match thread::Builder::new().name("connector_listener".into()).spawn(
                    move || {
                        let listener = match net::TcpListener::bind(format!("{}:{}", host, port).as_str()) {
                            Ok(l)  => l,
                            Err(e) => panic!("Couldn't open network listener - {}", e)
                        };

                        debug!("Starting connector listener");

                        for stream in listener.incoming() {
                            match stream {
                                Ok(s)  => {
                                    handle_read(
                                        s.try_clone().expect("FATAL ERROR: Couldn't copy TcpStream"),
                                        control_tx.clone()
                                    );
                                    handle_write(
                                        s,
                                        pipe_tx.clone()
                                    );
                                },
                                Err(e) => error!("Error processing incoming connection - {}", e)
                            };
                        }

                        panic!("FATAL ERROR: Connector thread quit unexpectedly");
                    }
                ) {
                    Ok(h)  => h,
                    Err(e) => panic!("FATAL ERROR: Couldn't spawn connector listener thread")
                };

                loop {
                    match connector_rx.recv() {
                        Ok(msg) => {
                            match msg {
                                super::Message::Data(n, t, m)    => {
                                    debug!("Connector received data - {} [{}]", n, t);
                                },
                                super::Message::Format(n, m)  => {
                                    debug!("Connector received format for {}", n);
                                },
                                super::Message::Shutdown(m)     => {
                                    debug!("Connector received shutdown command - {}", m);
                                    break;
                                }
                            }
                        },
                        Err(e)  => {
                            error!("FATAL ERROR: [Bug] Connector could not read on its channel - {}", e);
                            break;
                        }
                    }
                }
            }
        ) {
            Ok(t)   => { return Ok(Connector{ thread_handle: t, channel_in: pipe_tx_clone }); }
            Err(e)  => { return Err(format!("{}", e)); }
        }
    }
}


fn handle_read(stream: net::TcpStream, control_pipe: ConnectorMessageSender) {
    match thread::Builder::new().name("connector_reader".into()).spawn(
        move || {
            let mut reader = BufReader::new(stream);

            debug!("Connector reader started");

            loop {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer) {
                    Ok(n)  => {
                        if n < 1 {
                            warn!("Connector reader remote end disconnected");
                            return;
                        }

                        control_pipe.send(decoder::decode(&buffer));
                    }
                    Err(e) => panic!("FATAL ERROR: Connector reader couldn't read line - {}", e)
                }

                debug!("Received a message from an agent: '{}'", buffer);
            }
        }
    ) {
        Ok(_)  => { },
        Err(e) => error!("FATAL ERROR: Couldn't spawn connector reading thread")
    }
}

fn handle_write(stream: net::TcpStream, connector_pipe: ConnectorMessageSender) -> thread::JoinHandle<()> {
    match thread::Builder::new().name("connector_writer".into()).spawn(
        move || {
            let mut writer = BufWriter::new(stream);

            //loop {
            //    match data_queue.recv() {
            //        Ok(msg) => {
            //            match msg { // TODO: Handle writer failure - return data back to queue
            //                super::Message::Shutdown(m) => {
            //                    debug!("Connector writer received shutdown command");
            //                    return;
            //                },
            //                _ => { error!("[Bug] unknown command received on connector writer channel"); }
            //            }
            //        },
            //        Err(e) => {
            //            panic!("FATAL ERROR: [Bug] Controller writer sender pipe disconnected");
            //        }
            //    } 
            //}

            debug!("Connector writer stopped");
        }
    ) {
        Ok(t)  => return t,
        Err(e) => panic!("FATAL ERROR: Couldn't spawn connector reading thread")
    }
}
