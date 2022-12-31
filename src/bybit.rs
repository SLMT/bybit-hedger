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
}
