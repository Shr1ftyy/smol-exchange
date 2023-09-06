// TODO: Add helper functions for the tests, a lot of the code patterns I'm seeing
// are repeated in the tests, so it would be nice to have a helper function to make
// everything more concise and readable
#[cfg(test)]
mod tests {
    use futures_util::StreamExt as _;
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Triangular, TriangularError};
    use redis::PubSubCommands;
    use smolexchange::engine::engine::MatchingEngine;
    use smolexchange::engine::orderbook::*;
    use smolexchange::engine::*;
    use tokio;
    use uuid::Uuid;

    const SEED: u64 = 69420;
    fn generate_orders(stock: Stock) -> Vec<Order> {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let mut orders: Vec<Order> = Vec::new();
        // Bid Orders
        for i in 0..100 {
            let price: f32 = rng.gen_range(87.00..90.00);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                stock.clone(),
                orderbook::OrderSide::BID,
                match rng.gen_range(0..2) {
                    0 => orderbook::OrderType::LIMIT,
                    _ => orderbook::OrderType::MARKET,
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
                Uuid::new_v4(),
                stock.clone(),
                orderbook::OrderSide::ASK,
                match rng.gen_range(0..2) {
                    0 => orderbook::OrderType::LIMIT,
                    _ => orderbook::OrderType::MARKET,
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
        stock: Stock,
        num_orders: usize,
        order_type: orderbook::OrderType,
        order_side: orderbook::OrderSide,
        price_range: (f32, f32),
        quantity_range: (i32, i32),
    ) -> Result<Vec<Order>, TriangularError> {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let mut orders: Vec<Order> = Vec::new();
        let triangular: Triangular<f32> =
            match Triangular::new(price_range.0, price_range.1, price_range.1) {
                Ok(t) => t,
                Err(e) => return Err(e),
            };

        let qty_sample = match Triangular::new(
            quantity_range.0 as f32,
            quantity_range.1 as f32,
            quantity_range.0 as f32,
        ) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

        for i in 0..num_orders {
            let price: f32 = triangular.sample(&mut rng);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                stock.clone(),
                order_side,
                order_type,
                qty_sample.sample(&mut rng) as i32,
                chrono::Utc::now().timestamp() as u32,
                Some(rounded_price),
            );
            orders.push(order);
        }
        Ok(orders)
    }

    fn gen_orders(
        stock: Stock,
        num_orders: usize,
        order_side: orderbook::OrderSide,
        order_type: orderbook::OrderType,
        order_qty: i32,
        starting_price: Option<f32>,
        order_inc: Option<f32>,
    ) -> Vec<Order> {
        let mut orders = Vec::new();

        for i in 0..num_orders {
            let order = Order::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                stock.clone(),
                order_side,
                order_type,
                order_qty,
                chrono::Utc::now().timestamp() as u32,
                match starting_price {
                    Some(p) => Some(p + (i as f32 * order_inc.unwrap())),
                    None => None,
                },
            );
            orders.push(order);
        }

        orders
    }

    #[test]
    fn test_adding_orders_pricelevel() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );
        let mut p_level = PriceLevel::new(1.0, 0);
        let orders = gen_orders(
            stock,
            10,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );
        for order in orders {
            p_level.add_order(order);
        }

        // test number of orders in the price level
        assert_eq!(p_level.orders.len(), 10);
    }

    #[test]
    fn test_removing_orders_pricelevel() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );
        let mut p_level = PriceLevel::new(1.0, 0);

        let orders = gen_orders(
            stock,
            10,
            OrderSide::BID,
            OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );

        for order in orders.clone() {
            p_level.add_order(order.clone());
        }

        // test number of orders in the price level
        assert_eq!(p_level.orders.len(), 10);

        // remove 5 orders
        for order in orders[0..5].iter() {
            p_level.remove_order(order.clone());
        }

        // test number of orders in the price level
        assert_eq!(p_level.orders.len(), 5);
    }

    #[test]
    // TODO: improve this test
    fn test_get_oid_map() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );
        let mut o_book = orderbook::OrderBook::new(stock.clone());
        let orders = gen_orders(
            stock.clone(),
            200,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );

        for order in orders {
            o_book.add_order(order);
        }

        // test number of orders in the orderbook
        assert_eq!(o_book.oid_map.len(), 200);
    }

    #[test]
    fn test_queue_order() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        let mut o_book = orderbook::OrderBook::new(stock.clone());
        let orders = gen_orders(
            stock.clone(),
            200,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );

        for order in orders {
            o_book.queue_order(order);
        }

        // test number of orders in the orderbook
        assert_eq!(o_book.order_queue.len(), 200);
    }

    #[test]
    fn test_adding_orders() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        let mut o_book = orderbook::OrderBook::new(stock.clone());
        let orders = gen_orders(
            stock.clone(),
            200,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );

        for order in orders {
            o_book.add_order(order);
        }

        // test number of orders in the orderbook
        assert_eq!(o_book.oid_map.len(), 200);
    }

    #[test]
    fn test_get_price_level_orderbook() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        let mut o_book = orderbook::OrderBook::new(stock.clone());
        let orders = gen_orders(
            stock.clone(),
            200,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(1.0),
            Some(0.0),
        );

        for order in orders {
            match o_book.add_order(order) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        }

        // test number of orders in the orderbook
        assert_eq!(o_book.oid_map.len(), 200);

        // test get price level
        let price_level = match o_book.get_price_level(OrderSide::BID, 1.0) {
            Some(p) => Some(p.clone()),
            None => None,
        };

        assert_eq!(price_level.clone().unwrap().orders.len(), 200);
        // test price
        assert_eq!(price_level.clone().unwrap().price, 1.0);
        // test quantity
        assert_eq!(price_level.clone().unwrap().qty, 20000);
    }

    #[test]
    fn test_matching_orders() {
        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        let mut o_book = orderbook::OrderBook::new(stock.clone());

        // scenarios to test:
        // 1. limit asks in order book -> add limit bid -> match
        println!("1. limit asks in order book -> add limit bid -> match");

        let limit_asks = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            Some(88.0),
            Some(0.5),
        );

        for ask in limit_asks {
            o_book.queue_order(ask);
        }

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        o_book.print_orderbook();

        let limit_bids = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(88.0),
            Some(0.5),
        );

        for bid in limit_bids {
            o_book.queue_order(bid);
        }

        // check order map and price levels
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 7);

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        o_book.print_orderbook();

        // check order map and price levels
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);

        // 2. limit bids in order book -> add limit ask -> match
        println!("2. limit bids in order book -> add limit ask -> match");

        let limit_bids1 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(88.0),
            Some(0.5),
        );

        for bid in limit_bids1 {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map, price levels, and last market price
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 7);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(91.0));

        let limit_asks1 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            Some(91.0),
            Some(-0.5),
        );

        for ask in limit_asks1 {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(88.0));

        // 3. limit asks in order book -> add market bid -> match
        println!("3. limit asks in order book -> add market bid -> match");

        let limit_asks2 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            Some(88.0),
            Some(0.5),
        );

        for ask in limit_asks2 {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 7);
        assert_eq!(o_book.last_market_price, Some(88.0));

        let market_bids1 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for bid in market_bids1 {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(91.0));

        // 4. limit bids in order book -> add market ask -> match
        println!("4. limit bids in order book -> add market ask -> match");

        let limit_bids2 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(91.0),
            Some(-0.5),
        );

        for bid in limit_bids2 {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 7);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(91.0));

        let market_asks1 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for ask in market_asks1 {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(88.0));

        // 5. market asks in order book -> add limit bid -> match
        println!("5. market asks in order book -> add limit bid -> match");

        o_book.last_market_price = Some(69.0);

        let market_asks2 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for ask in market_asks2 {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 1);
        assert_eq!(o_book.last_market_price, Some(69.0));

        let limit_bids3 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            100,
            Some(88.0),
            Some(0.5),
        );

        for bid in limit_bids3 {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(69.0));

        // 6. market bids in order book -> add limit ask -> match
        println!("6. market bids in order book -> add limit ask -> match");

        o_book.last_market_price = Some(69.0);

        let market_bids1 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for bid in market_bids1.clone() {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 1);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(69.0));

        let limit_asks2 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            Some(69.0),
            Some(0.5),
        );

        for ask in limit_asks2.clone() {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 12);
        assert_eq!(o_book.bid_price_levels.len(), 1);
        assert_eq!(o_book.ask_price_levels.len(), 6);
        assert_eq!(o_book.last_market_price, Some(69.0));

        for order in limit_asks2.clone() {
            match o_book.delete_order(order.order_id) {
                Ok(_) => (),
                Err(e) => println!("Error deleting order: {}", e),
            }
        }

        for order in market_bids1.clone() {
            match o_book.delete_order(order.order_id) {
                Ok(_) => (),
                Err(e) => println!("Error deleting order: {}", e),
            }
        }

        // 7. market asks in order book -> add market bid -> match
        println!("7. market asks in order book -> add market bid -> match");

        o_book.last_market_price = Some(420.0);

        let market_asks3 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for ask in market_asks3 {
            o_book.queue_order(ask);
        }

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 1);
        assert_eq!(o_book.last_market_price, Some(420.0));

        let market_bids2 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for bid in market_bids2 {
            o_book.queue_order(bid);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(420.0));

        // 8. market bids in order book -> add market ask -> match
        println!("8. market bids in order book -> add market ask -> match");

        o_book.last_market_price = Some(21.0);

        let market_bids3 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::BID,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for bid in market_bids3 {
            o_book.queue_order(bid);
        }

        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 7);
        assert_eq!(o_book.bid_price_levels.len(), 1);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(21.0));

        let market_asks4 = gen_orders(
            stock.clone(),
            7,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::MARKET,
            100,
            None,
            None,
        );

        for ask in market_asks4 {
            o_book.queue_order(ask);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 0);
        assert_eq!(o_book.bid_price_levels.len(), 0);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(21.0));

        // 9. ??? random order types -> match
        println!("9. ??? random order types -> match");

        o_book.last_market_price = Some(60.0);

        // generate orders from price range 60.0 to 65.0 in ranges in 0.5 increments.
        // 100 orders total, and ensure the quantity at each price level decreases

        let stock_id: Uuid = Uuid::new_v4();
        let mut bids: Vec<Order> = Vec::new();

        for j in 0..10 {
            for i in 0..20 {
                let price: f32 = 65.0;
                // Round to 2 decimal places
                let rounded_price = (price * 100.0).round() / 100.0;
                let order = Order::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Stock::new(
                        stock_id,
                        "Stock 1".to_string(),
                        "STK 1".to_string(),
                        None,
                        None,
                        None,
                    ),
                    orderbook::OrderSide::BID,
                    orderbook::OrderType::LIMIT,
                    (10 - j) as i32,
                    chrono::Utc::now().timestamp() as u32,
                    Some(rounded_price - (j as f32 * 0.5)),
                );
                bids.push(order);
            }
        }

        for order in bids {
            o_book.queue_order(order);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check order queue, order map and price_levels
        assert_eq!(o_book.order_queue.len(), 0);
        assert_eq!(o_book.oid_map.len(), 200);
        assert_eq!(o_book.bid_price_levels.len(), 10);
        assert_eq!(o_book.ask_price_levels.len(), 0);
        assert_eq!(o_book.last_market_price, Some(60.0));

        let mut asks = Vec::new();

        for j in 0..10 {
            for _ in 0..20 {
                let price: f32 = 66.0;
                // Round to 2 decimal places
                let rounded_price = (price * 100.0).round() / 100.0;
                let order = Order::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Stock::new(
                        stock_id,
                        "Stock 1".to_string(),
                        "STK 1".to_string(),
                        None,
                        None,
                        None,
                    ),
                    orderbook::OrderSide::ASK,
                    orderbook::OrderType::LIMIT,
                    (10 - j) as i32,
                    chrono::Utc::now().timestamp() as u32,
                    Some(rounded_price + (j as f32 * 0.5)),
                );
                asks.push(order);
            }
        }

        for order in asks {
            o_book.queue_order(order);
        }

        // execute orders
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // create a new order with price of 65.0 and qty of 10
        // for stock with ticker STK 1
        let order0 = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            stock,
            orderbook::OrderSide::BID,
            orderbook::OrderType::LIMIT,
            10,
            chrono::Utc::now().timestamp() as u32,
            Some(66.0),
        );
        // queue the order
        o_book.queue_order(order0.clone());
        // execute the order
        match o_book.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("Error executing orders: {}", e),
        }

        // print orderbook
        o_book.print_orderbook();

        // check the order map and the price levels
        assert_eq!(o_book.oid_map.len(), 399);
        assert_eq!(o_book.bid_price_levels.len(), 10);
        // check that the top bid price level qty is 190
        let p_level = match o_book.get_price_level(OrderSide::ASK, 66.0) {
            Some(p) => p.clone(),
            None => panic!("Error getting price level"),
        };
        assert_eq!(p_level.qty, 190);
        assert_eq!(o_book.ask_price_levels.len(), 10);
        assert_eq!(o_book.last_market_price, Some(66.0));
    }

    // test adding a stock to the exchange
    #[test]
    fn test_exchange_add_stock() {
        // create new exchange
        let mut exchange = Exchange::new();

        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        // create new user
        let issuer = User::new(
            Uuid::new_v4(),
            String::from("John"),
            String::from("john.doe@gmail.com"),
            String::from("password"),
            Some(1e6 as f32),
        );

        // add stock to exchange
        exchange.add_stock(stock, issuer);

        // test number of stocks in exchange
        assert_eq!(exchange.stocks.len(), 1);

        // test to check users and userstocks
        assert_eq!(exchange.users.len(), 1);
        assert_eq!(exchange.user_stocks.len(), 1);
    }

    // test executing order in exchange
    #[test]
    fn test_exchange_execute_order() {
        // create new exchange
        let mut exchange = Exchange::new();

        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        // create new user
        let issuer = User::new(
            Uuid::new_v4(),
            String::from("John"),
            String::from("john.doe@gmail.com"),
            String::from("password"),
            Some(1e6 as f32),
        );

        // add stock to exchange
        exchange.add_stock(stock.clone(), issuer);

        // create new order
        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            stock.clone(),
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            chrono::Utc::now().timestamp() as u32,
            Some(69.0),
        );

        // execute order, and check execution type (should be add)
        let res: Execution = match exchange.execute_order(order) {
            Ok(r) => r,
            Err(e) => panic!("Error executing order: {}", e),
        };
        // check execution type
        assert_eq!(res.exec_type, orderbook::ExecutionType::ADD);
        println!("{}", res);

        // test number of orders in exchange
        let o_book = match exchange.orderbooks.get(&stock.clone().stock_id.to_string()) {
            Some(o) => o.clone(),
            None => panic!("Error getting orderbook"),
        };

        // test number of orders in orderbook
        assert_eq!(o_book.oid_map.len(), 1);
    }

    // test executing all orders in exchange
    #[test]
    fn test_exchange_execute_all_orders() {
        // create new exchange
        let mut exchange = Exchange::new();

        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        // create new user
        let issuer = User::new(
            Uuid::new_v4(),
            String::from("John"),
            String::from("john.doe@gmail.com"),
            String::from("password"),
            Some(1e6 as f32),
        );

        // add stock to exchange
        exchange.add_stock(stock.clone(), issuer);

        // create new orders
        let orders = gen_orders(
            stock.clone(),
            200,
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            Some(69.0),
            Some(0.0),
        );

        // get orderbook
        let mut o_book = match exchange.orderbooks.get(&stock.clone().stock_id.to_string()) {
            Some(o) => o.clone(),
            None => panic!("Error getting orderbook"),
        };

        // queue orders to order book
        for order in orders {
            o_book.queue_order(order);
        }

        // insert o_book to exchange
        exchange
            .orderbooks
            .insert(stock.clone().stock_id.to_string(), o_book);

        // check order queue length
        let order_book = match exchange.orderbooks.get(&stock.clone().stock_id.to_string()) {
            Some(o) => o.clone(),
            None => panic!("Error getting orderbook"),
        };

        assert_eq!(order_book.order_queue.len(), 200);

        // execute all orders
        match exchange.execute_all_orders() {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        };

        let book = match exchange.orderbooks.get(&stock.clone().stock_id.to_string()) {
            Some(o) => o.clone(),
            None => panic!("Error getting orderbook"),
        };

        book.print_orderbook();

        // check order queue length
        assert_eq!(book.order_queue.len(), 0);
    }

    // test adding a stock to MatchingEngine
    #[test]
    fn test_matching_engine_add_stock() {
        // create new matching engine
        let mut me = MatchingEngine::new("redis://127.0.0.1:6379");

        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        // create new user
        let issuer = User::new(
            Uuid::new_v4(),
            String::from("John"),
            String::from("john.doe@gmail.com"),
            String::from("password"),
            Some(1e6 as f32),
        );

        // add stock to matching engine
        me.add_stock(stock.clone(), issuer);

        // test number of stocks in matching engine
        let o_book: OrderBook = match me
            .exchange
            .orderbooks
            .get(&stock.clone().stock_id.to_string())
        {
            Some(o) => o.clone(),
            None => panic!("Error getting orderbook"),
        };
        assert_eq!(o_book.stock_id, stock.stock_id);
        assert_eq!(o_book.stock_info.ticker, stock.ticker);
        assert_eq!(o_book.stock_info.name, stock.name);

        // test to check users and userstocks
        assert_eq!(me.exchange.users.len(), 1);
        assert_eq!(me.exchange.user_stocks.len(), 1);
    }

    // test executing orders in MatchingEngine
    #[tokio::test]
    async fn test_matching_engine_execute_order() {
        // create new matching engine
        let mut me: MatchingEngine = MatchingEngine::new("redis://127.0.0.1:6379");
        // new redis client with same address
        let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();

        // create new stock
        let stock = Stock::new(
            Uuid::new_v4(),
            String::from("Apple"),
            String::from("AAPL"),
            Some(1e6 as i32),
            Some(1e6 as i32),
            Some(chrono::Utc::now().timestamp() as u32),
        );

        // establish pub sub connection to matching engine
        let mut publish_conn = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => panic!("Error getting connection: {}", e),
        };

        let mut pubsub_conn = client.get_async_connection().await.unwrap().into_pubsub();

        pubsub_conn
            .subscribe(format!("stock:{}", stock.clone().ticker))
            .await
            .unwrap();
        let mut pubsub_stream = pubsub_conn.on_message();

        // create new user
        let issuer = User::new(
            Uuid::new_v4(),
            String::from("John"),
            String::from("john.doe@gmail.com"),
            String::from("password"),
            Some(1e6 as f32),
        );

        // add stock to matching engine
        match me.add_stock(stock.clone(), issuer) {
            Ok(_) => (),
            Err(e) => panic!("Error adding stock: {}", e),
        };

        // create new order
        let order = Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            stock.clone(),
            orderbook::OrderSide::ASK,
            orderbook::OrderType::LIMIT,
            100,
            chrono::Utc::now().timestamp() as u32,
            Some(69.0),
        );

        // execute order (async), and check execution type (should be add)
        me.execute_order(order.clone()).await.unwrap();

        // get msg from channel
        let msg: String = pubsub_stream.next().await.unwrap().get_payload().unwrap();

        // print msg, and check that you received a msg
        println!("Message: {:?}", msg);
        assert!(msg.len() > 0);
        // decode json
        let exec: Execution = match serde_json::from_str(&msg) {
            Ok(r) => r,
            Err(e) => panic!("Error decoding json: {}", e),
        };

        // check order
        assert_eq!(exec.order, order.clone());
    }
}
