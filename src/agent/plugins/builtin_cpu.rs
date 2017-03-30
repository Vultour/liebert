use std::io::prelude::*;
use std::thread;
use std::sync;
use std::time;
use std::io::BufReader;
use std::fs::File;
use regex::Regex;

use time::get_time;

use std::str::FromStr;

use ::types;
use ::util;


pub fn start_builtin_cpu(conf: types::Configuration, control_tx: ::agent::MessageSender) -> Result<super::NamedPluginTuple, String> {
    let (pipe_tx, pipe_rx) = sync::mpsc::channel();
    let id = "builtin.cpu";
    thread::Builder::new().name(String::from("plugin_default_cpu")).spawn(
        move || {
            let control_tx = control_tx;
            let plugin_rx  = pipe_rx;

            let cpu_interval = u32::from_str(
                &conf.get_unsafe("builtin.cpu.interval")
            ).expect("FATAL ERROR: Couldn't convert 'plugins.builtin.cpu.interval' to an integer'");

            if !get_format(control_tx.clone(), cpu_interval, id) {
                error!("There has been an error while starting the CPU collector, it will be disabled");
                return;
            }

            let cpu_interval = cpu_interval * 1000;

            let re = match Regex::new(r"^cpu\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)") {
                Ok(re)  => re,
                Err(e)  => panic!("TODO: Handle regex compilation failure")
            };

            let mut jiffies = match get_current_jiffies(&re){
                Some(j) => j,
                None    => {
                    // TODO: Handle jiffies failure
                    panic!("TODO: Couldn't get jiffies");
                }
            };

            loop {
                match util::wait_exec_result(
                    time::Duration::from_millis(cpu_interval.into()),
                    &|| { super::is_shutdown(plugin_rx.try_recv()) }
                ){
                    Ok(_)   => {
                        debug!("Builtin CPU collector received shutdown command");
                        return;
                    },
                    Err(_)  => { }
                };

                let new_jiffies = match get_current_jiffies(&re) {
                    Some(j) => j,
                    None    => {
                        // TODO: Handle jiffies failure
                        panic!("TODO: Couldn't get jiffies");
                    }
                };

                let idle    = new_jiffies[3] - jiffies[3];
                let user    = new_jiffies[0] - jiffies[0];
                let system  = new_jiffies[2] - jiffies[2];
                let iowait  = new_jiffies[4] - jiffies[4];
                let other   = (new_jiffies[1] + new_jiffies[5] + new_jiffies[6])
                                - (jiffies[1] + jiffies[5] + jiffies[6]);
                let total   = idle + user + system + iowait + other;

                match control_tx.send(::agent::Message::Data(
                    id.into(),
                    get_time().sec,
                    format!(
                        "{} {} {} {}",
                        (((user as f64) / (total as f64)) * 100.0) as i32,
                        (((system as f64) / (total as f64)) * 100.0) as i32,
                        (((iowait as f64) / (total as f64)) * 100.0) as i32,
                        (((other as f64) / (total as f64)) * 100.0) as i32
                    )
                )) {
                    Ok(_)  => { debug!("Sent data to control channel"); }
                    Err(e) => { panic!(format!("FATAL ERROR: [Bug] Couldn't send data on control channel - {}", e.to_string())); }
                }

                jiffies = new_jiffies;
            }
        }
    )
    .map_err(|err| err.to_string())
    .map(|handle| (id.into(), pipe_tx, handle))
}


fn get_format(pipe: ::agent::MessageSender, interval: u32, id: &str) -> bool {
    let mut fmt = vec![
        ::types::MetricFormat::Gauge(String::from("user"), interval, Some(0), Some(100)),
        ::types::MetricFormat::Gauge(String::from("system"), interval, Some(0), Some(100)),
        ::types::MetricFormat::Gauge(String::from("iowait"), interval, Some(0), Some(100)),
        ::types::MetricFormat::Gauge(String::from("other"), interval, Some(0), Some(100))
    ];
    pipe.send(::agent::Message::Format(id.into(), fmt));

    true
}

fn get_current_jiffies(re: &Regex) -> Option<[i32; 7]> {
    let mut result: [i32; 7] = [-1; 7];
    let stat = match File::open("/proc/stat") {
        Ok(f)  => f,
        Err(_) => return None
    };
    let mut reader = BufReader::new(stat);
    let mut buffer = String::new();

    match reader.read_line(&mut buffer) {
        Ok(_)  => { },
        Err(_) => return None
    }

    for cap in re.captures_iter(&buffer) {
        for i in 0..7 {
            result[i] = i32::from_str(&cap[i + 1]).expect("FATAL ERROR: Jiffies conversion error");
        }
    }

    Some(result)
}