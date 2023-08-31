mod engine;
mod errors;
mod helpers;
use engine::orderbook::Order;
use engine::orderbook::Stock;
use uuid::Uuid;
use rand::Rng;

fn main() {
    // create new stock
    let stock = Stock::new(
        Uuid::new_v4(),
        String::from("Apple"),
        String::from("AAPL"),
        Some(1e6 as i32),
        Some(1e6 as i32),
        Some(chrono::Utc::now().timestamp() as u32),
    );
    let mut o_book = engine::orderbook::OrderBook::new(stock.clone());
    let mut orders: Vec<Order> = Vec::new();
    for i in 0..100 {
        let price: f32 = rand::thread_rng().gen_range(90.0..100.0);
        // Round to 2 decimal places
        let rounded_price = (price * 100.0).round() / 100.0;
        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            stock.clone(),
            match rand::thread_rng().gen_range(0..2) {
                0 => engine::orderbook::OrderSide::BID,
                _ => engine::orderbook::OrderSide::ASK,
            },
            match rand::thread_rng().gen_range(0..2) {
                0 => engine::orderbook::OrderType::LIMIT,
                _ => engine::orderbook::OrderType::MARKET,
            },
            rand::thread_rng().gen_range(10..100 ) as i32,
            chrono::Utc::now().timestamp() as u32,
            Some(rounded_price),
        );
        orders.push(order);
    }

    for order in &orders {
        let res = o_book.add_order(order.clone());
        match res {
            Ok(_) => {
                // assert if it's in the orderbook
                print!("checking if order is in orderbook: {} ... ", order.order_id);
                assert!(o_book.get_oid_map().contains_key(&order.order_id));
                println!("Order added successfully!");
            }
            Err(e) => println!("Error adding order: {:?}", e),
        }
    }

    // match orders
    let mut matched_orders: Vec<Order> = Vec::new();
    for o in &orders {

        let res = o_book.match_order(o.clone());
        match res {
            Ok(_) => {
                // assert if it's in the orderbook
                print!("checking if order is in orderbook: {} ... ", o.order_id);
                assert_eq!(o_book.get_oid_map().contains_key(&o.order_id), false);
                println!("Order matched successfully!");
                matched_orders.push(o.clone());
            }
            Err(e) => println!("Error matching order: {:?}", e),
        }
    }
}
