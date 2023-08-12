use std::fmt::Debug;

use super::orderbook::{self, Stock};
use crate::errors;

pub trait Matching {
    fn new(addr: &str) -> Self;
    fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError>;
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
