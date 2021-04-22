use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestPayload {
    // Creation Request must begin with the partner ID followed by a -
    pub(crate) creation_request_id: String,
    pub(crate) partner_id: String,
    pub(crate) value: RequestPayloadValue,
}

impl RequestPayload {
    pub fn new(aws_partner_id: String, amount: i64) -> Self {
        // Create UUID portion of request ID. We must ensure full ID cannot be above 40 char.
        let uuid_segment = Uuid::new_v4().to_string()[0..13].to_string();
        let creation_request_id = format!("{}-{}", aws_partner_id.clone(), uuid_segment);

        Self {
            creation_request_id,
            partner_id: aws_partner_id,
            value: RequestPayloadValue {
                currency_code: "USD".to_string(),
                amount,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestPayloadValue {
    pub(crate) currency_code: String,
    pub(crate) amount: i64,
}
