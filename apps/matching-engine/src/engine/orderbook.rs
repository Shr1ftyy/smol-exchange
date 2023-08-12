use core::fmt;
// TODO: Is this ok?
use crate::errors::OrderError;
use crate::errors::StockError;
use serde::{Deserialize, Serialize};

extern crate redis;
use redis::Commands;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    BID,
    ASK,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderType {
    MARKET,
    LIMIT,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
    pub stock_id: uuid::Uuid,
    pub name: String,
    pub ticker: String,
    pub total_issued: Option<i32>,
    pub outstanding_shares: Option<i32>,
    pub time_created: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: uuid::Uuid,
    pub creator_id: i32,
    pub stock: Stock,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub qty: i32,
    pub time_created: u32,
    pub price: Option<f32>,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderType::MARKET => write!(f, "MARKET"),
            OrderType::LIMIT => write!(f, "LIMIT"),
        }
    }
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderSide::BID => write!(f, "BID"),
            OrderSide::ASK => write!(f, "ASK"),
        }
    }
}

// implementation for Stock (with validation)
impl Stock {
    pub fn validate(&self) -> Result<(), StockError> {
        if self.stock_id.is_nil() {
            return Err(StockError::InvalidStockID);
        }
        if self.name.is_empty() {
            return Err(StockError::InvalidName);
        }
        if self.ticker.is_empty() {
            return Err(StockError::InvalidTicker);
        }
        match self.total_issued {
            Some(total_issued) => {
                if total_issued <= 0 {
                    return Err(StockError::InvalidTotalIssued);
                }
            }
            None => {
                return Err(StockError::InvalidTotalIssued);
            }
        }
        match self.outstanding_shares {
            Some(outstanding_shares) => {
                if outstanding_shares <= 0 {
                    return Err(StockError::InvalidOutstandingShares);
                }
            }
            None => {
                return Err(StockError::InvalidOutstandingShares);
            }
        }
        match self.time_created {
            Some(time_created) => {
                if time_created <= 0 {
                    return Err(StockError::InvalidTimeCreated);
                }
            }
            None => {
                return Err(StockError::InvalidTimeCreated);
            }
        }

        Ok(()) // If all checks pass, return Ok(())
    }

    // return new Stock
    pub fn new(
        stock_id: uuid::Uuid,
        name: String,
        ticker: String,
        total_issued: Option<i32>,
        outstanding_shares: Option<i32>,
        time_created: Option<u32>,
    ) -> Self {
        let stock: Stock = Stock {
            stock_id,
            name,
            ticker,
            total_issued: total_issued,
            outstanding_shares: outstanding_shares,
            time_created: time_created,
        };

        match stock.validate() {
            Ok(_) => println!("Stock is valid!"),
            Err(e) => println!("Error validating stock: {:?}", e),
        }

        stock
    }
}

impl Order {
    pub fn validate(&self) -> Result<(), OrderError> {
        if self.order_id.is_nil() {
            return Err(OrderError::InvalidOrderID);
        }
        if self.creator_id <= 0 {
            return Err(OrderError::InvalidCreatorID);
        }
        if self.stock.stock_id == uuid::Uuid::nil() {
            return Err(OrderError::InvalidStockID);
        }
        if self.price <= 0.0 {
            return Err(OrderError::InvalidPrice);
        }
        if self.qty == 0 {
            return Err(OrderError::InvalidQuantity);
        }
        if self.time_created == 0 {
            return Err(OrderError::InvalidTimeCreated);
        }

        Ok(()) // If all checks pass, return Ok(())
    }

    pub fn new(
        order_id: uuid::Uuid,
        creator_id: i32,
        stock: Stock,
        order_side: OrderSide,
        order_type: OrderType,
        price: f32,
        qty: i32,
        time_created: u32,
    ) -> Self {
        let order: Order = Order {
            order_id,
            creator_id,
            stock,
            order_side,
            order_type,
            price,
            qty,
            time_created,
        };

        match order.validate() {
            Ok(_) => println!("Order is valid!"),
            Err(e) => println!("Error validating order: {:?}", e),
        }

        // TODO: Is this how it is done?
        order
    }
}
