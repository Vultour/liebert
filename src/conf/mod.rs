extern crate toml;

use std::io::Read;

use std::fs;
use std::sync;

use ::types;

mod defaults;
pub mod agent;
//pub mod controller;


pub type ConfigResult = Result<types::ConfigurationMap, String>;


fn get_defaults() -> types::ConfigurationMap{
    let mut config = types::ConfigurationMap::new();

    // Global defaults

    config
}


fn value_to_string(value: &toml::Value) -> String{
    match *value{
        toml::Value::String(ref v)  => return v.clone(),
        toml::Value::Integer(ref v) => return v.to_string(),
        toml::Value::Boolean(ref v) => return v.to_string(),
        toml::Value::Float(ref v)   => return v.to_string(),
        _                           => panic!("FATAL ERROR: Encountered configuration value of type '{}' which is not supported", value.type_str())
    }
}


fn load_file(path: &str) -> Result<String, String>{
    let mut f: fs::File;
    let mut s = String::new();

    match fs::File::open(path){
        Ok(file)    => { f = file; }
        Err(e)      => { return Err(format!("Error opening configuration file [{}] - {}", path, e)); }
    }

    match f.read_to_string(&mut s){
        Ok(_)   => { }
        Err(e)  => { return Err(format!("Error reading configuration file [{}] - {}", path, e)); }
    }

    Ok(s)
}

pub fn check_conf(toml: &toml::Value, tx_control: sync::mpsc::Sender<types::Message>) -> types::ConfigurationMap{
    let mut conf = get_defaults();

    for (opt, value) in conf.iter_mut(){
       match toml.lookup(opt){
            Some(v) => {
                *value = value_to_string(v);
            }
            None => {
                if !opt.starts_with("."){
                    tx_control.send(types::Message::LogInfo(format!("Option '{}'  not found in config, using default value '{}'", opt, value)));
                }
            }
        }
    }

    conf
}
