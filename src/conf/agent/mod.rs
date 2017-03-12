extern crate toml;

use std::io::Read;

use std::fs;
use std::sync;

use ::types;

mod defaults;


pub type ConfigResult = Result<types::ConfigurationMap, String>;


fn get_defaults() -> types::ConfigurationMap{
    let mut config = types::ConfigurationMap::new();

    config.insert(String::from("controller.host"),                      String::from("127.0.0.1"));
    config.insert(String::from("controller.port"),                      String::from("7777"));
    config.insert(String::from("controller.retry_timeout"),             String::from("10000"));
    config.insert(String::from("controller.max_retries"),               String::from("10"));

    config.insert(String::from(".plugins"),                             String::from("0"));

    config.insert(String::from("builtin.cpu.enabled"),           String::from("true"));
    config.insert(String::from("builtin.cpu.interval"),          String::from("60"));

    config.insert(String::from("builtin.memory.enabled"),        String::from("true"));
    config.insert(String::from("builtin.memory.interval"),       String::from("60"));

    config.insert(String::from("builtin.hdd.enabled"),           String::from("true"));
    config.insert(String::from("builtin.hdd.devices"),           String::from("/dev/sda1"));
    config.insert(String::from("builtin.hdd.mountpoints"),       String::from(""));

    config.insert(String::from("builtin.network.enabled"),       String::from("true"));
    config.insert(String::from("builtin.network.interfaces"),    String::from("eth0"));

    config
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

pub fn from_file(path: &str, tx_control: sync::mpsc::Sender<types::Message>) -> ConfigResult{
    let file: toml::Value;

    match super::load_file(path){
        Ok(toml_str)    => {
            let mut parser: toml::Parser = toml::Parser::new(toml_str.as_str());

            match parser.parse(){
                Some(t) => { file = toml::Value::Table(t); }
                None => { return Err(format!("Configuration file does not contain valid TOML format. - {:#?}", parser.errors)); }
            }
        }
        Err(e)          => { return Err(e); }
    }

    let mut global_conf = super::check_conf(&file, tx_control.clone());

    let mut conf = get_defaults();

    for (opt, value) in conf.iter_mut(){
       match file.lookup(opt){
            Some(v) => {
                *value = super::value_to_string(v);
            }
            None => {
                if !opt.starts_with("."){
                    tx_control.send(types::Message::LogWarn(format!("Option '{}'  not found in config, using default value '{}'", opt, value)));
                }
            }
        }
    }

    let mut plugins = 0;
    match file.lookup("plugin"){
        Some(a) => {
            match a.as_slice(){
                Some(array) => {
                    for plugin in array{
                        let mut enabled = defaults::PLUGIN_ENABLED;
                        let mut name = String::from("");
                        let mut path = String::from("");

                        match plugin.lookup("enabled"){
                            Some(en)   => { enabled = en.as_bool().expect("FATAL ERROR: Plugin attribute 'enabled' must be a boolean!"); }
                            None       => { warn!("Found a plugin declaration without an 'enabled' attribute, it will be '{}' by default.", enabled); }
                        }

                        match plugin.lookup("name"){
                            Some(n) => { name = String::from(n.as_str().expect("FATAL ERROR: Plugin attribute 'name' must be a string!")); }
                            None    => { }
                        }

                        match plugin.lookup("path"){
                            Some(p) => { path = String::from(p.as_str().expect("FATAL ERROR: Plugin attribute 'path' must be a string!")); }
                            None    => { }
                        }

                        if name == "" { warn!("Found plugin declaration without a 'path' attribute, this plugin will be ignored."); }
                        if path == "" { warn!("Found plugin declaration without a 'name' attribute, this plugin will be ignored."); }

                        if (name != "") && (path != ""){
                            plugins += 1;
                            if enabled { info!("Plugin '{}' is enabled. [{}]", name, path); }
                            conf.insert(String::from(format!(".plugin.{}.enabled", plugins)),   String::from(format!("{}", enabled)));
                            conf.insert(String::from(format!(".plugin.{}.name", plugins)),      name);
                            conf.insert(String::from(format!(".plugin.{}.path", plugins)),      path);
                        }

                    }
                }
                None => { warn!("Found invalid plugin directive in config. {}", a); }
            }
        }
        None => { info!("No plugins found"); }
    }

    conf.insert(String::from(".plugins"), format!("{}", plugins));

    conf.extend(global_conf);

    Ok(conf)
}
