mod engine;
mod errors;
use uuid::Uuid;

fn main() {
    let order = engine::orderbook::Order::new(
        Uuid::new_v4(),
        1,
        1,
        engine::orderbook::OrderType::BUY,
        100,
        100,
        100,
    );

    let engine: engine::engine::MatchingEngine = engine::engine::MatchingEngine::new("redis://127.0.0.1:6379/");
    let _ = engine.add_order(order);
    println!("Hello, world!");
}
