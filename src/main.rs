mod bybit;
mod utils;

use std::env;

use dotenv::dotenv;

use bybit::Bybit;
use rust_decimal::Decimal;

#[derive(Debug)]
enum Action {
    Buy(Decimal),
    Sell(Decimal),
    None,
}

fn main() {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("No API_KEY in .env");
    let api_secret = env::var("API_SECRET").expect("NO API_SECRET in .env");
    let bybit = Bybit::new(api_key, api_secret);

    // Read the current delta
    let delta = get_current_delta(&bybit);

    // Check if it is needed to add or reduce positions
    let action = decide_action(delta);
    dbg!(action);

    // TODO: buy or sell contracts

    // TODO: check again for the delta

    // TODO: good => wait for the next time to check
}

fn get_current_delta(bybit: &Bybit) -> Decimal {
    let response = bybit.query_asset_info();
    let list = response.result.data_list;

    if list.is_empty() {
        panic!("No available assets");
    }

    list[0].total_delta.parse().unwrap()
}

fn decide_action(delta: Decimal) -> Action {
    let hedging_amount = -delta.round_dp(3);
    if hedging_amount > Decimal::ZERO {
        Action::Buy(hedging_amount)
    } else if hedging_amount < Decimal::ZERO {
        Action::Sell(hedging_amount)
    } else {
        Action::None
    }
}
