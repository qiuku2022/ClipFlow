use std::sync::{Condvar, Mutex};

pub struct BackpressureController {
    max_depth: usize,
    state: Mutex<usize>,
    condvar: Condvar,
}

impl BackpressureController {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            state: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    pub fn try_push(&self) -> bool {
        let mut depth = self.state.lock().unwrap();
        if *depth < self.max_depth {
            *depth += 1;
            true
        } else {
            false
        }
    }

    pub fn wait_and_push(&self) {
        let mut depth = self.state.lock().unwrap();
        while *depth >= self.max_depth {
            depth = self.condvar.wait(depth).unwrap();
        }
        *depth += 1;
    }

    pub fn pop(&self) {
        let mut depth = self.state.lock().unwrap();
        if *depth > 0 {
            *depth -= 1;
            self.condvar.notify_one();
        }
    }

    pub fn current_depth(&self) -> usize {
        *self.state.lock().unwrap()
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}
