use std::collections::HashMap;


type RoutingHashMap         = HashMap<String, Vec<::controller::MessageSender>>;


pub struct Router {
    routes: RoutingHashMap
}

impl Router {
    pub fn new() -> Router { Router { routes: RoutingHashMap::new() } }

    pub fn add(&mut self, metric: String, plugin: ::controller::MessageSender) {
        if self.routes.get(&metric).is_none() { self.routes.insert(metric.clone(), Vec::new()); }
        match self.routes.get_mut(&metric) {
            Some(plugins) => { plugins.push(plugin); },
            _ => { }
        }
    }

    pub fn get_channels(&self, metric: &str) -> Option<&Vec<::controller::MessageSender>> {
        self.routes.get(metric)
    }
}