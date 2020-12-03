use std::sync::{Arc, atomic::{AtomicBool, Ordering}};


#[derive(Debug, Clone)]
pub struct Handle {
    pub(crate) request_redraw: Arc<AtomicBool>,
}


impl Handle {
    pub(crate) fn new() -> Self {
        Self {
            request_redraw: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn request_redraw(&mut self){
        self.request_redraw.store(true, Ordering::SeqCst);
    }
}