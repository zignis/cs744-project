use crate::utils::us_to_ms;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Attribute, Cell, Color,
    Row, Table,
};
use hdrhistogram::Histogram;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thousands::Separable;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Metrics {
    pub workload: String,
    pub success: Arc<AtomicU64>, // 2xx response from server
    pub failure: Arc<AtomicU64>, // not network failures, 4xx/5xx from the server
    hist: Arc<Mutex<Histogram<u64>>>,
}

#[derive(Serialize)]
pub struct Summary {
    pub workload: String,
    pub threads: usize,
    pub start_time: String,
    pub duration_seconds: u64,
    pub success: u64,
    pub failure: u64,
    pub throughput_rps: f64,
    pub avg_latency: f64,
    pub max_latency: u64,
}

impl Metrics {
    pub fn new(workload: String) -> Self {
        Metrics {
            workload,
            success: Arc::new(AtomicU64::new(0)),
            failure: Arc::new(AtomicU64::new(0)),
            hist: Arc::new(Mutex::new(Histogram::new_with_max(10_000_000, 2).unwrap())),
        }
    }

    #[inline(always)]
    pub fn record_success(&self, latency: Duration) {
        self.success.fetch_add(1, Ordering::Relaxed);
        let _ = self.hist.lock().record(latency.as_micros() as u64);
    }

    #[inline(always)]
    pub fn record_failure(&self, latency: Duration) {
        self.failure.fetch_add(1, Ordering::Relaxed);
        let _ = self.hist.lock().record(latency.as_micros() as u64);
    }

    pub fn summary(&self, threads: usize, duration_secs: u64) -> Summary {
        let success = self.success.load(Ordering::Relaxed);
        let failure = self.failure.load(Ordering::Relaxed);
        let throughput = if duration_secs > 0 {
            (success + failure) as f64 / duration_secs as f64
        } else {
            0.0
        };
        let h = self.hist.lock();

        Summary {
            threads,
            workload: self.workload.clone(),
            start_time: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .unwrap_or_else(|_| "_".into()),
            duration_seconds: duration_secs,
            success,
            failure,
            throughput_rps: throughput,
            avg_latency: h.mean(),
            max_latency: h.max(),
        }
    }

    pub fn print_report(&self, s: &Summary) {
        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_header(vec![
                Cell::new("metric").add_attribute(Attribute::Bold),
                Cell::new("value").add_attribute(Attribute::Bold),
            ])
            .add_row(Row::from(vec!["workload", &s.workload]))
            .add_row(Row::from(vec![
                "duration (s)",
                &s.duration_seconds.to_string(),
            ]))
            .add_row(Row::from(vec!["threads", &s.threads.to_string()]))
            .add_row(Row::from(vec![
                Cell::new("pass").fg(Color::Green),
                Cell::new(s.success).fg(Color::Green),
            ]))
            .add_row(Row::from(vec![
                Cell::new("fail").fg(Color::Red),
                Cell::new(s.failure).fg(Color::Red),
            ]))
            .add_row(Row::from(vec![
                "throughput (req/s)",
                &format!("{:.2}", s.throughput_rps).separate_with_commas(),
            ]))
            .add_row(Row::from(vec![
                "avg latency (ms)",
                &us_to_ms(s.avg_latency),
            ]))
            .add_row(Row::from(vec![
                "max latency (ms)",
                &us_to_ms(s.max_latency as f64),
            ]));

        println!("{}", table);
    }
}
