mod engine;
mod errors;
mod helpers;
#[cfg(test)]
mod tests {
    use super::*;
    use engine::orderbook::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;
    use uuid::Uuid;
    use rand_distr::{Distribution, Triangular, TriangularError};

    const SEED: u64 = 69420;
    fn generate_orders() -> Vec<Order> {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let mut orders: Vec<Order> = Vec::new();
        // Bid Orders
        for i in 0..100 {
            let price: f32 = rng.gen_range(87.00..90.00);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i),
                    format!("STK{}", i),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(chrono::Utc::now().timestamp() as u32),
                ),
                engine::orderbook::OrderSide::BID,
                match rng.gen_range(0..2) {
                    0 => engine::orderbook::OrderType::LIMIT,
                    _ => engine::orderbook::OrderType::MARKET,
                },
                rng.gen_range(10..100) as i32,
                chrono::Utc::now().timestamp() as u32,
                Some(rounded_price),
            );
            orders.push(order);
        }

        // Ask Orders
        for i in 0..100 {
            let price: f32 = rng.gen_range(89.00..92.00);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i), // TODO: names, tickers should be consistent with orderbook ticker
                    format!("STK{}", i),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(chrono::Utc::now().timestamp() as u32),
                ),
                engine::orderbook::OrderSide::ASK,
                match rng.gen_range(0..2) {
                    0 => engine::orderbook::OrderType::LIMIT,
                    _ => engine::orderbook::OrderType::MARKET,
                },
                rng.gen_range(10..100) as i32,
                chrono::Utc::now().timestamp() as u32,
                Some(rounded_price),
            );
            orders.push(order);
        }
        orders
    }

    // a function for generating orders in a triangular distribution using rand_distr, 
    // lets the user specify what type of orders to generate, the amount, price range, etc.
    fn generate_triangular_orders(
        num_orders: usize,
        order_type: engine::orderbook::OrderType,
        order_side: engine::orderbook::OrderSide,
        price_range: (f32, f32),
        quantity_range: (i32, i32),
    ) -> Result<Vec<Order>, TriangularError> {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let mut orders: Vec<Order> = Vec::new();
        let triangular: Triangular<f32> = match Triangular::new(price_range.0, price_range.1, price_range.0){ 
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        for i in 0..num_orders {
            let price: f32 = triangular.sample(&mut rng);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i), // TODO: names, tickers should be consistent with orderbook ticker
                    format!("STK{}", i),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(rng.gen_range(1e6..1e7) as i32),
                    Some(chrono::Utc::now().timestamp() as u32),
                ),
                order_side,
                order_type,
                rng.gen_range(quantity_range.0..quantity_range.1) as i32,
                chrono::Utc::now().timestamp() as u32,
                Some(rounded_price),
            );
            orders.push(order);
        }
        Ok(orders)
    }

    #[test]
    fn test_adding_orders() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());
        let orders = generate_orders();
        for order in orders {
            o_book.add_order(order);
        }

        // test number of orders in the orderbook
        assert_eq!(o_book.oid_map.len(), 200);
    }

    #[test]
    fn test_matching_orders() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());

        // scenarios to test:
        // 1. limit asks in order book -> add limit bid -> match
        let mut limit_asks = Vec::new();

        for i in 0..6 {
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i),
                    format!("STK{}", i),
                    None, None, None
                ),
                engine::orderbook::OrderSide::ASK,
                engine::orderbook::OrderType::LIMIT,
                100,
                chrono::Utc::now().timestamp() as u32,
                Some(88.0 + (0.5 * (i as f32))),
            );
            limit_asks.push(order);
        }

        for ask in limit_asks {
            o_book.queue_order(ask);
        }

        let mut limit_bids = Vec::new();
        for i in 0..6 {
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i),
                    format!("STK{}", i),
                    None, None, None
                ),
                engine::orderbook::OrderSide::BID,
                engine::orderbook::OrderType::LIMIT,
                100,
                chrono::Utc::now().timestamp() as u32,
                Some(91.5 - (0.5 * (i as f32))),
            );
            limit_bids.push(order);
        }

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        o_book.print_orderbook();

        for bid in limit_bids {
            o_book.queue_order(bid);
        }

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        o_book.print_orderbook();

        // 2. limit bids in order book -> add limit ask -> match
        // 3. limit asks in order book -> add market bid -> match
        // 4. limit bids in order book -> add market ask -> match
        // 5. market asks in order book -> add limit bid -> match
        // 6. market bids in order book -> add limit ask -> match
        // 7. market asks in order book -> add market bid -> match
        // 8. market bids in order book -> add market ask -> match
        // 9. no asks in order book -> add limit bid -> no match
        // 10. no bids in order book -> add limit ask -> no match
        // 11. no asks in order book -> add market bid -> no match
        // 12. no bids in order book -> add market ask -> no match

    }
}
