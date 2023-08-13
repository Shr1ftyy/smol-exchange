mod engine;
mod errors;
mod helpers;
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


    // function to generate a bunch of orders:
    fn generate_orders() -> Vec<Order> {
        let mut orders: Vec<Order> = Vec::new();
        // Bid Orders
        for i in 0..100 {
            let price: f32 = rand::thread_rng().gen_range(97.00..99.00);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i),
                    format!("STK{}", i),
                    Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                    Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                    Some(chrono::Utc::now().timestamp() as u32),
                ),
                engine::orderbook::OrderSide::BID,
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

        // Ask Orders
        for i in 0..100 {
            let price: f32 = rand::thread_rng().gen_range(98.00..100.00);
            // Round to 2 decimal places
            let rounded_price = (price * 100.0).round() / 100.0;
            let order = Order::new(
                Uuid::new_v4(),
                1,
                Stock::new(
                    uuid::Uuid::new_v4(),
                    format!("Stock {}", i),
                    format!("STK{}", i),
                    Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                    Some(rand::thread_rng().gen_range(1e6..1e7) as i32),
                    Some(chrono::Utc::now().timestamp() as u32),
                ),
                engine::orderbook::OrderSide::ASK,
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
        orders
    }



    #[test]
    fn test_adding_orders_orderbook() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());
        let orders: Vec<Order> = generate_orders();
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

        // print orderbook
        o_book.print_orderbook();

    }

    #[test]
    // matching orders test
    fn test_matching_orders() {
        let mut o_book = engine::orderbook::OrderBook::new(uuid::Uuid::new_v4());
        let orders = generate_orders();
     
        let mut highest_bid_price = f32::NEG_INFINITY;

        for order in &orders {
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

        // match first 25 orders
        let p_level = match o_book.get_price_level(engine::orderbook::OrderSide::BID, highest_bid_price) {
            Some(p) => p.clone(),
            None => panic!("Price level not found!"),
        };

        for order in p_level.orders.iter() {
            let res = o_book.match_order(order.clone().order_id);
            match res {
                Ok(_) => {
                    // assert if it's in the orderbook
                    print!("checking if order is in orderbook: {} ... ", order.order_id);
                    // if order not in oid map, print that it got filled successfully
                    if !o_book.get_oid_map().contains_key(&order.order_id) {
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

// junk
        // let mut filled_orders: Vec<Order> = Vec::new();
        
        // let p_level = match o_book.get_price_level(engine::orderbook::OrderSide::ASK, lowest_ask_price) {
        //     Some(p) => p.clone(),
        //     None => panic!("Price level not found!"),
        // };

        // // match orders from p_level 
        // for o in p_level.orders.iter() {
        //     let res = o_book.match_order(o.clone().order_id);
        //     match res {
        //         Ok(_) => {
        //             // assert if it's in the orderbook
        //             print!("checking if order is in orderbook: {} ... ", o.order_id);
        //             // if order not in oid map, print that it got filled successfully
        //             if !(o_book.get_oid_map().contains_key(&o.order_id)) {
        //                 println!("Order filled successfully!");
        //                 filled_orders.push(o.clone());
        //             } else {
        //                 println!("Order not filled!");
        //             }
        //         }
        //         Err(e) => println!("Error matching order: {:?}", e),
        //     }
        // }

        // for o in &orders {

        //     let res = o_book.match_order(o.clone().order_id);
        //     match res {
        //         Ok(_) => {
        //             // assert if it's in the orderbook
        //             print!("checking if order is in orderbook: {} ... ", o.order_id);
        //             // if order not in oid map, print that it got filled successfully
        //             if !o_book.get_oid_map().contains_key(&o.order_id) {
        //                 println!("Order filled successfully!");
        //                 filled_orders.push(o.clone());
        //             } else {
        //                 println!("Order not filled!");
        //             }
        //         }
        //         Err(e) => println!("Error matching order: {:?}", e),
        //     }
        // }