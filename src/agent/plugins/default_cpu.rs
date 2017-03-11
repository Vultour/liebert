use std::io::prelude::*;
use std::thread;
use std::sync;
use std::time;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;
use regex::Regex;

use ::types;
use ::util;


pub fn start_default_cpu(conf: types::complex::Configuration, control_tx: types::MessageSender, id: i32) -> Result<super::PluginTuple, String> {
    let (pipe_tx, pipe_rx) = sync::mpsc::channel();
    thread::Builder::new().name(String::from("plugin_default_cpu")).spawn(
        move || {
            let control_tx = control_tx;
            let plugin_rx  = pipe_rx;

            let cpu_interval = u64::from_str(
                &conf.get_unsafe("builtin.cpu.interval")
            ).expect("FATAL ERROR: Couldn't convert 'plugins.builtin.cpu.interval' to an integer'");

            if !get_format(control_tx.clone(), cpu_interval, id) {
                error!("There has been an error while starting the CPU collector, it will be disabled");
                return;
            }

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
                    time::Duration::from_millis(cpu_interval),
                    &|| { super::is_shutdown(plugin_rx.try_recv()) }
                ){
                    Ok(_)   => {
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

                match control_tx.send(types::Message::Data((
                    id,
                    format!(
                        "{} {} {} {}",
                        (((user as f64) / (total as f64)) * 100.0) as i32,
                        (((system as f64) / (total as f64)) * 100.0) as i32,
                        (((iowait as f64) / (total as f64)) * 100.0) as i32,
                        (((other as f64) / (total as f64)) * 100.0) as i32
                    )
                ))) {
                    Ok(_)  => { debug!("Sent data to control channel"); }
                    Err(e) => { panic!(format!("FATAL ERROR: [Bug] Couldn't send data on control channel - {}", e.to_string())); }
                }

                jiffies = new_jiffies;
            }
        }
    )
    .map_err(|err| err.to_string())
    .map(|handle| (pipe_tx, handle))
}


fn get_format(pipe: types::MessageSender, interval: u64, id: i32) -> bool {
    let mut fmt = vec![
        super::Format::Gauge(String::from("user"), interval, Some(0), Some(100)),
        super::Format::Gauge(String::from("system"), interval, Some(0), Some(100)),
        super::Format::Gauge(String::from("iowait"), interval, Some(0), Some(100)),
        super::Format::Gauge(String::from("other"), interval, Some(0), Some(100))
    ];
    pipe.send(types::Message::Format(fmt));

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