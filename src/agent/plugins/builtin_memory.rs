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


pub fn start_builtin_memory(conf: types::Configuration, control_tx: ::agent::MessageSender) -> Result<super::NamedPluginTuple, String> {
    let (pipe_tx, pipe_rx) = sync::mpsc::channel();
    let id = "builtin.memory";
    thread::Builder::new().name(String::from("plugin_default_memory")).spawn(
        move || {
            let control_tx = control_tx;
            let plugin_rx  = pipe_rx;

            let memory_interval = u64::from_str(
                &conf.get_unsafe("builtin.memory.interval")
            ).expect("FATAL ERROR: Couldn't convert 'builtin.memory.interval' to an integer'");

            if !get_format(control_tx.clone(), memory_interval, id) {
                error!("There has been an error while starting the CPU collector, it will be disabled");
                return;
            }

            let memory_interval = memory_interval * 1000;

            let re_total   = Regex::new(r"^MemTotal:\s*(\d+)").expect("TODO: Handle regex compilation failure");
            let re_free    = Regex::new(r"^MemFree:\s*(\d+)").expect("TODO: Handle regex compilation failure");
            let re_buffers = Regex::new(r"^Buffers:\s*(\d+)").expect("TODO: Handle regex compilation failure");
            let re_cache   = Regex::new(r"^Cached:\s*(\d+)").expect("TODO: Handle regex compilation failure");

            loop {
                match util::wait_exec_result(
                    time::Duration::from_millis(memory_interval),
                    &|| { super::is_shutdown(plugin_rx.try_recv()) }
                ){
                    Ok(_)   => {
                        debug!("Default memory collector thread received shutdown command");
                        return;
                    },
                    Err(_)  => { }
                };

                let (total, free, buffers, cache) = match get_current_meminfo(&re_total, &re_free, &re_buffers, &re_cache) {
                    Some(x) => x,
                    None    => {
                        error!("Couldn't get renewed memory information, builtin memory collector will be disabled.");
                        return;
                    }
                };
                let used = total - free;

                match control_tx.send(::agent::Message::Data(
                    id.into(),
                    get_time().sec,
                    format!(
                        "{} {} {} {}",
                        free,
                        used,
                        buffers,
                        cache
                    )
                )) {
                    Ok(_)  => { debug!("Sent data to control channel"); }
                    Err(e) => { panic!(format!("FATAL ERROR: [Bug] Couldn't send data on control channel - {}", e.to_string())); }
                }
            }
        }
    )
    .map_err(|err| err.to_string())
    .map(|handle| (id.into(), pipe_tx, handle))
}


fn get_format(pipe: ::agent::MessageSender, interval: u64, id: &str) -> bool {
    let mut fmt = vec![
        super::Format::Gauge(String::from("free"),    interval, Some(0), None),
        super::Format::Gauge(String::from("used"),    interval, Some(0), None),
        super::Format::Gauge(String::from("buffers"), interval, Some(0), None),
        super::Format::Gauge(String::from("cache"),   interval, Some(0), None)
    ];
    pipe.send(::agent::Message::Format(id.into(), fmt));

    true
}

fn get_current_meminfo(re_total: &Regex, re_free: &Regex, re_buffers: &Regex, re_cache: &Regex) -> Option<(u32, u32, u32, u32)> { // (total, free, buffers, cache)
    let (mut total, mut free, mut buffers, mut cache) = (0, 0, 0, 0);
    let stat = match File::open("/proc/meminfo") {
        Ok(f)  => f,
        Err(e) => {
            debug!("ERROR: Couldn't open /proc/meminfo - {}", e);
            return None;
        }
    };
    let mut reader = BufReader::new(stat);

    for line in reader.lines() {
        let line = match line {
            Ok(l)  => l,
            Err(e) => {
                debug!("ERROR: Couldn't read line from /proc/meminfo - {}", e);
                return None;
            }
        };

        for cap in re_total.captures_iter(&line) {
            total = u32::from_str(&cap[1]).expect("FATAL ERROR: Memory conversion error");
        }
        for cap in re_free.captures_iter(&line) {
            free = u32::from_str(&cap[1]).expect("FATAL ERROR: Memory conversion error");
        }
        for cap in re_buffers.captures_iter(&line) {
            buffers = u32::from_str(&cap[1]).expect("FATAL ERROR: Memory conversion error");
        }
        for cap in re_cache.captures_iter(&line) {
            cache = u32::from_str(&cap[1]).expect("FATAL ERROR: Memory conversion error");
        }
    }

    Some((total, free, buffers, cache))
}