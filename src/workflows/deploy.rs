use crate::temporal::parse_activity_result;
use prost_wkt_types::Duration as ProstDuration;
use std::time::Duration;
use temporal_sdk::ActivityOptions;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core::protos::temporal::api::common::v1::RetryPolicy;
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use tracing::{debug, error, info};

pub async fn deploy_from_source(ctx: WfContext) -> WorkflowResult<String> {
    debug!("inside deploy workflow");
    let act_handle = ctx
        .activity(ActivityOptions {
            activity_type: "from_source".to_string(),
            input: "".as_json_payload()?, // no actual payload
            retry_policy: Some(RetryPolicy {
                initial_interval: Some(ProstDuration {
                    seconds: 0,
                    nanos: 50_000_000, // %0ms
                }),
                maximum_attempts: 2,
                ..Default::default()
            }),
            start_to_close_timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        })
        .await;

    match parse_activity_result::<String>(&act_handle) {
        Ok(result) => {
            info!("activity completed with: {:#?}", result);
            Ok(WfExitValue::Normal(result))
        }
        Err(_) => {
            error!("activity failed");
            Ok(WfExitValue::Evicted)
        }
    }
}
