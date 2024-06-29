use std::time::SystemTime;

pub struct StopWatch {
    name: &'static str,
    started_at: SystemTime,
}

pub fn start_watch(name: &'static str) -> StopWatch {
    StopWatch {
        name,
        started_at: SystemTime::now(),
    }
}

impl StopWatch {
    pub fn force_complete(self) {}
}

impl Drop for StopWatch {
    fn drop(&mut self) {
        match SystemTime::now().duration_since(self.started_at) {
            Ok(it) => println!("stopwatch: {} took {:03}s", self.name, it.as_secs_f32()),
            Err(it) => println!("stopwatch: {} took unknown ({})", self.name, it),
        }
    }
}
