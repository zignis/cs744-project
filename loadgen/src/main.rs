use clap::Parser;
use futures::future::join_all;
use loadgen::client::HttpClient;
use loadgen::metrics::Metrics;
use loadgen::workloads::{preload_hotset, Config as WConfig, WorkloadGenerator, WorkloadType};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Parser)]
#[command(author, version, about = "load generator")]
struct Args {
    /// number of concurrent (closed-loop) clients
    #[arg(long, default_value_t = 32)]
    threads: usize,
    /// duration of the load test (in seconds)
    #[arg(long, default_value_t = 300)]
    duration: u64,
    /// server endpoint
    #[arg(long, default_value = "http://localhost:8000")]
    server: String,
    /// workload type: put_all | get_all | get_popular | mixed
    #[arg(long, default_value = "get_popular")]
    workload: String,
    /// percentage of GET requests in mixed workload
    #[arg(long, default_value = "80")]
    mixed_workload_get_pct: u8,
    /// percentage of PUT requests in mixed workload
    #[arg(long, default_value = "10")]
    mixed_workload_put_pct: u8,
    /// percentage of DELETE requests in mixed workload
    #[arg(long, default_value = "10")]
    mixed_workload_delete_pct: u8,
    /// percentage of GET requests in mixed workload that hit hot keys
    #[arg(long, default_value = "30")]
    mixed_workload_hot_get_pct: u8,
    /// for key PUT request, the size of value in bytes
    #[arg(long, default_value_t = 64)]
    payload_size: usize,
    /// hotset size for get_popular (how many popular keys)
    #[arg(long, default_value_t = 3)]
    hotset: usize,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("starting loadgen with args: {:#?}", args);

    let workload_type = match args.workload.as_str() {
        "put_all" => WorkloadType::PutAll,
        "get_all" => WorkloadType::GetAll,
        "get_popular" => WorkloadType::GetPopular,
        "mixed" => WorkloadType::Mixed,
        other => panic!("unknown workload `{}`", other),
    };

    let workload_cfg = WConfig {
        payload_size: args.payload_size,
        hotset: args.hotset,
        mixed_get_pct: args.mixed_workload_get_pct,
        mixed_put_pct: args.mixed_workload_put_pct,
        mixed_delete_pct: args.mixed_workload_delete_pct,
        mixed_hot_get_pct: args.mixed_workload_hot_get_pct,
    };

    let client = Arc::new(HttpClient::new(&args.server, args.threads * 2)?);
    let metrics = Arc::new(Metrics::new(args.workload));
    let start = Instant::now();
    let duration = Duration::from_secs(args.duration);
    let end = start + duration;

    if matches!(
        workload_type,
        WorkloadType::GetPopular | WorkloadType::Mixed
    ) && args.hotset > 0
    {
        println!("starting preloading...");
        preload_hotset(&client, args.hotset, args.payload_size)
            .await
            .expect("failed preload/warmup");
        println!("finished preloading");
    }

    // spawn workers
    let mut handles = Vec::with_capacity(args.threads);

    for i in 0..args.threads {
        let client = client.clone();
        let metrics = metrics.clone();
        let mut workload_gen =
            WorkloadGenerator::new(workload_type.clone(), workload_cfg.clone(), i as u64);

        // spawn a thread per client (closed-loop)
        handles.push(tokio::spawn(async move {
            loop {
                // each client stops when elapsed exceeds duration
                if Instant::now() >= end {
                    break;
                }

                let now = Instant::now();
                let res = client.send_request(workload_gen.next_request()).await;
                let latency = now.elapsed();

                match res {
                    Ok(status) if status.is_success() => metrics.record_success(latency),
                    _ => metrics.record_failure(latency),
                }
            }
        }));
    }

    join_all(handles).await;
    let summary = metrics.summary(args.threads, args.duration);
    metrics.print_report(&summary);

    Ok(())
}
