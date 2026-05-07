use std::time::Instant;

pub struct ScriptIsolation {
    pub max_heap_mb: u32,
    pub max_stack_kb: u32,
    pub max_time_ms: u64,
    pub max_instructions: u64,
    pub recursion_depth: u32,
    start_time: Option<Instant>,
}

impl Default for ScriptIsolation {
    fn default() -> Self {
        Self {
            max_heap_mb: 10,
            max_stack_kb: 64,
            max_time_ms: 5000,
            max_instructions: 1_000_000,
            recursion_depth: 50,
            start_time: None,
        }
    }
}

impl ScriptIsolation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn check_timeout(&self) -> bool {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            elapsed.as_millis() as u64 > self.max_time_ms
        } else {
            false
        }
    }

    pub fn get_elapsed_ms(&self) -> Option<u64> {
        self.start_time.map(|s| s.elapsed().as_millis() as u64)
    }
}
