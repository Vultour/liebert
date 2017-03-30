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


pub type    ConnectorMessageSender   = sync::mpsc::Sender<Message>;
type        ConnectorMessageReceiver = sync::mpsc::Receiver<Message>;
type        ConnectorThreadTuple     = (ConnectorMessageSender, thread::JoinHandle<()>);
type        ConnectorMutexedReceiver = sync::Arc<sync::Mutex<ConnectorMessageReceiver>>;


pub enum Message {
    Data(String, i64, String),
    Format(String, Vec<::types::MetricFormat>),
    Shutdown(String),
    Placeholder
}


pub struct Connector {
    pub thread_handle: thread::JoinHandle<()>,
    pub channel_in: ConnectorMessageSender
}


impl Connector {
    pub fn new(conf: types::Configuration, tx: super::MessageSender) -> Result<Connector, String> {
        let (pipe_tx, pipe_rx) = sync::mpsc::channel::<Message>();
        let pipe_tx_clone = pipe_tx.clone();
        match thread::Builder::new().name(String::from("connector")).spawn(
            move || {
                let connector_rx = pipe_rx;
                let control_tx = tx;
                let (data_queue_tx, data_queue_rx) = sync::mpsc::channel();
                let data_queue_rx = sync::Arc::new(sync::Mutex::new(data_queue_rx));
                let mut s: net::TcpStream;
                let mut retries: i32 = -1;
                loop {
                    retries += 1;
                    let host = &conf.get_unsafe("controller.host");
                    let port = i32::from_str(&conf.get_unsafe("controller.port")).expect("FATAL ERROR: Couldn't convert 'controller.port' to an integer");
                    let timeout: i64 = i64::from_str(&conf.get_unsafe("controller.retry_timeout")).expect("FATAL ERROR: Couldn't convert 'controller.retry_timeout to an integer'");
                    match net::TcpStream::connect(format!("{}:{}", host, port).as_str()) {
                        Ok(stream)   => {
                            s = stream;
                            info!("Controller connection established with {}:{}", host, port);
                            break;
                        }
                        Err(e)  => {
                            let max_retries: i32 = i32::from_str(&conf.get_unsafe("controller.max_retries")).expect("FATAL ERROR: Coulnd't convert 'controller.max_retries to an integer'");
                            warn!("Couldn't connect to controller instance at '{}:{}', retry number {} - {}", host, port, retries,  e);

                            if (retries >= max_retries) && (max_retries > 0){
                                error!("[connector] Controller connection could not be established in configured number of retries, shutting down");
                                control_tx.send(super::Message::Shutdown(format!("[connector] Couldn't establish a connection to controller, giving up after {} retries", retries))).expect("FATAL ERROR: [BUG] Control channel hung up");
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
                    ) {
                        Ok(_)   => {
                            debug!("Connector received shutdown command before initial connection was established");
                            return;
                        },
                        Err(_)  => { }
                    }
                }

                let (reader_pipe, reader_handle) = start_reader(
                    s.try_clone().expect("FATAL ERROR: Couldn't copy TCP stream for reading"),
                    pipe_tx.clone()
                );
                let writer_handle = start_writer(
                    s.try_clone().expect("FATAL ERROR: Couldn't copy TCP stream for writing"),
                    data_queue_rx.clone(),
                    pipe_tx.clone()
                );

                loop {
                    match connector_rx.recv() {
                        Ok(msg) => {
                            match msg {
                                Message::Data(n, t, m)    => {
                                    debug!("Connector received data - {} [{}]: {}", n, t, m);
                                    data_queue_tx.send(Message::Data(n, t, m));
                                },
                                Message::Format(n, m)  => {
                                    debug!("Connector received format for {}", n);
                                    data_queue_tx.send(Message::Format(n, m));
                                },
                                Message::Shutdown(m)     => {
                                    debug!("Connector received shutdown command - {}", m);
                                    s.shutdown(net::Shutdown::Both);
                                    data_queue_tx.send(Message::Shutdown(m.clone()));
                                    reader_pipe.send(Message::Shutdown(m.clone()));
                                    writer_handle.join();
                                    reader_handle.join();
                                    break;
                                },
                                _ => warn!("Connector received an unprocessed message")
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


fn start_reader(stream: net::TcpStream, connector_pipe: ConnectorMessageSender) -> ConnectorThreadTuple {
    let (pipe_tx, pipe_rx) = sync::mpsc::channel();
    match thread::Builder::new().name("con-reader".into()).spawn(
        move || {
            let mut reader = BufReader::new(stream);
            let pipe = pipe_rx;

            debug!("Connector reader started");

            loop {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer) {
                    Ok(n)  => {
                        if n < 1 {
                            warn!("Connector reader remote end disconnected");
                            return;
                        }
                    }
                    Err(e) => panic!("FATAL ERROR: Connector reader couldn't read line - {}", e)
                }

                debug!("Received a message from controller: '{}'", buffer);
            }
        }
    ) {
        Ok(t)  => return (pipe_tx, t),
        Err(e) => panic!("FATAL ERROR: Couldn't spawn connector reading thread")
    }
}

fn start_writer(stream: net::TcpStream, data_queue: ConnectorMutexedReceiver, connector_pipe: ConnectorMessageSender) -> thread::JoinHandle<()> {
    match thread::Builder::new().name("con-writer".into()).spawn(
        move || {
            let mut writer = BufWriter::new(stream);
            let data_queue = match data_queue.lock() {
                Ok(g)   => g,
                Err(pg) => pg.into_inner()
            };

            loop {
                match data_queue.recv() {
                    Ok(msg) => {
                        match msg { // TODO: Handle writer failure - return data back to queue
                            Message::Data(n, t, m) => {
                                writer.write_fmt(format_args!(
                                    "DATA {} {} {}\n",
                                    n, t, m
                                ));
                                writer.flush();
                            },
                            Message::Format(n, f) => {
                                let mut format_string = String::from(format!("FORMAT {}", n));
                                for format in f {
                                    let format_id = format.to_id();
                                    match format{
                                        ::types::MetricFormat::Gauge(name, hb, min, max) => {
                                            format_string += &format!(
                                                "\n{} {} {} {} {}",
                                                format_id,
                                                name,
                                                hb,
                                                rrd_unwrap(min),
                                                rrd_unwrap(max)
                                            );
                                        },
                                        ::types::MetricFormat::Counter(name, hb, min, max) => {
                                            format_string += &format!(
                                                "\n{} {} {} {} {}",
                                                format_id,
                                                name,
                                                hb,
                                                rrd_unwrap(min),
                                                rrd_unwrap(max)
                                            );
                                        }
                                    }
                                }
                                format_string += &format!("\nFORMAT_END\n");

                                writer.write_fmt(format_args!("{}", format_string));
                                writer.flush();
                            },
                            Message::Shutdown(m) => {
                                debug!("Connector writer received shutdown command");
                                return;
                            },
                            _ => { error!("[Bug] unknown command received on connector writer channel"); }
                        }
                    },
                    Err(e) => {
                        panic!("FATAL ERROR: [Bug] Controller writer sender pipe disconnected");
                    }
                }
            }

            debug!("Connector writer started");
        }
    ) {
        Ok(t)  => return t,
        Err(e) => panic!("FATAL ERROR: Couldn't spawn connector reading thread")
    }
}


fn rrd_unwrap<T>(o: Option<T>) -> String where T: Display{
    match o {
        Some(x) => format!("{}", x),
        None    => "U".into()
    }
}

fn should_abort(message: Result<Message, sync::mpsc::TryRecvError>) -> bool {
    match message {
        Err(e)  => {
            match e {
                sync::mpsc::TryRecvError::Disconnected  => { panic!("FATAL ERROR: [BUG] Connector sender disconnected"); },
                sync::mpsc::TryRecvError::Empty         => { }
            }
        },
        Ok(msg) => {
            match msg {
                Message::Shutdown(m) => { return true; }
                _                           => { warn!("Connector received a message before it finished initialization, it will be ignored"); }
            }
        }
    }

    false
}
