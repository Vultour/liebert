#[macro_use]
extern crate log;
extern crate toml;
extern crate clap;
extern crate chan_signal;
extern crate regex;
extern crate time;

use std::str::FromStr;

use std::thread;
use std::sync;

mod agent;
mod types;
mod args;
mod conf;
mod logger;
mod util;
mod watchdog;
mod signals;


fn main(){
    let args: types::ConfigurationMap;
    let conf: types::Configuration;

    // Create a control channel
    let (tx_control, rx_control) = sync::mpsc::channel();

    // Parse command line arguments
    match args::agent::get_args(){
        Ok(a)   => { args = a; }
        Err(e)  => { panic!(e); }
    }

    // Parse configuration file
    match conf::agent::from_file(args.get(".args.config").expect("FATAL ERROR: [BUG] Missing configuration file location")){
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
        _ => log::LogLevelFilter::Debug
    };

    // Initialize logging
    logger::init(log_level);

    debug!("Logger initialized with log level {}", log_level);
    info!("Initializing");
    debug!("Loaded configuration: {:#?}", conf);

    // Start the signal handler
    let signal_tx_control = tx_control.clone();
    signals::handle(
        chan_signal::notify(&[chan_signal::Signal::INT, chan_signal::Signal::TERM]),
        move || {
            signal_tx_control.send(agent::Message::Shutdown("Interrupt signal".into()));
        }
    );

    let mut watchdog = watchdog::Watchdog::new();

    // Start controller connector
    let connector = match agent::connector::Connector::new(conf.clone(), tx_control.clone()){
        Ok(c)   => c,
        Err(e)  => panic!(e)
    };

    // Start plugins
    let plugins_controller = match agent::plugins::Controller::new(conf.clone(), tx_control.clone()){
        Ok(pc)  => pc,
        Err(e)  => panic!(e)
    };

    watchdog.watch(connector.thread_handle);
    watchdog.watch(plugins_controller.thread_handle);

    match watchdog.monitor(){
        Ok(_)   => { }
        Err(e)  => { panic!("FATAL ERROR: Watchdog error - {}", e); }
    }

    loop{
        match rx_control.recv(){
            Ok(msg)   => {
                match msg{
                    agent::Message::Fatal(m)        => {
                        error!("Control channel received a message indicating an unrecoverable error: {}", m);
                        panic!("Fatal error on control channel, see relevant log message");
                    },
                    agent::Message::Shutdown(m)     => {
                        info!("Received shutdown message on control channel - {}", m);
                        info!("Initiating shutdown");

                        match plugins_controller.channel_in.send(agent::Message::Shutdown(String::from("Global shutdown"))){
                            Ok(_)   => {},
                            Err(e)  => error!("There has been an error while trying to stop 'plugins_controller', if there is a previous error related to this module you can probably disregard this message. The error received was: {}", e)
                        }

                        match connector.channel_in.send(agent::connector::Message::Shutdown(String::from("Global shutdown"))){
                            Ok(_)   => {},
                            Err(e)  => error!("There has been an error while trying to stop 'connector', if there is a previous error related to this module you can probably disregard this message. The error received was: {}", e)
                        }

                        break;
                    },
                    agent::Message::LogError(m)     => { error!("{}", m); },
                    agent::Message::LogWarn(m)      => { warn!("{}", m); },
                    agent::Message::LogInfo(m)      => { info!("{}", m); },
                    agent::Message::LogDebug(m)     => { debug!("{}", m); },
                    agent::Message::Format(n, f)    => {
                        debug!("Format received on control channel for {}, see below", n);
                        for format in &f {
                            match format {
                                &::types::MetricFormat::Gauge(ref name, ref hb, ref min, ref max)   => {
                                    debug!(
                                        "FORMAT: {} - {}s -- min:{} -- max:{}",
                                        name, hb, min.unwrap_or(-1), max.unwrap_or(-1)
                                    );
                                },
                                &::types::MetricFormat::Counter(ref name, ref hb, ref min, ref max) => {
                                    debug!(
                                        "FORMAT: {} - {}s -- min:{} -- max:{}",
                                        name, hb, min.unwrap_or(-1), max.unwrap_or(-1)
                                    );
                                }
                            }
                        }
                        connector.channel_in.send(agent::connector::Message::Format(n, f));
                    },
                    agent::Message::Data(id, t, m)   => {
                        debug!(
                            "Data received on control channel from plugin '{}' [{}]: {}",
                            id, t, m
                        );
                        connector.channel_in.send(agent::connector::Message::Data(id, t, m));
                    }
                }
            },
            Err(_)  => {
                panic!("FATAL ERROR: [BUG] All control channel senders have closed before normal shutdown was initiated");
            }
        }
    }

    match watchdog.join(){
        Ok(_)   => { debug!("All watchdog threads terminated gracefully"); }
        Err(e)  => { panic!("FATAL ERROR: Watchdog error - {}", e); }
    }

    info!("Agent stopped");
}
