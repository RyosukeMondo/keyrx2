//! Lock-free latency metrics collection.
//!
//! This module provides efficient latency recording for the hot path
//! and periodic aggregation for WebSocket broadcast.
//!
//! Design principles:
//! - Lock-free recording using atomic operations (no mutexes on hot path)
//! - Ring buffer for recent samples (cache-friendly, bounded memory)
//! - Heavy percentile computation done off hot path (every 1 second)

use hdrhistogram::Histogram;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Size of the ring buffer for recent latency samples.
/// 1024 samples covers ~10 seconds at 100 keys/sec typing speed.
const SAMPLE_BUFFER_SIZE: usize = 1024;

/// Lock-free latency recorder for the hot path.
///
/// Uses atomic operations to avoid mutex contention during event processing.
/// Samples are stored in a ring buffer that wraps around automatically.
///
/// # Performance
///
/// - `record()`: O(1), ~10-50ns (atomic operations only)
/// - Memory: 8KB fixed (1024 × 8-byte atomics)
pub struct LatencyRecorder {
    /// Ring buffer for recent latency samples (microseconds).
    samples: [AtomicU64; SAMPLE_BUFFER_SIZE],
    /// Current write index (wraps around).
    write_index: AtomicU64,
    /// Total samples recorded since creation.
    total_samples: AtomicU64,
    /// Samples recorded since last snapshot.
    samples_since_snapshot: AtomicU64,
}

impl LatencyRecorder {
    /// Creates a new latency recorder with zeroed samples.
    pub fn new() -> Self {
        // Initialize array of atomics using from_fn to avoid interior mutability warning
        Self {
            samples: std::array::from_fn(|_| AtomicU64::new(0)),
            write_index: AtomicU64::new(0),
            total_samples: AtomicU64::new(0),
            samples_since_snapshot: AtomicU64::new(0),
        }
    }

    /// Records a latency sample (lock-free, O(1)).
    ///
    /// This is the hot-path method called after each event is processed.
    /// Uses relaxed ordering since we don't need strict synchronization -
    /// occasional out-of-order writes are acceptable for statistics.
    ///
    /// # Arguments
    ///
    /// * `latency_us` - Processing latency in microseconds
    #[inline]
    pub fn record(&self, latency_us: u64) {
        let idx = self.write_index.fetch_add(1, Ordering::Relaxed) as usize % SAMPLE_BUFFER_SIZE;
        self.samples[idx].store(latency_us, Ordering::Relaxed);
        self.total_samples.fetch_add(1, Ordering::Relaxed);
        self.samples_since_snapshot.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the total number of samples recorded.
    pub fn total_samples(&self) -> u64 {
        self.total_samples.load(Ordering::Relaxed)
    }

    /// Returns samples recorded since last snapshot and resets counter.
    fn take_samples_since_snapshot(&self) -> u64 {
        self.samples_since_snapshot.swap(0, Ordering::Relaxed)
    }

    /// Collects all samples from the ring buffer.
    ///
    /// This is called by the aggregator, NOT on the hot path.
    fn collect_samples(&self) -> Vec<u64> {
        let mut samples = Vec::with_capacity(SAMPLE_BUFFER_SIZE);
        for i in 0..SAMPLE_BUFFER_SIZE {
            let value = self.samples[i].load(Ordering::Relaxed);
            if value > 0 {
                samples.push(value);
            }
        }
        samples
    }
}

impl Default for LatencyRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated latency statistics snapshot.
///
/// This structure is sent to WebSocket clients for display.
#[derive(Debug, Clone)]
pub struct LatencySnapshot {
    /// Minimum latency in microseconds.
    pub min_us: u64,
    /// Average latency in microseconds.
    pub avg_us: u64,
    /// Maximum latency in microseconds.
    pub max_us: u64,
    /// 50th percentile (median) in microseconds.
    pub p50_us: u64,
    /// 95th percentile in microseconds.
    pub p95_us: u64,
    /// 99th percentile in microseconds.
    pub p99_us: u64,
    /// Number of samples in this snapshot.
    pub sample_count: u64,
    /// Timestamp in microseconds since UNIX epoch.
    pub timestamp_us: u64,
}

impl LatencySnapshot {
    /// Creates an empty snapshot (all zeros).
    pub fn empty() -> Self {
        Self {
            min_us: 0,
            avg_us: 0,
            max_us: 0,
            p50_us: 0,
            p95_us: 0,
            p99_us: 0,
            sample_count: 0,
            timestamp_us: current_timestamp_us(),
        }
    }
}

/// Metrics aggregator that computes statistics from recorded samples.
///
/// This runs on a background task (not the hot path) and periodically
/// computes percentiles using HdrHistogram.
pub struct MetricsAggregator {
    /// HdrHistogram for efficient percentile calculation.
    /// Configured for microsecond precision, up to 1 second max.
    histogram: Histogram<u64>,
    /// Last time a snapshot was computed.
    last_snapshot: Instant,
    /// Rolling window duration for statistics.
    _window_duration: Duration,
}

impl MetricsAggregator {
    /// Creates a new metrics aggregator.
    ///
    /// # Arguments
    ///
    /// * `window_duration` - Rolling window for statistics (e.g., 60 seconds)
    pub fn new(window_duration: Duration) -> Self {
        // Configure histogram: 1μs to 1s range, 3 significant figures
        let histogram =
            Histogram::new_with_bounds(1, 1_000_000, 3).expect("Failed to create histogram");

        Self {
            histogram,
            last_snapshot: Instant::now(),
            _window_duration: window_duration,
        }
    }

