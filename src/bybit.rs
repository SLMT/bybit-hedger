use hmac::{Hmac, Mac};
use log::trace;
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::Sha256;

use crate::utils;

const RECV_WINDOW: &str = "5000";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub base_coin: String,
    pub total_delta: String,
    pub total_gamma: String,
    pub total_vega: String,
    pub total_theta: String,
    #[serde(rename = "totalRPL")]
    pub total_rpl: String,
    #[serde(rename = "sessionUPL")]
    pub session_upl: String,
    #[serde(rename = "sessionRPL")]
    pub session_rpl: String,
    pub im: String,
    pub mm: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderPlacingInfo {
    pub order_id: String,
    pub order_link_id: String,
    pub symbol: String,
    pub order_type: String,
    pub side: String,
    pub order_qty: String,
    pub order_price: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    pub order_id: String,
    pub order_link_id: String,
    pub symbol: String,
    pub order_type: String,
    pub side: String,
    pub order_status: String,
    pub price: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct List<T> {
    pub result_total_size: i32,
    pub data_list: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    pub ret_code: i32,
    pub ret_msg: String,
    pub result: T,
}

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

    pub fn query_asset_info(&self) -> Response<List<AssetInfo>> {
        self.send_request(
            "{}".to_owned(),
            "https://api.bybit.com/option/usdc/openapi/private/v1/query-asset-info",
        )
    }

    pub fn place_market_order_on_perpetual(
        &self,
        symbol: &str,
        side: &str,
        quantity: Decimal,
    ) -> Response<OrderPlacingInfo> {
        // Build the requedst body
        let mut parameters = Map::new();
        parameters.insert("symbol".to_owned(), Value::String(symbol.to_owned()));
        parameters.insert("orderType".to_owned(), Value::String("Market".to_owned()));
        parameters.insert("orderFilter".to_owned(), Value::String("Order".to_owned()));
        parameters.insert("side".to_owned(), Value::String(side.to_owned()));
        parameters.insert(
            "orderQty".to_owned(),
            Value::String(format!("{}", quantity)),
        );
        let request_body = Value::Object(parameters).to_string();

        self.send_request(
            request_body,
            "https://api.bybit.com/perpetual/usdc/openapi/private/v1/place-order",
        )
    }

    pub fn query_perpetual_order(&self, order_id: &str) -> Response<List<OrderInfo>> {
        // Build the requedst body
        let mut parameters = Map::new();
        parameters.insert("category".to_owned(), Value::String("PERPETUAL".to_owned()));
        parameters.insert("orderId".to_owned(), Value::String(order_id.to_owned()));
        let request_body = Value::Object(parameters).to_string();

        self.send_request(
            request_body,
            "https://api.bybit.com/option/usdc/openapi/private/v1/query-order-history",
        )
    }

    fn send_request<T>(&self, request_body: String, url: &str) -> Response<T>
    where
        T: DeserializeOwned,
    {
        let timestamp = utils::get_unix_epoch_millis();
        let signature = self.sign(&request_body, timestamp);

        trace!("Sending a request to ByBit...");
        trace!("URL: {}", url);
        trace!("Request: {}", &request_body);

        let client = Client::new();
        let response = client
            .post(url)
            .header("X-BAPI-API-KEY", &self.api_key)
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-SIGN-TYPE", "2")
            .header("X-BAPI-TIMESTAMP", format!("{}", timestamp))
            .header("X-BAPI-RECV-WINDOW", RECV_WINDOW)
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .expect("fails to send the request");

        // Parse the response
        let response_body = response.text().expect("cannot read the response body");

        trace!("Get response: {:?}", &response_body);

        serde_json::from_str(&response_body).expect("cannot parse the response body")
    }

    fn sign(&self, request_body: &str, timestamp: u128) -> String {
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
