use std::sync::{Arc, RwLock};

use crate::Event;

pub trait Listener: Send + Sync {
    fn on(&self, e: &Event);
}

impl<F: Fn(&Event) + Send + Sync> Listener for F {
    fn on(&self, e: &Event) {
        self(e)
    }
}

#[derive(Default)]
pub struct Bus {
    subs: RwLock<Vec<Arc<dyn Listener>>>,
}

impl Bus {
    pub fn join(&self, l: Arc<dyn Listener>) {
        self.subs.write().unwrap().push(l);
    }

    pub fn emit(&self, e: Event) {
        for l in self.subs.read().unwrap().iter() {
            l.on(&e);
        }
    }
}
