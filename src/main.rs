mod bybit;
mod utils;

use std::{cmp::Ordering, env, thread};

use chrono::{DurationRound, Local, Timelike};
use dotenv::dotenv;

use bybit::Bybit;
use log::info;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
enum Action {
    Buy(Decimal),
    Sell(Decimal),
}

const SYMBOL: &str = "BTCPERP";

type StdDuration = std::time::Duration;
type ChDuration = chrono::Duration;

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

    loop {
        info!("Current time: {}", Local::now().format("%F %T"));

        // Read the current delta
        let delta = get_current_delta(&bybit);
        info!("Current delta: {}", delta);

        // Check if it is needed to add or reduce positions
        let action = decide_action(delta);

        // Buy or sell contracts
        if let Some(action) = action {
            info!("Decide to take action: {:?}", action);

            // Place an order
            let response = match action {
                Action::Buy(quantity) => {
                    bybit.place_market_order_on_perpetual(SYMBOL, "Buy", quantity)
                }
                Action::Sell(quantity) => {
                    bybit.place_market_order_on_perpetual(SYMBOL, "Sell", quantity)
                }
            };

            info!(
                "Order is placed successfully. Order ID: {}.",
                response.result.order_id
            );

            info!("Wait for a second to check the order...");
            thread::sleep(StdDuration::from_secs(1));

            // Check if the order is filled
            loop {
                let response = bybit.query_perpetual_order(&response.result.order_id);
                dbg!(&response);
                if response.result.data_list[0].order_status == "Filled" {
                    break;
                }
                info!("The order is not filled yet.");

                thread::sleep(StdDuration::from_secs(5));
            }

            info!("The order is filled.");

            // Check again for the delta
            let delta = get_current_delta(&bybit);
            info!("New delta: {}", delta);
        } else {
            info!("Decide to do nothing");
        }

        // Wait for the next time to check
        let sleep_len = get_duration_to_next_check_time();
        thread::sleep(sleep_len);
    }
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

fn decide_action(delta: Decimal) -> Option<Action> {
    let hedging_amount = -delta.round_dp(3);
    match hedging_amount.cmp(&Decimal::ZERO) {
        Ordering::Greater => Some(Action::Buy(hedging_amount)),
        Ordering::Less => Some(Action::Sell(hedging_amount)),
        Ordering::Equal => None,
    }
}

fn get_duration_to_next_check_time() -> StdDuration {
    let now = Local::now();
    let target = if now.minute() >= 10 {
        let t = now + ChDuration::hours(1);
        t.duration_trunc(ChDuration::hours(1)).unwrap() + ChDuration::minutes(10)
    } else {
        now.duration_trunc(ChDuration::hours(1)).unwrap() + ChDuration::minutes(10)
    };
    info!("Next check time: {}", target.format("%F %T"));
    (target - now).to_std().unwrap()
}
