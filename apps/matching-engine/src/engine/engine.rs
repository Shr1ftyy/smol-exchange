use std::fmt::Debug;

use super::orderbook::{self, Stock};
use crate::errors;

pub trait Matching {
    fn new(addr: &str) -> Self;
    fn get_order(&mut self, order_id: uuid::Uuid) -> Result<orderbook::Order, errors::OrderError>;
    fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError>;
    fn match_order(&mut self, order: orderbook::Order) -> Result<(), errors::OrderError>;
    fn add_order(&mut self, order: orderbook::Order) -> Result<(), errors::OrderError>;
    fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        order: orderbook::Order
    ) -> Result<(), errors::OrderError>;
    fn cancel_order(&mut self, order_id: uuid::Uuid) -> Result<(), errors::OrderError>;
}

pub trait Management {
    fn new(addr: &str) -> Self;
    // add stock
    fn add_stock(&mut self, stock: Stock) -> Result<(), errors::StockError>;
    // modify stock
    fn modify_stock(
        &mut self,
        stock_id: uuid::Uuid,
        name: String,
        ticker: String,
    ) -> Result<(), errors::StockError>;
    // remove stock completely from redis
    fn remove_stock(&mut self, stock_id: uuid::Uuid) -> Result<(), errors::StockError>;
}

pub struct MatchingEngine {
    client: redis::Client,
    conn: redis::Connection,
}

impl Debug for MatchingEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchingEngine")
            .field("client", &self.client)
            // TODO: wtf this is yucky no?
            .field("conn", &self.client)
            .finish()
    }
}

impl Matching for MatchingEngine {
    fn new(addr: &str) -> Self {
        // let orderbook = orderbook::orderbook::OrderBook::new(1);
        let client = redis::Client::open(addr).unwrap();
        let conn = client.get_connection().unwrap();
        MatchingEngine {
            // orderbook,
            client,
            conn,
        }
    }

    // get order from redis
    fn get_order(&mut self, order_id: uuid::Uuid) -> Result<orderbook::Order, errors::OrderError> {
        let order_id: String = order_id.to_string();
        let order_id_str = order_id.as_str();

        // get order from redis self.conn
        let order = redis::cmd("HGET")
            .arg("orders")
            .arg(order_id_str)
            .query::<String>(&mut self.conn);

        match order {
            Ok(order) => {
                let order: orderbook::Order = serde_json::from_str(&order).unwrap();
                Ok(order)
            }
            Err(e) => Err(errors::OrderError::Other(e.to_string())),
        }
    }

    // match order
    fn match_order(&mut self, order: orderbook::Order) -> Result<(), errors::OrderError> {
        todo!();
    }
    // fn match_order(&self, order: orderbook::Order) -> Result<(), errors::OrderError>{
    //     let mut conn = self.client.get_connection().unwrap();

    //     let orders_to_search = match order.order_side {
    //         orderbook::OrderSide::BID => {
    //             // get bids
    //             redis::cmd("ZRANGEBYSCORE")
    //                 .arg(format!("orderbook:{}:BIDS", order.stock.ticker))
    //                 .arg("-inf")
    //                 .arg(order.price)
    //                 .query::<Vec<String>>(&mut self.conn)
    //         }
    //         orderbook::OrderSide::ASK => {
    //             // get asks
    //             redis::cmd("ZRANGEBYSCORE")
    //                 .arg(format!("orderbook:{}:ASKS", order.stock.ticker))
    //                 .arg("-inf")
    //                 .arg(order.price)
    //                 .query::<Vec<String>>(&mut self.conn)
    //         }
    //     };

    //     // match orders
    //     match orders_to_search {
    //         Ok(orders_to_search) => {
    //             // iterate through orders
    //             for order_id in orders_to_search {
    //                 // get order from redis
    //                 let order_to_match = self.get_order(order_id.parse().unwrap());

