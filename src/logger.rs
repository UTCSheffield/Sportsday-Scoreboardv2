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

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            module: "test_module".to_string(),
        };

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module, "test_module");
    }

    #[test]
    fn test_log_collector_new() {
        let collector = LogCollector::new(100);
        let entries = collector.get_entries();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_log_collector_add_entry() {
        let collector = LogCollector::new(10);

        collector.add_entry(Level::Info, "Test message", Some("test_module"));

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[0].message, "Test message");
        assert_eq!(entries[0].module, "test_module");
    }

    #[test]
    fn test_log_collector_add_entry_with_no_module() {
        let collector = LogCollector::new(10);

        collector.add_entry(Level::Info, "Test message", None);

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].module, "app");
    }

    #[test]
    fn test_log_collector_max_entries() {
        let collector = LogCollector::new(3);

        collector.add_entry(Level::Info, "Message 1", Some("module1"));
        collector.add_entry(Level::Warn, "Message 2", Some("module2"));
        collector.add_entry(Level::Error, "Message 3", Some("module3"));
        collector.add_entry(Level::Debug, "Message 4", Some("module4"));

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 3);
        // Should have newest first (after reverse)
        assert_eq!(entries[0].message, "Message 4");
        assert_eq!(entries[1].message, "Message 3");
        assert_eq!(entries[2].message, "Message 2");
    }

    #[test]
    fn test_log_collector_different_levels() {
        let collector = LogCollector::new(10);

        collector.add_entry(Level::Info, "Info message", Some("module"));
        collector.add_entry(Level::Warn, "Warn message", Some("module"));
        collector.add_entry(Level::Error, "Error message", Some("module"));
        collector.add_entry(Level::Debug, "Debug message", Some("module"));

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].level, "DEBUG");
        assert_eq!(entries[1].level, "ERROR");
        assert_eq!(entries[2].level, "WARN");
        assert_eq!(entries[3].level, "INFO");
    }

    #[test]
    fn test_log_collector_clear() {
        let collector = LogCollector::new(10);

        collector.add_entry(Level::Info, "Message 1", Some("module"));
        collector.add_entry(Level::Info, "Message 2", Some("module"));

        assert_eq!(collector.get_entries().len(), 2);

        collector.clear();

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_log_collector_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let collector = Arc::new(LogCollector::new(100));
        let mut handles = vec![];

        for i in 0..10 {
            let collector = Arc::clone(&collector);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    collector.add_entry(
                        Level::Info,
                        &format!("Thread {} message {}", i, j),
                        Some("test"),
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 100);
    }

    #[test]
    fn test_log_collector_order() {
        let collector = LogCollector::new(10);

        collector.add_entry(Level::Info, "First", Some("module"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.add_entry(Level::Info, "Second", Some("module"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.add_entry(Level::Info, "Third", Some("module"));

        let entries = collector.get_entries();
        // Newest first after reverse
        assert_eq!(entries[0].message, "Third");
        assert_eq!(entries[1].message, "Second");
        assert_eq!(entries[2].message, "First");
    }

    // E2E test
    #[tokio::test]
    async fn test_e2e_logger_integration() {
        use log::Level;

        let collector = LogCollector::new(100);

        // Simulate logging from different parts of the application
        collector.add_entry(Level::Info, "Server started", Some("main"));
        collector.add_entry(Level::Debug, "Processing request", Some("routes"));
        collector.add_entry(Level::Warn, "Low memory", Some("system"));
        collector.add_entry(Level::Error, "Database error", Some("db"));

        let entries = collector.get_entries();
        assert_eq!(entries.len(), 4);

        // Verify entries are in reverse order (newest first)
        assert_eq!(entries[0].level, "ERROR");
        assert_eq!(entries[0].message, "Database error");

        // Test clear functionality
        collector.clear();
        assert_eq!(collector.get_entries().len(), 0);
    }
}
