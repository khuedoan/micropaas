use micropaas::temporal::get_client;
use temporal_client::WorkflowOptions;
use temporal_sdk_core::WorkflowClientTrait;
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::try_from_env("LOG_LEVEL").unwrap_or(EnvFilter::new("info")))
        .without_time()
        .init();

    let client = get_client().await?;

    let _example = client
        .start_workflow(
            vec!["".as_json_payload()?],
            "deploy".to_string(),
            "workflow-id-5".to_string(),
            "deploy_from_source".to_string(),
            None,
            WorkflowOptions {
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}
