use lazy_static::lazy_static;

use std::{sync::Mutex, time::Duration};

lazy_static! {
    pub static ref PROFILE_MANAGER: Mutex<ProfilingManager> = Mutex::new(ProfilingManager::new());
}

macro_rules! perf {
    ($target:expr) => {
        let _tracker = {
            #[cfg(feature = "profiling")]
            crate::profiling::Tracker::new($target)
        };
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Record {
    calls: u32,
    total_duration: Duration,
    avg_duration: Duration,
}

pub struct ProfilingManager {
    map: std::collections::HashMap<&'static str, Record>,
}

impl ProfilingManager {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
    pub fn track(&mut self, target: &'static str, duration: std::time::Duration) {
        // Only execute if profiling
        #[cfg(feature = "profiling")]
        {
            match self.map.get_mut(target) {
                Some(record) => {
                    record.calls += 1;
                    record.total_duration += duration;
                    record.avg_duration = record.total_duration / record.calls;
                }
                None => {
                    self.map.insert(
                        target,
                        Record {
                            calls: 1,
                            total_duration: duration,
                            avg_duration: duration,
                        },
                    );
                }
            };
        }
    }
    pub fn flush(&mut self) {
        // Only execute if profiling
        #[cfg(feature = "profiling")]
        {
            {
                let sync_duration = std::time::Instant::now();

                use std::fs::File;
                use std::io::prelude::*;

                let mut file = File::create("profiling_log.txt").unwrap();

                let mut records: Vec<(&&str, &Record)> = self.map.iter().map(|m| m).collect();
                records.sort_by(|a, b| {
                    let v1 = a.1.total_duration; //.1.avg_duration * a.1.calls;
                    let v2 = b.1.total_duration; //s b.1.avg_duration * b.1.calls;

                    v2.partial_cmp(&v1).unwrap()
                });

                file.write_all(format!("{:#?}", records).as_bytes())
                    .unwrap();

                let sync_duration = std::time::Instant::now() - sync_duration;
                self.track("PERF SYNC", sync_duration);
            }
        }
    }
}

impl Drop for ProfilingManager {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct Tracker {
    start: std::time::Instant,
    target: &'static str,
}

impl Tracker {
    pub fn new(target: &'static str) -> Self {
        Self {
            target,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        {
            {
                let duration = std::time::Instant::now() - self.start;
                match PROFILE_MANAGER.lock() {
                    Ok(mut mgr) => {
                        mgr.track(self.target, duration);
                    }
                    Err(_) => {}
                }
            }
        }
    }
}
