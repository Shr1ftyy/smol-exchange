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


    // function to generate a bunch of orders:
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
    fn test_adding_orders_orderbook() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());
        let orders: Vec<Order> = match generate_triangular_orders(
            500, 
            OrderType::LIMIT, 
            OrderSide::BID, 
            (87.0, 90.0), 
            (10, 100)
        ) 
            { 
            Ok(orders) => orders,
            Err(e) => panic!("Error generating orders: {:?}", e),
        };
        // print lentgh of orders
        println!("Number of orders: {}", orders.len());

        for order in orders {
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
    }

    #[test]
    // matching orders test
    fn test_matching_orders() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());
        let ask_limit_orders = match generate_triangular_orders(500, OrderType::LIMIT, OrderSide::ASK, (87.0, 90.0), (10, 12)){
            Ok(orders) => orders,
            Err(e) => panic!("Error generating orders: {:?}", e),
        };

        // market ask orders (TODO: test market ask order matching?)
        // let ask_orders = match generate_triangular_orders(500, OrderType::MARKET, OrderSide::ASK, (87.0, 90.0), (10, 20)){
        //     Ok(orders) => orders,
        //     Err(e) => panic!("Error generating orders: {:?}", e),
        // };

        let bid_market_orders = match generate_triangular_orders(100, OrderType::MARKET, OrderSide::BID, (85.0, 88.0), (5, 10)){
            Ok(orders) => orders,
            Err(e) => panic!("Error generating orders: {:?}", e),
        };

        let bid_limit_orders = match generate_triangular_orders(100, OrderType::LIMIT, OrderSide::BID, (85.0, 88.0), (5, 10)){
            Ok(orders) => orders,
            Err(e) => panic!("Error generating orders: {:?}", e),
        };

        let mut highest_bid_price = f32::NEG_INFINITY;

        // add ask limit orders into the orderbook
        for order in ask_limit_orders {
            let res = &o_book.add_order(order.clone());
            match res {
                Ok(_) => {
                    // assert if it's in the orderbook
                    print!("checking if order is in orderbook: {} ... ", order.order_id);
                    assert!(o_book.get_oid_map().contains_key(&order.order_id));
                    println!("Order added successfully!");
                    if order.order_side == engine::orderbook::OrderSide::BID {
                        if order.price.unwrap() > highest_bid_price {
                            // round to 2 decimal places before assigning
                            highest_bid_price = (order.price.unwrap() * 100.0).round() / 100.0;
                        }
                    }
                }
                Err(e) => println!("Error adding order: {:?}", e),
            }
        }

        // add bid market orders into the orderbook
        for order in bid_market_orders {
            let res = &o_book.add_order(order.clone());
            match res {
                Ok(_) => {
                    // assert if it's in the orderbook
                    print!("checking if order is in orderbook: {} ... ", order.order_id);
                    assert!(o_book.get_oid_map().contains_key(&order.order_id));
                    println!("Order added successfully!");
                    if order.order_side == engine::orderbook::OrderSide::BID {
                        if order.price.unwrap() > highest_bid_price {
                            // round to 2 decimal places before assigning
                            highest_bid_price = (order.price.unwrap() * 100.0).round() / 100.0;
                        }
                    }
                }
                Err(e) => println!("Error adding order: {:?}", e),
            }
        }

        println!("Orderbook before matching orders: ");
        o_book.print_orderbook();

        let p_level: PriceLevel =
            match o_book.get_price_level(engine::orderbook::OrderSide::BID, highest_bid_price) {
                Some(p) => p.clone(),
                None => panic!("Price level not found!"),
            };

        for order_id in p_level.orders.iter() {
            let res = o_book.match_order(*order_id);
            match res {
                Ok(_) => {
                    // assert if it's in the orderbook 
                    print!("checking if order is in orderbook: {} ... ", order_id);
                    // if order not in oid map, print that it got filled successfully
                    if !o_book.get_oid_map().contains_key(order_id) {
                        println!("Order filled successfully!");
                    } else {
                        println!("Order not filled!");
                    }
                }
                Err(e) => println!("Error matching order: {:?}", e),
            }
        }

        println!("Orderbook after matching orders: ");
        o_book.print_orderbook();
    }
}
