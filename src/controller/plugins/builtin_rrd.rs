use std::mem;
use std::thread;
use std::sync;
use std::process::{Command, Stdio};
use std::collections::HashMap;


type FormatVector  = Vec<::types::MetricFormat>;
type FormatHashMap = HashMap<String, FormatVector>;


pub fn start_builtin_rrd(conf: ::types::Configuration, tx: ::controller::MessageSender) -> Result<super::NamedPluginTuple, String> {
    let (channel_tx, channel_rx) = sync::mpsc::channel();
    let channel_tx_clone = channel_tx.clone();
    thread::Builder::new().name("plg_bi-rrd".into()).spawn(
        move || {
            let mut formats = get_current_formats();
            let binary = conf.get_unsafe("builtin.rrd.binary");
            let path = conf.get_unsafe("builtin.rrd.data");
            loop {
                match channel_rx.recv() {
                    Ok(msg) => {
                        match msg {
                            ::controller::Message::Format(h, m, f) => {
                                let metric_uid = format!("{}-{}", h, m);
                                match formats.get(&metric_uid) {
                                    Some(ref mut fr) => {
                                        if !format_equals(fr, &f) {
                                            trace!("Replacing existing format");
                                            create_rrd_file(&binary, &path, &metric_uid, &f);
                                            mem::replace(fr, &f); // I don't even know.
                                        }
                                    },
                                    None => { }
                                }
                                if formats.get(&metric_uid).is_none() {
                                    create_rrd_file(&binary, &path, &metric_uid, &f);
                                    formats.insert(metric_uid, f);
                                }
                            },
                            ::controller::Message::Data(h, m, t, d) => {
                                let metric_uid = format!("{}-{}", h, m);
                                update_rrd_file(&binary, &path, &metric_uid, t, d);

                            },
                            _ => { warn!("Unprocessed message received on builtin_rrd channel"); }
                        }
                    },
                    Err(e) => panic!("All Builtin RRD sender channels closed - {}", e)
                }
            }
        }
    ).map_err(|e| e.to_string())
    .map(|h| ("builtin.rrd".into(), channel_tx_clone, h))
}


fn format_equals(x: &FormatVector, y: &FormatVector) -> bool {
    if x.len() != y.len() { return false; }
    for i in 0..(x.len() - 1) {
        if x[i] != y[i] { return false; }
    }

    true
}

fn get_current_formats() -> FormatHashMap {
    FormatHashMap::new()
}


// RRD functions
fn create_rrd_file(bin: &str, path: &str, id: &str, format: &FormatVector) {
    debug!("Creating RRD file for {}", id);
    let mut cmd = Command::new(bin);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.arg("create");
    cmd.arg(format!("{}/{}.rrd", path, id));
    cmd.arg("--step");
    cmd.arg("5");

    for f in format {
        match f {
            &::types::MetricFormat::Gauge(ref n, ref hb, ref min, ref max) => {
                cmd.arg(format!("DS:{}:GAUGE:{}:{}:{}", n, hb, ::util::rrd_to_string(min), ::util::rrd_to_string(max)));
            },
            &::types::MetricFormat::Counter(ref n, ref hb, ref min, ref max) => {
                cmd.arg(format!("DS:{}:COUNTER:{}:{}:{}", n, hb, ::util::rrd_to_string(min), ::util::rrd_to_string(max)));
            }
        }
    }

    cmd.arg("RRA:MAX:0.5:1:60");
    cmd.arg("RRA:MAX:0.5:5:144");
    cmd.arg("RRA:MAX:0.5:15:96");
    cmd.arg("RRA:MAX:0.5:60:168");
    cmd.arg("RRA:MAX:0.5:720:62");

    let cmd = match cmd.spawn() {
        Ok(c)   => c,
        Err(e) => {
            error!("Could not create RRD file for {} - {}", id, e);
            return;
        }
    };

    match cmd.wait_with_output() {
        Ok(o)  => {
            if !o.status.success() {
                error!(
                    "Call to rrdcreate returned an exit code indicating an error - code: {} - stdout: {} - stderr: {}",
                    o.status.code().unwrap_or(-1),
                    String::from_utf8(o.stdout).unwrap_or("< liebert internal: error converting byte array into string >".into()),
                    String::from_utf8(o.stderr).unwrap_or("< liebert internal: error converting byte array into string >".into())
                );
            }
        },
        Err(e) => { error!("Could not wait for rrdcreate output - {}", e); }
    }
}

fn update_rrd_file(bin: &str, path: &str, id: &str, timestamp: u32, data: Vec<i64>) {
    debug!("Updating RRD file for {}", id);
    let mut cmd = Command::new(bin);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.arg("update");
    cmd.arg(format!("{}/{}.rrd", path, id));
    cmd.arg(format!("{}:{}", timestamp, data.into_iter().map(|n| n.to_string()).collect::<Vec<String>>().join(":")));

    let cmd = match cmd.spawn() {
        Ok(c)   => c,
        Err(e) => {
            error!("Could not update RRD file for {} - {}", id, e);
            return;
        }
    };

    match cmd.wait_with_output() {
        Ok(o)  => {
            if !o.status.success() {
                error!(
                    "Call to rrdupdate returned an exit code indicating an error - code: {} - stdout: {} - stderr: {}",
                    o.status.code().unwrap_or(-1),
                    String::from_utf8(o.stdout).unwrap_or("< liebert internal: error converting byte array into string >".into()),
                    String::from_utf8(o.stderr).unwrap_or("< liebert internal: error converting byte array into string >".into())
                );
            }
        },
        Err(e) => { error!("Could not wait for rrdupdate output - {}", e); }
    }
}