    //                 // match order
    //                 match order_to_match {
    //                     Ok(order_to_match) => {
    //                         // if order is a buy
    //                         if order.order_side == orderbook::OrderSide::BID {
    //                             // if order is a market order
    //                             if order.order_type == orderbook::OrderType::MARKET {
    //                                 // if order qty is greater than or equal to the current order qty
    //                                 if order.qty >= order.qty {
    //                                     // execute order
    //                                     self.execute_order(order);
    //                                 }
    //                             }
    //                             // if order is a limit order
    //                             else if order.order_type == orderbook::OrderType::LIMIT {
    //                                 // if order price is greater than or equal to the current order price
    //                                 if order.price >= order.price {
    //                                     // execute order
    //                                     self.execute_order(order);
    //                                 }
    //                             }
    //                         }
    //                         // if order is a sell
    //                         else if order.order_side == orderbook::OrderSide::ASK {
    //                             // if order is a market order
    //                             if order.order_type == orderbook::OrderType::MARKET {
    //                                 // if order qty is greater than or equal to the current order qty
    //                                 if order.qty >= order.qty {
    //                                     // execute order
    //                                     self.execute_order(order);
    //                                 }
    //                             }
    //                             // if order is a limit order
    //                             else if order.order_type == orderbook::OrderType::LIMIT {
    //                                 // if order price is greater than or equal to the current order price
    //                                 if order.price >= order.price {
    //                                     // execute order
    //                                     self.execute_order(order);
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     Err(e) => return Err(errors::OrderError::Other(e.to_string())),
    //                 }
    //             }
    //             Ok(())
    //         }
    //         Err(e) => Err(errors::OrderError::Other(e.to_string())),
    //     }

    // }

    fn add_order(&mut self, order: orderbook::Order) -> Result<(), errors::OrderError> {
        let mut conn: redis::Connection = self.client.get_connection().unwrap();
        let order_id: String = order.order_id.to_string();
        let ticker_string: String = order.stock.ticker.to_string();
        let ticker: &str = ticker_string.as_str();
        let order_id_str: &str = order_id.as_str();
        let order_string: String = serde_json::to_string(&order).unwrap();
        let order_str: &str = order_string.as_str();
        let stock_id_string: String = order.stock.stock_id.to_string();
        let stock_id_str: &str = stock_id_string.as_str();

        // check existance of stock, get market price
        let stock = redis::cmd("HGET")
            .arg("stocks")
            .arg(stock_id_str)
            .query::<String>(&mut self.conn);

        // perform match statment on stock
        match stock {
            // if it exists
            Ok(stock) => {
                let stock: Stock = serde_json::from_str(&stock).unwrap();
                // perform operations based on whether the order is a buy/sell or market/limit
                match order.order_side {
                    orderbook::OrderSide::BID => {
                        match order.order_type {
                            orderbook::OrderType::MARKET => {
                                // add order to redis
                                let res = redis::pipe()
                                    .cmd("HSET")
                                    .arg("orders")
                                    .arg(order_id_str)
                                    .arg(order_str)
                                    .cmd("SADD")
                                    .arg("order_ids")
                                    .arg(order_id_str)
                                    .cmd("ZADD")
                                    .arg(format!("orderbook:{}:BIDS", ticker))
                                    .arg(order.price)
                                    .arg(order_id_str)
                                    .query::<()>(&mut self.conn);

                                match res {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(errors::OrderError::Other(e.to_string())),
                                }
                            }
                            orderbook::OrderType::LIMIT => {
                                // add order to redis
                                let res = redis::pipe()
                                    .cmd("HSET")
                                    .arg("orders")
                                    .arg(order_id_str)
                                    .arg(order_str)
                                    .cmd("SADD")
                                    .arg("order_ids")
                                    .arg(order_id_str)
                                    .cmd("ZADD")
                                    .arg(format!("orderbook:{}:BIDS", ticker))
                                    .arg(order.price)
                                    .arg(order_id_str)
                                    .query::<()>(&mut self.conn);

                                match res {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(errors::OrderError::Other(e.to_string())),
                                }
                            }
                        }
                    }
                    orderbook::OrderSide::ASK => {
                        match order.order_type {
                            orderbook::OrderType::MARKET => {
                                // add order to redis
                                let res = redis::pipe()
                                    .cmd("HSET")
                                    .arg("orders")
                                    .arg(order_id_str)
                                    .arg(order_str)
                                    .cmd("SADD")
                                    .arg("order_ids")
                                    .arg(order_id_str)
                                    .cmd("ZADD")
                                    .arg(format!("orderbook:{}:ASKS", ticker))
                                    .arg(order.price)
                                    .arg(order_id_str)
                                    .query::<()>(&mut self.conn);

                                match res {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(errors::OrderError::Other(e.to_string())),
                                }
                            }
                            orderbook::OrderType::LIMIT => {
                                // add order to redis
                                let res = redis::pipe()
                                    .cmd("HSET")
                                    .arg("orders")
                                    .arg(order_id_str)
                                    .arg(order_str)
                                    .cmd("ZADD")
                                    .arg(format!("orderbook:{}:ASKS", ticker))
                                    .arg(order.price)
                                    .arg(order_id_str)
                                    .query::<()>(&mut self.conn);

                                match res {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(errors::OrderError::Other(e.to_string())),
                                }
                            }
                        }
                    }
                }
            }
            // if the stock doesn't exist, throw an error
            Err(e) => Err(errors::OrderError::Other(e.to_string())),
        }
    }

