mod engine;
mod errors;
#[cfg(test)]
mod tests {
    use super::*;
    use engine::engine::Management;
    use engine::engine::Matching;
    use engine::engine::MatchingEngine;
    use engine::orderbook::Order;
    use engine::orderbook::Stock;
    use rand::Rng;
    use uuid::Uuid;

    #[test]
    fn orders() {
        // generate a small number of stocks, and a bunch of orders for each stock
        // keep price ranges for the stocks small, so that we can test the matching engine
        // with a small number of orders
        let mut stocks: Vec<Stock> = Vec::new();
        let mut orders: Vec<Order> = Vec::new();

        for i in 0..10 {
            let stock: Stock = Stock::new(
                uuid::Uuid::new_v4(),
                format!("Stock {}", i),
                format!("STK{}", i),
                Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                Some(chrono::Utc::now().timestamp() as u32),
            );
            stocks.push(stock);
        }

        for i in 0..100 {
            let stock: &Stock = stocks.get(i % 10).unwrap();
            let order = Order::new(
                Uuid::new_v4(),
                1,
                stock.clone(),
                match rand::thread_rng().gen_range(0..2) {
                    0 => engine::orderbook::OrderSide::BUY,
                    _ => engine::orderbook::OrderSide::SELL,
                },
                match rand::thread_rng().gen_range(0..2) {
                    0 => engine::orderbook::OrderType::LIMIT,
                    _ => engine::orderbook::OrderType::MARKET,
                },
                rand::thread_rng().gen_range(90..100) as f32,
                rand::thread_rng().gen_range(10..100) as i32,
                chrono::Utc::now().timestamp() as u32,
            );
            orders.push(order);
        }

        // initialize matching engine
        let mut engine: MatchingEngine = Matching::new("redis://127.0.0.1:6379/");
        // connect to redis with management trait
        let mut manager: MatchingEngine = Management::new("redis://127.0.0.1:6379/");

        // add stocks to engine with manager
        for stock in stocks {
            let res = manager.add_stock(stock);
            match res {
                Ok(_) => println!("Stock added successfully!"),
                Err(e) => println!("Error adding stock: {:?}", e),
            }
        }

        // add orders to engine
        for order in orders {
            let res = engine.add_order(order);
            match res {
                Ok(_) => println!("Order added successfully!"),
                Err(e) => println!("Error adding order: {:?}", e),
            }
        }
    }
}
