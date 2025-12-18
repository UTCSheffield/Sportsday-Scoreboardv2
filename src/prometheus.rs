use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use async_sqlite::Pool;
use prometheus::Gauge;
use std::fs;
use std::thread;
use std::time::Duration;

use crate::db::{events::Events, users::Users};

// Parse total jiffies from /proc/stat (first "cpu" line)
fn read_total_jiffies() -> Option<u64> {
    let s = fs::read_to_string("/proc/stat").ok()?;
    for line in s.lines() {
        if line.starts_with("cpu ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // parts[0] == "cpu"
            let mut sum: u64 = 0;
            for v in parts.iter().skip(1) {
                if let Ok(n) = v.parse::<u64>() {
                    sum = sum.saturating_add(n);
                }
            }
            return Some(sum);
        }
    }
    None
}

// Parse process jiffies (utime + stime) from /proc/self/stat
fn read_proc_jiffies() -> Option<u64> {
    let s = fs::read_to_string("/proc/self/stat").ok()?;
    // stat fields: see proc manpage. utime is field 14, stime 15 (1-based)
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() > 15 {
        let utime = parts[13].parse::<u64>().ok()?;
        let stime = parts[14].parse::<u64>().ok()?;
        return Some(utime.saturating_add(stime));
    }
    None
}

// Read resident set size (VmRSS) in bytes from /proc/self/status
fn read_proc_rss_bytes() -> Option<u64> {
    let s = fs::read_to_string("/proc/self/status").ok()?;
    for line in s.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // VmRSS: <value> kB
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    return Some(kb * 1024);
                }
            }
        }
    }
    None
}

// Collect CPU and memory usage for the current process only (Linux /proc implementation).
pub fn build_prom(pool: Pool) -> PrometheusMetrics {
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let cpu_usage = Gauge::new(
        "process_cpu_usage_percent",
        "Current CPU usage of this process in percent",
    )
    .unwrap();
    let mem_usage = Gauge::new(
        "process_memory_bytes",
        "Resident memory used by this process in bytes",
    )
    .unwrap();
    let event_count = Gauge::new("event_count", "Total number of events in the database").unwrap();
    let user_count = Gauge::new("user_count", "Total number of users in the database").unwrap();

    prometheus
        .registry
        .register(Box::new(cpu_usage.clone()))
        .unwrap();

    prometheus
        .registry
        .register(Box::new(mem_usage.clone()))
        .unwrap();

    prometheus
        .registry
        .register(Box::new(event_count.clone()))
        .unwrap();

    prometheus
        .registry
        .register(Box::new(user_count.clone()))
        .unwrap();

    thread::spawn(move || {
        // Create a new tokio runtime for async operations
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        // initial values
        let mut prev_total = read_total_jiffies().unwrap_or(0);
        let mut prev_proc = read_proc_jiffies().unwrap_or(0);

        loop {
            thread::sleep(Duration::from_secs(1));

            let total = match read_total_jiffies() {
                Some(v) => v,
                None => continue,
            };
            let proc = match read_proc_jiffies() {
                Some(v) => v,
                None => continue,
            };

            let delta_total = total.saturating_sub(prev_total);
            let delta_proc = proc.saturating_sub(prev_proc);

            prev_total = total;
            prev_proc = proc;

            if delta_total > 0 {
                // Percentage = proc_delta / total_delta * 100
                let percent = (delta_proc as f64 / delta_total as f64) * 100.0;
                cpu_usage.set(percent);
            }

            if let Some(rss_bytes) = read_proc_rss_bytes() {
                mem_usage.set(rss_bytes as f64);
            }

            // Update event and user counts
            let pool_clone = pool.clone();
            if let Ok(count) = rt.block_on(async { Events::count(&pool_clone).await }) {
                event_count.set(count as f64);
            }

            let pool_clone = pool.clone();
            if let Ok(count) = rt.block_on(async { Users::count(&pool_clone).await }) {
                user_count.set(count as f64);
            }
        }
    });

    prometheus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_read_total_jiffies() {
        let result = read_total_jiffies();
        // On Linux, this should return Some value
        if cfg!(target_os = "linux") {
            assert!(result.is_some());
            if let Some(jiffies) = result {
                assert!(jiffies > 0);
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_read_proc_jiffies() {
        let result = read_proc_jiffies();
        // Should return some value on Linux
        if cfg!(target_os = "linux") {
            assert!(result.is_some());
            if let Some(jiffies) = result {
                assert!(jiffies >= 0);
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_read_proc_rss_bytes() {
        let result = read_proc_rss_bytes();
        // Should return some value on Linux
        if cfg!(target_os = "linux") {
            assert!(result.is_some());
            if let Some(bytes) = result {
                assert!(bytes > 0);
            }
        }
    }

    #[tokio::test]
    async fn test_build_prom_creates_metrics() {
        use crate::test_harness;

        let db = test_harness::setup_db("prometheus_test").await;
        let prom = build_prom(db.clone());

        // Verify the prometheus metrics builder was created successfully
        assert_eq!(prom.registry.gather().len() >= 4, true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_jiffies_increase_over_time() {
        let first = read_total_jiffies();

        // Do some work
        let mut sum = 0u64;
        for i in 0..1000000 {
            sum = sum.wrapping_add(i);
        }

        let second = read_total_jiffies();

        if let (Some(f), Some(s)) = (first, second) {
            // Total jiffies should increase (or at least not decrease)
            assert!(s >= f, "Expected jiffies to increase: {} -> {}", f, s);
        }

        // Use sum to prevent optimization
        assert!(sum > 0);
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_read_functions_on_non_linux() {
        // On non-Linux systems, these should return None
        assert_eq!(read_total_jiffies(), None);
        assert_eq!(read_proc_jiffies(), None);
        assert_eq!(read_proc_rss_bytes(), None);
    }

    #[test]
    fn test_gauge_registration() {
        let cpu_gauge = Gauge::new("test_cpu_usage", "Test CPU usage metric").unwrap();

        cpu_gauge.set(42.5);
        assert_eq!(cpu_gauge.get(), 42.5);
    }

    #[test]
    fn test_multiple_gauge_updates() {
        let mem_gauge = Gauge::new("test_memory", "Test memory metric").unwrap();

        mem_gauge.set(100.0);
        assert_eq!(mem_gauge.get(), 100.0);

        mem_gauge.set(200.0);
        assert_eq!(mem_gauge.get(), 200.0);

        mem_gauge.set(150.0);
        assert_eq!(mem_gauge.get(), 150.0);
    }
}
