extern crate log;
extern crate env_logger;
extern crate time;


pub fn init(log_level: log::LogLevelFilter){
    env_logger::LogBuilder::new()
        .format(
            |r: &log::LogRecord|{
                format!(
                    "{}.{} [{:<5}]: {}",
                    time::strftime("%Y-%m-%d %H:%I:%S", &time::now()).unwrap(),
                    &time::now().tm_nsec.to_string()[0..5],
                    //time::now().tm_nsec.to_string().chars().take(5).collect::<String>(), // Maybe there's a nicer way?
                    r.level(),
                    r.args())

            }
        )
        .filter(None, log_level)
        .init().expect("FATAL ERROR: [BUG] Failed to initialize logger");
}
