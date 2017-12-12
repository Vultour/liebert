use std::fmt;

use ::types;


pub struct Configuration{
    conf: types::ConfigurationMutex
}


impl Configuration{
    pub fn new(c: types::ConfigurationMutex) -> Configuration{
        Configuration{ conf: c }
    }

    pub fn clone(&self) -> Configuration{
        Configuration{ conf: self.conf.clone() }
    }

    pub fn clone_inner(&self) -> types::ConfigurationMutex{
        self.conf.clone()
    }
}

impl Configuration{
    pub fn get(&self, path: &str) -> Option<String>{
        let c = self.conf.lock().expect("FATAL ERROR: Couldn't lock configuration mutex");
        match c.get(path){
            Some(item)  => { return Some((*item).clone()); }
            None        => { return None; }
        }
    }

    pub fn get_unsafe(&self, path: &str) -> String{
        self.get(path).expect(format!("FATAL ERROR: [Bug?] Couldn't find option '{}' in configuration", path).as_str())
    }
}

impl fmt::Debug for Configuration{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let data = self.conf.lock().expect("FATAL ERROR: Couldn't lock configuration mutex");
        data.fmt(f)
    }
}
