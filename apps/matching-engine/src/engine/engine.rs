use super::orderbook::Exchange;
use super::orderbook::Execution;
use super::orderbook::Order;
use super::orderbook::OrderBook;
use super::orderbook::OrderSide;
use super::orderbook::OrderType;
use super::orderbook::PriceLevel;
use super::orderbook::Stock;
use super::orderbook::User;
use crate::errors;
use std::fmt::Debug;

use std::collections::BTreeMap;

use redis::AsyncCommands;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
// enum for change type
pub enum ChangeType {
    Transaction,
    OrderModification,
    OrderCancellation,
    OrderAddition,
    OrderMatch,
    StockAddition,
    StockDeleteion,
}

pub struct MatchingEngine {
    pub exchange: Exchange,
    pub client: redis::Client,
    pub conn: redis::Connection,
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

impl MatchingEngine {
    pub fn new(addr: &str) -> Self {
        // let orderbook = orderbook::orderbook::OrderBook::new(1);
        let exchange = Exchange::new();
        let client = match redis::Client::open(addr) {
            Ok(client) => client,
            Err(e) => panic!("Error connecting to redis: {:?}", e),
        };
        let conn = match client.get_connection() {
            Ok(conn) => conn,
            Err(e) => panic!("Error connecting to redis: {:?}", e),
        };
        MatchingEngine {
            exchange,
            client,
            conn,
        }
    }

    // get a stock
    pub fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, errors::StockError> {
        // get stock from self.exchange
        let stock: Result<Stock, errors::StockError> = self.exchange.get_stock(stock_id);
        match stock {
            Ok(stock) => Ok(stock),
            Err(e) => Err(e),
        }
    }

    // TODO change price level return value to Result, we can't panic here
    pub fn get_price_level(
        &mut self,
        stock_id: uuid::Uuid,
        order_side: OrderSide,
        price: f32,
    ) -> &mut PriceLevel {
        // get orderbook given stock id
        let orderbook: &mut OrderBook =
            match self.exchange.orderbooks.get_mut(&stock_id.to_string()) {
                Some(orderbook) => orderbook,
                None => panic!("Orderbook not found"),
            };

        match orderbook.get_price_level(order_side, price) {
            Some(price_level) => price_level,
            None => panic!("Price level not found"),
        }
    }

    pub fn get_oid_map(&self, stock_id: uuid::Uuid) -> &BTreeMap<uuid::Uuid, Order> {
        // get orderbook given stock id
        let orderbook: &OrderBook = match self.exchange.orderbooks.get(&stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => panic!("Orderbook not found"),
        };

        orderbook.get_oid_map()
    }

    pub fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        price: f32,
        quantity: i32,
    ) -> Result<(), errors::OrderError> {
        let orderbook: &mut OrderBook =
            match self.exchange.orderbooks.get_mut(&order_id.to_string()) {
                Some(orderbook) => orderbook,
                None => {
                    return Err(errors::OrderError::Other(String::from(
                        "Orderbook not found",
                    )))
                }
            };

        match orderbook.modify_order(order_id, quantity, Some(price)) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), errors::OrderError> {
        let orderbook = match self.exchange.orderbooks.get_mut(&order_id.to_string()) {
            Some(orderbook) => orderbook,
            None => {
                return Err(errors::OrderError::Other(String::from(
                    "Orderbook not found",
                )))
            }
        };

        match orderbook.delete_order(order_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // add stock
    pub fn add_stock(&mut self, stock: Stock, issuer: User) -> Result<(), errors::StockError> {
        // add stock to self.exchange
        match self.exchange.add_stock(stock.clone(), issuer) {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    pub async fn execute_order(&mut self, order: Order) -> Result<Execution, errors::OrderError> {
        // get orderbook given stock id
        let orderbook: &mut OrderBook = match self
            .exchange
            .orderbooks
            .get_mut(&order.clone().stock.stock_id.to_string())
        {
            Some(orderbook) => orderbook,
            None => {
                return Err(errors::OrderError::Other(String::from(
                    "Orderbook not found",
                )))
            }
        };

        //queue and execute order
        orderbook.queue_order(order.clone());

        // execute order and publish execution to redis pub sub for a stock ticker channel
        let channel: String = format!("stock:{}", order.clone().stock.ticker);
        let mut pubsub_conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => panic!("Error connecting to redis: {:?}", e),
        };

        match orderbook.execute_order() {
            Ok(exec) => {
                // publish execution to redis pub sub for a stock ticker channel
                let data = json!(exec);
                let data = serde_json::to_string(&data).unwrap();
                // publish
                let _: () = pubsub_conn.publish(channel, data).await.unwrap();
                Ok(exec)
            },
            Err(e) => Err(e),
        }
    }

    // execute all orders with self.execute order and return a vec of excecutions
    pub async fn execute_all_orders(&mut self) -> Result<Vec<Execution>, errors::OrderError> {
        let mut executions: Vec<Execution> = Vec::new();
        for orderbook in self.exchange.orderbooks.values_mut() {
            match orderbook.execute_all_orders() {
                Ok(execs) => {
                    for exec in execs {
                        executions.push(exec);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // execute order and publish execution to redis pub sub for a stock ticker channel
        let channel: String = format!("stock:{}", executions.clone()[0].order.stock.ticker);
        let mut pubsub_conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => panic!("Error connecting to redis: {:?}", e),
        };

        for exec in executions.clone() {
            // publish execution to redis pub sub for a stock ticker channel
            let data = json!(exec);
            let data = serde_json::to_string(&data).unwrap();
            // publish
            let _: () = pubsub_conn.publish(channel.clone(), data).await.unwrap();
        }

        Ok(executions)
    }
}
