use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use temporal_sdk::{ActContext, ActivityError};
use tracing::info;

/// Make the http request
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Response {
    args: HashMap<String, String>,
}

pub async fn from_source(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<String, ActivityError> {
    let id = "todo-id";
    info!("Starting http request activity: {}", id);
    let response = reqwest::get(format!("http://httpbin.org/get?answer={}", id))
        .await?
        .json::<Response>()
        .await?;

    info!("Response: {:?}", response);
    if let Some(answer) = response.args.get("answer") {
        return Ok(answer.to_string());
    }

    Err(ActivityError::from(anyhow::anyhow!("No answer found")))
}