    // modify order
    fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        order: orderbook::Order,
    ) -> Result<(), errors::OrderError> {
        let order_id: String = order_id.to_string();
        let order_id_str = order_id.as_str();

        let order: Result<String, redis::RedisError> = redis::cmd("HGET")
            .arg("orders")
            .arg(order_id_str)
            .query::<String>(&mut self.conn);

        match order {
            Ok(order) => {
                let order: orderbook::Order = serde_json::from_str(&order).unwrap();
                let ticker: String = order.stock.ticker.to_string();
                let order_id: String = order.order_id.to_string();
                let order_id_str = order_id.as_str();
                redis::pipe()
                    .cmd("HDEL")
                    .arg("orders")
                    .arg(order_id_str)
                    .cmd("SREM")
                    .arg("order_ids")
                    .arg(order_id_str)
                    .cmd("ZREM")
                    .arg(format!("orderbook:{}:BIDS", ticker))
                    .arg(order.price)
                    .arg(order_id_str)
                    .cmd("ZREM")
                    .arg(format!("orderbook:{}:ASKS", ticker))
                    .arg(order.price)
                    .arg(order_id_str)
                    .query::<()>(&mut self.conn);
                self.add_order(order)
            }
            Err(e) => Err(errors::OrderError::Other(e.to_string())),
        }
    }

    // cancel order
    fn cancel_order(&mut self, order_id: uuid::Uuid) -> Result<(), errors::OrderError> {
        let mut conn: redis::Connection = self.client.get_connection().unwrap();
        let order_id: String = order_id.to_string();
        let order_id_str = order_id.as_str();

        let order: Result<String, redis::RedisError> = redis::cmd("HGET")
            .arg("orders")
            .arg(order_id_str)
            .query::<String>(&mut self.conn);

        match order {
            Ok(order) => {
                let order: orderbook::Order = serde_json::from_str(&order).unwrap();
                let ticker: String = order.stock.ticker.to_string();
                let order_id: String = order.order_id.to_string();
                let order_id_str = order_id.as_str();
                redis::pipe()
                    .cmd("HDEL")
                    .arg("orders")
                    .arg(order_id_str)
                    .cmd("SREM")
                    .arg("order_ids")
                    .arg(order_id_str)
                    .cmd("ZREM")
                    .arg(format!("orderbook:{}", ticker))
                    .arg(order_id_str)
                    .query::<()>(&mut self.conn)
                    .unwrap();
                Ok(())
            }
            Err(e) => Err(errors::OrderError::Other(e.to_string())),
        }
    }

    fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError> {
        let mut conn: redis::Connection = self.client.get_connection().unwrap();
        let stock_id: String = stock_id.to_string();
        let stock_id_str = stock_id.as_str();

        let stock = redis::cmd("HGET")
            .arg("stocks")
            .arg(stock_id_str)
            .query::<String>(&mut self.conn);

        match stock {
            Ok(stock) => {
                let stock: Stock = serde_json::from_str(&stock).unwrap();
                Ok(stock)
            }
            Err(e) => Err(errors::StockError::Other(e.to_string())),
        }
    }
}

