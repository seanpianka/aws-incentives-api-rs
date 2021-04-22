use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};

use crate::request::RequestPayload;
use crate::response::ResponsePayload;

type HmacSha256 = Hmac<Sha256>;

#[async_trait]
pub trait IncentivesService {
    /// Generate a new Amazon gift card.
    async fn generate(&self) -> Result<String, String>;
}

#[async_trait]
impl IncentivesService for ServiceImpl {
    async fn generate(&self) -> Result<String, String> {
        let now = Utc::now();
        self.generate_giftcard_code(now).await
    }
}

pub fn new_service(
    aws_partner_id: String,
    aws_incentives_access_key: String,
    aws_incentives_secret_signing_key: String,
    aws_incentives_host: String,
    aws_incentives_url: String,
) -> impl IncentivesService {
    ServiceImpl::new(
        aws_partner_id,
        aws_incentives_access_key,
        aws_incentives_secret_signing_key,
        aws_incentives_host,
        aws_incentives_url,
    )
}

struct ServiceImpl {
    aws_partner_id: String,
    aws_incentives_secret_signing_key: String,
    aws_incentives_access_key: String,
    aws_incentives_host: String,
    aws_incentives_url: String,
    aws_region_name: String,
    incentive_amount: i64,
}

impl ServiceImpl {
    const SERVICE_NAME: &'static str = "AGCODService";

    pub fn new(
        aws_partner_id: String,
        aws_incentives_access_key: String,
        aws_incentives_secret_signing_key: String,
        aws_incentives_host: String,
        aws_incentives_url: String,
    ) -> Self {
        Self {
            aws_partner_id,
            aws_incentives_access_key,
            aws_incentives_secret_signing_key,
            aws_incentives_host,
            aws_incentives_url,
            aws_region_name: "us-east-1".to_string(),
            incentive_amount: 5,
        }
    }

