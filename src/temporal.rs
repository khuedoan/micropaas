use std::str::FromStr;
use temporal_client::{Client, RetryClient};
use temporal_sdk::sdk_client_options;
use temporal_sdk_core::protos::coresdk::activity_result::{
    activity_resolution::Status::Completed, ActivityResolution,
};
use temporal_sdk_core::Url;
use tracing::{debug, info};

pub async fn get_client() -> Result<RetryClient<Client>, anyhow::Error> {
    info!("connecting to Temporal");
    let server_options = sdk_client_options(Url::from_str("http://localhost:7233")?).build()?;
    let client = server_options.connect("default", None).await?;
    info!("connected to Temporal");

    Ok(client)
}

pub fn parse_activity_result<'a, T>(result: &'a ActivityResolution) -> Result<T, anyhow::Error>
where
    T: serde::Deserialize<'a>,
{
    if result.completed_ok() {
        if let Some(Completed(result)) = &result.status {
            if let Some(payload) = &result.result {
                // let data = from_utf8(&payload.data).unwrap();
                let result: T = serde_json::from_slice(&payload.data).unwrap();
                // println!("Activity completed with: {:#?}", string_result.to_owned());
                return Ok(result);
            }
        } else {
            debug!("activity failed with {:?}", result.status);
        }
    }
    Err(anyhow::anyhow!("activity failed"))
}
