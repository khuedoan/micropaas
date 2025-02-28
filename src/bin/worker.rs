use anyhow::Result;
use micropaas::activities;
use micropaas::temporal::get_client;
use micropaas::workflows;
use std::sync::Arc;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::try_from_env("LOG_LEVEL").unwrap_or(EnvFilter::new("info")))
        .without_time()
        .init();

    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("deploy")
        .worker_build_id("core-worker")
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;

    let mut worker = Worker::new_from_core(Arc::new(core_worker), "deploy");

    worker.register_activity("from_source", activities::build::from_source);
    worker.register_wf("deploy_from_source", workflows::deploy::deploy_from_source);

    worker.run().await?;

    Ok(())
}
