use super::orderbook;
use crate::errors;

#[derive(Debug)]
pub struct MatchingEngine {
    // orderbook: orderbook::orderbook::OrderBook,
    client: redis::Client,
}

impl MatchingEngine {
    pub fn new(addr: &str) -> Self {
        // let orderbook = orderbook::orderbook::OrderBook::new(1);
        let client = redis::Client::open(addr).unwrap();
        MatchingEngine {
            // orderbook,
            client,
        }
    }

    // TODO: match order
    pub fn match_order(&self, order: orderbook::Order) -> Result<(), errors::OrderError> {
        let mut conn = self.client.get_connection().unwrap();
        let order_id: String = order.order_id.to_string();
        let order_id_str = order_id.as_str();
        let order_string: String = serde_json::to_string(&order).unwrap();
        let order_str = order_string.as_str();
        // iterate through orderbook and match orders
        let orderbook: Vec<String> = redis::cmd("ZRANGE")
            .arg("orderbook")
            .arg(0)
            .arg(-1)
            .query(&mut conn)
            .unwrap();

        for order_id in orderbook {
            let order: String = redis::cmd("HGET")
                .arg("orders")
                .arg(order_id)
                .query(&mut conn)
                .unwrap();
            let order: orderbook::Order = serde_json::from_str(&order).unwrap();
            // if order is buy and price is greater than or equal to orderbook price
            if order.order_type == orderbook::OrderType::BUY && order.price >= order.price {
                // if order qty is greater than or equal to orderbook qty
                if order.qty >= order.qty {
                    // match order
        }            
    }


    pub fn add_order(&self, order: orderbook::Order) -> Result<(), errors::OrderError> {
        let mut conn = self.client.get_connection().unwrap();
        let order_id: String = order.order_id.to_string();
        let order_id_str = order_id.as_str();
        let order_string: String = serde_json::to_string(&order).unwrap();
        let order_str = order_string.as_str();
        redis::pipe()
            .cmd("HSET")
            .arg("orders")
            .arg(order_id_str)
            .arg(order_str)
            .cmd("SADD")
            .arg("order_ids")
            .arg(order_id_str)
            .cmd("ZADD")
            .arg("orderbook")
            .arg(order.price)
            .arg(order_id_str)
            .query::<()>(&mut conn)
            .unwrap();
        Ok(())
    }

    // modify order
    pub fn modify_order(&self, order: orderbook::Order) -> Result<(), errors::OrderError> {
        // if order qty is 0, cancel order
        if order.qty == 0 {
            return self.cancel_order(order.order_id);
        }

        let mut conn = self.client.get_connection().unwrap();
        let order_id: String = order.order_id.to_string();
        let order_id_str = order_id.as_str();
        let order_string: String = serde_json::to_string(&order).unwrap();
        let order_str = order_string.as_str();
        redis::pipe()
            .cmd("HSET")
            .arg("orders")
            .arg(order_id_str)
            .arg(order_str)
            .query::<()>(&mut conn)
            .unwrap();
        Ok(())
    }

    // cancel order
    pub fn cancel_order(&self, order_id: uuid::Uuid) -> Result<(), errors::OrderError> {
        let mut conn = self.client.get_connection().unwrap();
        let order_id: String = order_id.to_string();
        let order_id_str = order_id.as_str();
        redis::pipe()
            .cmd("HDEL")
            .arg("orders")
            .arg(order_id_str)
            .cmd("SREM")
            .arg("order_ids")
            .arg(order_id_str)
            .cmd("ZREM")
            .arg("orderbook")
            .arg(order_id_str)
            .query::<()>(&mut conn)
            .unwrap();
        Ok(())
    }
}
