mod bybit;
mod utils;

use std::env;

use dotenv::dotenv;

use bybit::Bybit;

fn main() {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("No API_KEY in .env");
    let api_secret = env::var("API_SECRET").expect("NO API_SECRET in .env");
    let bybit = Bybit::new(api_key, api_secret);

    // TODO: read the current delta
    let result = bybit.query_asset_info();
    dbg!(result);

    // TODO: check if it is needed to add or reduce positions

    // TODO: buy or sell contracts

    // TODO: wait for the results

    // TODO: check again for the delta

    // TODO: good => wait for the next time to check
}
