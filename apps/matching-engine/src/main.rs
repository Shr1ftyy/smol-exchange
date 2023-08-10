mod engine;
mod errors;
use engine::engine::Matching;
use engine::engine::MatchingEngine;
use engine::orderbook::Order;
use engine::orderbook::Stock;
use uuid::Uuid;

fn main() {
    // initialize stock
    let stock: Stock = Stock::new(
        uuid::Uuid::new_v4(),
        "AAPL".to_string(),
        "Apple".to_string(),
        Some(1e6 as i32),
        Some(1e6 as i32),
        None,
    );

    let order = Order::new(
        Uuid::new_v4(),
        1,
        stock,
        engine::orderbook::OrderSide::BUY,
        engine::orderbook::OrderType::LIMIT,
        100.0,
        100,
        100,
    );

    let mut engine: MatchingEngine = Matching::new("redis://127.0.0.1:6379/");
    let _ = engine.add_order(order);
    println!("Hello, world!");
}
