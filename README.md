# AWS Incentives API

A Rust service for using the [Amazon Gift Card API](https://developer.amazon.com/incentives-api) (aka. Amazon Incentives API) to generate Amazon gift cards. 

This library implements the [AWS Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/sigv4_signing.html) signature algorithm.

> "Every request to an endpoint of the Incentives API must be digitally signed using your Incentives API security credentials and the Signature Version 4 signature algorithm. Signing correctly using Signature Version 4 can be the toughest hurdle when calling Incentives API endpoints."
> - https://developer.amazon.com/docs/incentives-api/incentives-api.html

# Usage

Using the service is simple. Instantiate the service with the following credentials, then call the `.generate()` method to generate a new giftcard code:
* PartnerID
* AWS Incentives API Access Key
* AWS Incentives API Secret Key
* AWS Incentives API Hostname
* AWS Incentives API URL

```rust
let service: impl IncentivesService = new_service(
    "PartnerID".to_string(),
    "AccessKey".to_string(),
    "SecretKey".to_string(),
    "agcod-v2-gamma.amazon.com".to_string(),         // Sandbox Host
    "https://agcod-v2-gamma.amazon.com".to_string(), // Sandbox URL
    "us-east-1",                                     // AWS Region
);
let giftcard_code = match service.generate().await {
    Ok(code) => {
        println!("successfully generated a giftcard code: {}", code)
    }
    Err(err) => {
        panic!("failed to generate giftcard code: {}", err)
    }
};
```