    /// Computes a statistics snapshot from the recorder.
    ///
    /// This method:
    /// 1. Collects samples from the ring buffer
    /// 2. Adds them to the histogram
    /// 3. Computes percentiles
    /// 4. Returns a snapshot
    ///
    /// Called periodically (every 1 second) from the background broadcast task.
    pub fn compute_snapshot(&mut self, recorder: &LatencyRecorder) -> LatencySnapshot {
        let samples_count = recorder.take_samples_since_snapshot();

        if samples_count == 0 {
            // No new samples - return last known values or empty
            return LatencySnapshot {
                min_us: self.histogram.min(),
                avg_us: self.histogram.mean() as u64,
                max_us: self.histogram.max(),
                p50_us: self.histogram.value_at_percentile(50.0),
                p95_us: self.histogram.value_at_percentile(95.0),
                p99_us: self.histogram.value_at_percentile(99.0),
                sample_count: self.histogram.len(),
                timestamp_us: current_timestamp_us(),
            };
        }

        // Collect and add samples to histogram
        let samples = recorder.collect_samples();
        for &sample in &samples {
            // Clamp to histogram bounds to avoid errors
            let clamped = sample.clamp(1, 1_000_000);
            let _ = self.histogram.record(clamped);
        }

        // Compute statistics
        let snapshot = LatencySnapshot {
            min_us: self.histogram.min(),
            avg_us: self.histogram.mean() as u64,
            max_us: self.histogram.max(),
            p50_us: self.histogram.value_at_percentile(50.0),
            p95_us: self.histogram.value_at_percentile(95.0),
            p99_us: self.histogram.value_at_percentile(99.0),
            sample_count: self.histogram.len(),
            timestamp_us: current_timestamp_us(),
        };

        self.last_snapshot = Instant::now();
        snapshot
    }

    /// Resets the histogram (e.g., after config reload).
    pub fn reset(&mut self) {
        self.histogram.reset();
    }
}

/// Returns current timestamp in microseconds since UNIX epoch.
fn current_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_recorder_new() {
        let recorder = LatencyRecorder::new();
        assert_eq!(recorder.total_samples(), 0);
    }

    #[test]
    fn test_latency_recorder_record() {
        let recorder = LatencyRecorder::new();

        recorder.record(100);
        recorder.record(200);
        recorder.record(300);

        assert_eq!(recorder.total_samples(), 3);
    }

    #[test]
    fn test_latency_recorder_ring_buffer_wrap() {
        let recorder = LatencyRecorder::new();

        // Write more than buffer size to test wrap-around
        for i in 0..SAMPLE_BUFFER_SIZE + 100 {
            recorder.record(i as u64);
        }

        assert_eq!(recorder.total_samples(), SAMPLE_BUFFER_SIZE as u64 + 100);

        // Samples should still be collectible
        let samples = recorder.collect_samples();
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_metrics_aggregator_new() {
        let aggregator = MetricsAggregator::new(Duration::from_secs(60));
        assert!(aggregator.histogram.is_empty());
    }

    #[test]
    fn test_metrics_aggregator_compute_snapshot() {
        let recorder = LatencyRecorder::new();
        let mut aggregator = MetricsAggregator::new(Duration::from_secs(60));

        // Record some samples
        recorder.record(100);
        recorder.record(200);
        recorder.record(300);
        recorder.record(400);
        recorder.record(500);

        let snapshot = aggregator.compute_snapshot(&recorder);

        assert_eq!(snapshot.sample_count, 5);
        assert!(snapshot.min_us >= 100);
        assert!(snapshot.max_us <= 500);
        assert!(snapshot.avg_us > 0);
    }

    #[test]
    fn test_metrics_aggregator_empty_snapshot() {
        let recorder = LatencyRecorder::new();
        let mut aggregator = MetricsAggregator::new(Duration::from_secs(60));

        // No samples recorded
        let snapshot = aggregator.compute_snapshot(&recorder);

        assert_eq!(snapshot.sample_count, 0);
    }

    #[test]
    fn test_latency_snapshot_empty() {
        let snapshot = LatencySnapshot::empty();

        assert_eq!(snapshot.min_us, 0);
        assert_eq!(snapshot.avg_us, 0);
        assert_eq!(snapshot.max_us, 0);
        assert_eq!(snapshot.sample_count, 0);
        assert!(snapshot.timestamp_us > 0);
    }

    #[test]
    fn test_concurrent_recording() {
        use std::sync::Arc;
        use std::thread;

        let recorder = Arc::new(LatencyRecorder::new());
        let mut handles = vec![];

        // Spawn multiple threads recording concurrently
        for _ in 0..4 {
            let r = Arc::clone(&recorder);
            handles.push(thread::spawn(move || {
                for i in 0..1000 {
                    r.record(i as u64);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All samples should be recorded
        assert_eq!(recorder.total_samples(), 4000);
    }
}
