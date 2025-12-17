use chrono::{DateTime, Utc};
use log::Level;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub module: String, // Changed from Option<String> to String
}

/// Thread-safe log collector that stores recent log entries
#[derive(Debug, Clone)]
pub struct LogCollector {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
    max_entries: usize,
}

impl LogCollector {
    /// Create a new log collector with a maximum number of entries to store
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Add a new log entry
    pub fn add_entry(&self, level: Level, message: &str, module: Option<&str>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
            module: module.unwrap_or("app").to_string(), // Default to "app" if no module
        };

        let mut entries = self.entries.lock().unwrap();

        // Remove oldest entry if we've reached the limit
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
    }

    /// Get all log entries as a vector (newest first)
    pub fn get_entries(&self) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        let mut result: Vec<_> = entries.iter().cloned().collect();
        result.reverse(); // Show newest first
        result
    }

    /// Clear all log entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
}

/// Custom logger that writes to both env_logger and our collector
pub struct CustomLogger {
    collector: LogCollector,
    env_logger: env_logger::Logger,
}

impl CustomLogger {
    pub fn new(collector: LogCollector) -> Self {
        let env_logger =
            env_logger::Logger::from_env(env_logger::Env::new().default_filter_or("info"));

        Self {
            collector,
            env_logger,
        }
    }
}

impl log::Log for CustomLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        // Log to env_logger first
        self.env_logger.log(record);

        // Add to our collector
        self.collector.add_entry(
            record.level(),
            &record.args().to_string(),
            record.module_path(),
        );
    }

    fn flush(&self) {
        self.env_logger.flush();
    }
}