impl Management for MatchingEngine {
    // return new MatchingEngine
    fn new(addr: &str) -> Self {
        let client: redis::Client = redis::Client::open(addr).unwrap();
        let conn: redis::Connection = client.get_connection().unwrap();
        MatchingEngine { client, conn }
    }
    // add stock
    fn add_stock(&mut self, stock: Stock) -> Result<(), errors::StockError> {
        let mut conn: redis::Connection = self.client.get_connection().unwrap();
        let stock_id: String = stock.stock_id.to_string();
        let stock_id_str: &str = stock_id.as_str();
        let stock_string: String = serde_json::to_string(&stock).unwrap();
        let stock_str: &str = stock_string.as_str();

        let res = redis::pipe()
            .cmd("HSET")
            .arg("stocks")
            .arg(stock_id_str)
            .arg(stock_str)
            .query::<()>(&mut self.conn);

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(errors::StockError::Other(e.to_string())),
        }
    }

    // modify stock
    fn modify_stock(
        &mut self,
        stock_id: uuid::Uuid,
        name: String,
        ticker: String,
    ) -> Result<(), errors::StockError> {
        let mut conn = self.client.get_connection().unwrap();
        let stock_id: String = stock_id.to_string();
        let stock_id_str = stock_id.as_str();

        let stock: Result<Stock, errors::StockError> = self.get_stock(stock_id.parse().unwrap());

        match stock {
            Ok(stock) => {
                let mut stock: Stock = stock;
                let stock_id: String = stock.stock_id.to_string();
                let stock_id_str = stock_id.as_str();
                stock.name = name;
                stock.ticker = ticker;
                let stock_string: String = serde_json::to_string(&stock).unwrap();
                let stock_str = stock_string.as_str();
                redis::pipe()
                    .cmd("HSET")
                    .arg("stocks")
                    .arg(stock_id_str)
                    .arg(stock_str)
                    .cmd("SADD")
                    .arg("stock_ids")
                    .arg(stock_id_str)
                    .query::<()>(&mut self.conn)
                    .unwrap();
                Ok(())
            }
            Err(e) => Err(errors::StockError::Other(e.to_string())),
        }
    }

    // remove stock completely from redis
    fn remove_stock(&mut self, stock_id: uuid::Uuid) -> Result<(), errors::StockError> {
        let mut conn = self.client.get_connection().unwrap();
        let stock_id: String = stock_id.to_string();
        let stock_id_str = stock_id.as_str();

        let stock = redis::cmd("HGET")
            .arg("stocks")
            .arg(stock_id_str)
            .query::<String>(&mut self.conn);

        match stock {
            Ok(stock) => {
                let stock: Stock = serde_json::from_str(&stock).unwrap();
                let stock_id: String = stock.stock_id.to_string();
                let stock_id_str = stock_id.as_str();
                redis::pipe()
                    .cmd("HDEL")
                    .arg("stocks")
                    .arg(stock_id_str)
                    .cmd("SREM")
                    .arg("stock_ids")
                    .arg(stock_id_str)
                    .query::<()>(&mut self.conn)
                    .unwrap();
                Ok(())
            }
            Err(e) => Err(errors::StockError::Other(e.to_string())),
        }
    }
}