    async fn generate_giftcard_code(&self, time: DateTime<Utc>) -> Result<String, String> {
        let time = time
            .to_rfc3339_opts(SecondsFormat::Secs, true)
            .replace(":", "")
            .replace("-", "");
        let payload = RequestPayload::new(self.aws_partner_id.clone(), self.incentive_amount);
        let canonical_hash = self.generate_canonical_request(time.clone(), &payload);
        let string_to_sign = self.build_string_to_sign(canonical_hash, time.as_str());
        let auth_signature = self.generate_authorization_header(string_to_sign, time.clone());

        let headers: HeaderMap = {
            let amazon_target = format!(
                "com.amazonaws.agcod.{}.CreateGiftCard",
                ServiceImpl::SERVICE_NAME
            );

            let mut h = HashMap::new();
            h.insert("host", self.aws_incentives_host.as_str());
            h.insert("x-amz-date", time.as_str());
            h.insert("x-amz-target", amazon_target.as_str());
            h.insert("accept", "application/json");
            h.insert("content-type", "application/json");
            h.insert("regionName", self.aws_region_name.as_str());
            h.insert("serviceName", ServiceImpl::SERVICE_NAME);
            h.insert("Authorization", auth_signature.as_str());
            println!("{}", serde_json::to_string_pretty(&h).unwrap());
            headermap_from_hashmap(h.iter())
        };

        let payload = serde_json::to_string(&payload).unwrap();
        let client = reqwest::Client::builder()
            .default_headers(headers.into())
            .build()
            .unwrap();
        match client
            .post(format!(
                "{}/CreateGiftCard",
                self.aws_incentives_url.as_str()
            ))
            .body(payload)
            .send()
            .await
        {
            Ok(resp) => {
                let bytes = match resp.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        return Err(format!(
                            "failed to stream bytes from response: {}",
                            e.to_string()
                        ))
                    }
                };
                println!("{}", String::from_utf8(bytes.to_vec()).unwrap());
                match serde_json::from_slice::<ResponsePayload>(&bytes) {
                    Ok(resp) => Ok(resp.claim_code().to_string()),
                    Err(err) => Err(format!(
                        "failed to deserialize response bytes to payload type: {}\npayload: {}",
                        err.to_string(),
                        String::from_utf8(bytes.to_vec()).unwrap(),
                    )),
                }
            }
            Err(err) => Err(format!(
                "failed to send POST request to incentives API: {}",
                err.to_string()
            )),
        }
    }

    /// Generates the signature to put in the POST message header 'Authorization'
    fn generate_authorization_header(&self, string_to_sign: String, date: String) -> String {
        let derived_key = self.build_derived_key(date.clone());
        let final_signature = hmac_sha256(string_to_sign.as_str(), derived_key.as_ref());
        let abridged_date = &date[0..8];
        let credential = format!(
            "Credential={}/{}/{}/{}/aws4_request",
            self.aws_incentives_access_key,
            abridged_date,
            self.aws_region_name,
            ServiceImpl::SERVICE_NAME,
        );
        let signed_headers = "SignedHeaders=accept;host;x-amz-date;x-amz-target";

        format!(
            "AWS4-HMAC-SHA256 {}, {}, Signature={}",
            credential,
            signed_headers,
            hex::encode(final_signature)
        )
    }

    /// Creates a printout of all information sent to the AGCOD service
    fn generate_canonical_request(&self, date: String, payload: &RequestPayload) -> String {
        let payload = serde_json::to_string(payload).unwrap();
        let payload = hex::encode(sha256(payload.trim()));
        let request = format!(
            "POST
/CreateGiftCard

accept:application/json
host:agcod-v2-gamma.amazon.com
x-amz-date:{}
x-amz-target:com.amazonaws.agcod.{}.CreateGiftCard

accept;host;x-amz-date;x-amz-target
{}
        ",
            date,
            ServiceImpl::SERVICE_NAME,
            payload
        );
        println!("{}", request.as_str());
        hex::encode(sha256(request.trim()))
    }

    /// Uses the previously calculated canonical request to create a single "String to Sign" for the request
    fn build_string_to_sign(&self, canonical_request_hash: String, date: &str) -> String {
        let abridged_date = &date[0..8];
        format!(
            "AWS4-HMAC-SHA256\n{}\n{}/{}/{}/aws4_request\n{}",
            date,
            abridged_date,
            self.aws_region_name,
            ServiceImpl::SERVICE_NAME,
            canonical_request_hash
        )
    }

    /// Create a derived key based on the secret key and parameters related to the call
    fn build_derived_key(&self, date: String) -> Vec<u8> {
        let signature_aws_key = format!("AWS4{}", self.aws_incentives_secret_signing_key);
        let abridged_date = &date[0..8];
        let date_key = hmac_sha256(abridged_date, signature_aws_key.as_bytes());
        let region_key = hmac_sha256(self.aws_region_name.as_str(), date_key.as_ref());
        let service_key = hmac_sha256(ServiceImpl::SERVICE_NAME, region_key.as_ref());
        let signing_key = hmac_sha256("aws4_request", service_key.as_ref());
        signing_key
    }
}

fn hmac_sha256(value: &str, key: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_varkey(key).expect("HMAC can take key of any size");
    mac.update(value.as_bytes());
    mac.finalize().into_bytes().to_owned().to_vec()
}

fn sha256(value: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher.finalize().to_vec()
}

/// Convert HashMap<&str, &str> to HeaderMap
/// https://github.com/seanmonstar/reqwest/issues/555#issuecomment-507566071
fn headermap_from_hashmap<'a, I, S>(headers: I) -> HeaderMap
where
    I: Iterator<Item = (S, S)> + 'a,
    S: AsRef<str> + 'a,
{
    headers
        .map(|(name, val)| {
            (
                HeaderName::from_str(name.as_ref()),
                HeaderValue::from_str(val.as_ref()),
            )
        })
        // We ignore the errors here. If you want to get a list of failed conversions, you can use Iterator::partition
        // to help you out here
        .filter(|(k, v)| k.is_ok() && v.is_ok())
        .map(|(k, v)| (k.unwrap(), v.unwrap()))
        .collect()
}
