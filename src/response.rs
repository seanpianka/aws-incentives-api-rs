use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct ResponsePayload {
    card_info: CardInfo,
    creation_request_id: String,
    gc_claim_code: String,
    gc_expiration_date: Option<String>,
    gc_id: String,
    status: String,
}

impl ResponsePayload {
    pub fn claim_code(&self) -> &str {
        self.gc_claim_code.as_str()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CardInfo {
    card_number: Option<String>,
    card_status: String,
    expiration_date: Option<String>,
    value: CardValue,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CardValue {
    amount: f64,
    currency_code: String,
}
