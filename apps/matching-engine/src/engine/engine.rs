use std::fmt::Debug;

use super::orderbook;
use super::orderbook::Exchange;
use super::orderbook::Order;
use super::orderbook::OrderBook;
use super::orderbook::OrderSide;
use super::orderbook::OrderType;
use super::orderbook::PriceLevel;
use super::orderbook::Stock;
use crate::errors;

use std::collections::BTreeMap;

// struct for user
pub struct User {
    user_id: uuid::Uuid,
    name: String,
    email: String,
    balance: Option<f32>,
}

// struct for transaction
pub struct Transaction {
    transaction_id: uuid::Uuid,
    buyer_id: uuid::Uuid,
    seller_id: uuid::Uuid,
    stock_id: uuid::Uuid,
    order_type: OrderType,
    price: f32,
    quantity: i32,
    time_executed: u32,
}

// struct for user-stocks
pub struct UserStocks {
    user_id: uuid::Uuid,
    stock_id: uuid::Uuid,
    quantity: i32,
}

// enum for change type
pub enum ChangeType {
    Transaction,
    OrderModification,
    OrderCancellation,
    OrderAddition,
    OrderMatch,
}

// struct which logs changes in the orderbook and their type (i.e transaction, order modification, etc.)
pub struct OrderbookLog {
    change_type: ChangeType,
    timestamp: u32,
    data: String,
}

pub trait Matching {
    fn new(addr: &str) -> Self;
    fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError>;
    // get price level
    fn get_price_level(&mut self, stock_id: uuid::Uuid, order_side: OrderSide, price: f32) -> &mut PriceLevel;
    // get oid map
    fn get_oid_map(&self, stock_id: uuid::Uuid) -> &BTreeMap<uuid::Uuid, Order>;
    // execute order
    fn execute_order(&mut self, order: Order) -> Result<(), errors::OrderError>;
    // modify order
    fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        price: f32,
        quantity: i32,
    ) -> Result<(), errors::OrderError>;
    // delete order
    fn delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), errors::OrderError>;
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
    exchange: Exchange,
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
        let exchange = Exchange::new();
        let client = redis::Client::open(addr).unwrap();
        let conn = client.get_connection().unwrap();
        MatchingEngine {
            exchange,
            client,
            conn,
        }
    }

    // get a stock
    fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError> {
        // get stock from self.exchange
        let stock: Result<Stock, errors::StockError> = self.exchange.get_stock(stock_id);
        match stock {
            Ok(stock) => Ok(stock),
            Err(e) => Err(e),
        }
    }

    // TODO change price level return value to Result, we can't panic here
    fn get_price_level(&mut self, stock_id: uuid::Uuid, order_side: OrderSide, price: f32) -> &mut PriceLevel {
        // get orderbook given stock id
        let orderbook: &mut OrderBook = match self.exchange.orderbooks.get_mut(&stock_id.to_string()) { 
            Some(orderbook) => orderbook,
            None => panic!("Orderbook not found")
        };

        match orderbook.get_price_level(order_side, price) {
            Some(price_level) => price_level,
            None => panic!("Price level not found")
        }
    }

    fn get_oid_map(&self, stock_id: uuid::Uuid) -> &BTreeMap<uuid::Uuid, Order> {
        // get orderbook given stock id
        let orderbook: &OrderBook = match self.exchange.orderbooks.get(&stock_id.to_string()) { 
            Some(orderbook) => orderbook,
            None => panic!("Orderbook not found")
        };

        orderbook.get_oid_map()
    }

    fn execute_order(&mut self, order: Order) -> Result<(), errors::OrderError> {
        // get orderbook given stock id
        let orderbook: &mut OrderBook = match self.exchange.orderbooks.get_mut(&order.stock.stock_id.to_string()) { 
            Some(orderbook) => orderbook,
            None => return Err(errors::OrderError::Other(String::from("Orderbook not found")))
        };

        //queue and execute order
        orderbook.queue_order(order);

        match orderbook.execute_all_orders() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        price: f32,
        quantity: i32,
    ) -> Result<(), errors::OrderError> {
        let orderbook: &mut OrderBook = match self.exchange.orderbooks.get_mut(&order_id.to_string()) { 
            Some(orderbook) => orderbook,
            None => return Err(errors::OrderError::Other(String::from("Orderbook not found")))
        };

        match orderbook.modify_order(order_id, quantity, Some(price)) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), errors::OrderError> {
        let orderbook = match self.exchange.orderbooks.get_mut(&order_id.to_string()) { 
            Some(orderbook) => orderbook,
            None => return Err(errors::OrderError::Other(String::from("Orderbook not found")))
        };

        match orderbook.delete_order(order_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Management for MatchingEngine {
    // return new MatchingEngine
    fn new(addr: &str) -> Self {
        let exchange: Exchange = Exchange::new();
        let client: redis::Client = redis::Client::open(addr).unwrap();
        let conn: redis::Connection = client.get_connection().unwrap();
        MatchingEngine {
            exchange,
            client,
            conn,
        }
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
