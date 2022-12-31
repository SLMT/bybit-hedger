mod bybit;
mod utils;

use std::{cmp::Ordering, env};

use dotenv::dotenv;

use bybit::Bybit;
use log::info;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
enum Action {
    Buy(Decimal),
    Sell(Decimal),
    None,
}

const SYMBOL: &str = "BTCPERP";

fn main() {
    // Enable logging
    set_logger_level();
    pretty_env_logger::init();

    // Read environment variables
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("No API_KEY in .env");
    let api_secret = env::var("API_SECRET").expect("NO API_SECRET in .env");

    // Create a client
    let bybit = Bybit::new(api_key, api_secret);

    // Read the current delta
    let delta = get_current_delta(&bybit);
    info!("Current delta: {}", delta);

    // Check if it is needed to add or reduce positions
    let action = decide_action(delta);
    match action {
        Action::None => info!("Decide to do nothing"),
        _ => info!("Decide to take action: {:?}", action),
    }

    // Buy or sell contracts
    let response = match action {
        Action::Buy(quantity) => {
            Some(bybit.place_market_order_on_perpetual(SYMBOL, "Buy", quantity))
        }
        Action::Sell(quantity) => {
            Some(bybit.place_market_order_on_perpetual(SYMBOL, "Sell", quantity))
        }
        Action::None => None,
    };
    dbg!(response);

    // TODO: check again for the delta

    // TODO: good => wait for the next time to check
}

fn set_logger_level() {
    match std::env::var("RUST_LOG") {
        Ok(_) => {}
        Err(_) => std::env::set_var("RUST_LOG", "bybit_hedger=TRACE"),
    }
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
    match hedging_amount.cmp(&Decimal::ZERO) {
        Ordering::Greater => Action::Buy(hedging_amount),
        Ordering::Less => Action::Sell(hedging_amount),
        Ordering::Equal => Action::None,
    }
}
