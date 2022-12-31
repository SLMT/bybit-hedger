use hmac::{Hmac, Mac};
use reqwest::blocking::Client;
use sha2::Sha256;

use crate::utils;

const RECV_WINDOW: &'static str = "5000";

pub struct Bybit {
    api_key: String,
    api_secret: String,
}

impl Bybit {
    pub fn new(key: String, secret: String) -> Bybit {
        Bybit {
            api_key: key,
            api_secret: secret,
        }
    }

    pub fn query_asset_info(&self) {
        let request_body = "{}";
        let timestamp = utils::get_unix_epoch_millis();
        let signature = self.sign(request_body, timestamp);

        let client = Client::new();
        let response = client
            .post("https://api.bybit.com/option/usdc/openapi/private/v1/query-asset-info")
            .header("X-BAPI-API-KEY", &self.api_key)
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-SIGN-TYPE", "2")
            .header("X-BAPI-TIMESTAMP", format!("{}", timestamp))
            .header("X-BAPI-RECV-WINDOW", RECV_WINDOW)
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .expect("fails to send the request");

        println!("{}", response.text().unwrap());
    }

    pub fn sign(&self, request_body: &str, timestamp: u128) -> String {
        // Create the context for signing
        let context = format!(
            "{}{}{}{}",
            timestamp, self.api_key, RECV_WINDOW, request_body
        );

        // Create the signing process
        let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
            .expect("size of API_SECRET is not valid");
        mac.update(context.as_bytes());
        let signature = mac.finalize().into_bytes();

        utils::to_hex(&signature)
    }
}
