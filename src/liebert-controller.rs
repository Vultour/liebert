#[macro_use]
extern crate log;
extern crate clap;
extern crate time;
//extern crate regex;
extern crate chan_signal;


use std::str::FromStr;

use std::thread;
use std::sync;


mod controller;
mod args;
mod conf;
mod types;
mod logger;
mod watchdog;
mod signals;
mod util;


fn main(){
    let args: types::ConfigurationMap;
    let conf: types::Configuration;

    // Create a control channel
    let (tx_control, rx_control) = sync::mpsc::channel();

    // Parse command line arguments
    match args::controller::get_args(){
        Ok(a)   => { args = a; }
        Err(e)  => { panic!(e); }
    }

    // Parse configuration file
    match conf::controller::from_file(args.get(".args.config").expect("FATAL ERROR: [BUG] Missing configuration file location")){
        Ok(mut c)   => {
            c.extend(args);
            conf = types::Configuration::new(sync::Arc::new(sync::Mutex::new(c)));
        }
        Err(e)  => { panic!(e); }
    }

    // Determine log level
    let verbosity = u32::from_str(&conf.get_unsafe(".args.verbose")).unwrap();
    let log_level = match verbosity {
        0 => log::LogLevelFilter::Warn,
        1 => log::LogLevelFilter::Info,
        2 => log::LogLevelFilter::Debug,
        _ => log::LogLevelFilter::Trace
    };

    // Initialize logging
    logger::init(log_level);

    debug!("Logger initialized with log level {}", log_level);
    info!("Initializing");
    debug!("Loaded configuration: {:#?}", conf);

    // Start the signal handler
    trace!("Trying to start signal handler");
    let signal_tx_control = tx_control.clone();
    signals::handle(
        chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]),
        move || {
            signal_tx_control.send(controller::Message::Shutdown("Interrupt signal".into()));
        }
    );

    let mut watchdog = watchdog::Watchdog::new();

    trace!("Trying to start connector");
    let connector = match controller::Connector::new(conf.clone(), tx_control.clone()) {
        Ok(c)  => c,
        Err(e) => panic!("FATAL ERROR: Couldn't start connector - {}", e)
    };

    watchdog.watch(connector.thread_handle);

    trace!("Entering main message loop");
    loop {
        match rx_control.recv() {
            Ok(msg) => {
                match msg {
                    controller::Message::Data(n, t, d) => {
                        debug!("Data for {} at {}", n, t);
                    },
                    controller::Message::Format(n, f) => {
                        debug!("Format for {}", n);
                    }
                    controller::Message::Shutdown(m) => {
                        info!("Received shutdown command - {}", m);
                        break;
                    }
                }
            },
            Err(e) => panic!("FATAL ERROR: [Bug] All control channels were closed before normal shutdown was initiated")
        }
    }

    info!("Controller stopped");
}
