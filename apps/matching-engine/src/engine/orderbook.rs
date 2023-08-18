use core::fmt;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::primitive;
// TODO: Is this ok?
use crate::errors::OrderError;
use crate::errors::StockError;
use crate::helpers::helpers;
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};

extern crate redis;
use redis::Commands;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    BID,
    ASK,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f32,
    pub qty: i32,
    pub orders: VecDeque<uuid::Uuid>,
}

#[derive(Debug)]
pub struct OrderBook {
    pub stock_id: uuid::Uuid,
    pub bid_price_levels: BTreeMap<String, PriceLevel>,
    pub ask_price_levels: BTreeMap<String, PriceLevel>,
    pub oid_map: BTreeMap<uuid::Uuid, Order>,
    pub last_market_price: Option<f32>,
}

pub struct Exchange {
    pub orderbooks: BTreeMap<String, OrderBook>,
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

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let order_type = match self.order_type {
            OrderType::MARKET => "MARKET",
            OrderType::LIMIT => "LIMIT",
        };
        let order_side = match self.order_side {
            OrderSide::BID => "BID",
            OrderSide::ASK => "ASK",
        };
        write!(
            f,
            "Order: {} {} {} {} {} {} {} {}",
            self.order_id,
            self.creator_id,
            self.stock.stock_id,
            self.stock.name,
            self.stock.ticker,
            order_type,
            order_side,
            self.qty
        )
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
        if self.price <= Some(0.0) {
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
        qty: i32,
        time_created: u32,
        price: Option<f32>,
    ) -> Self {
        let order: Order = Order {
            order_id,
            creator_id,
            stock,
            order_side,
            order_type,
            qty,
            time_created,
            price,
        };

        match order.validate() {
            Ok(_) => println!("Order is valid!"),
            Err(e) => println!("Error validating order: {:?}", e),
        }

        // TODO: Is this how it is done?
        order
    }
}

impl PriceLevel {
    pub fn new(price: f32, qty: i32) -> Self {
        let price_level: PriceLevel = PriceLevel {
            price,
            qty,
            orders: VecDeque::new(),
        };

        price_level
    }

    // add order id to price level
    pub fn add_order(&mut self, order: Order) {
        self.orders.push_back(order.order_id);
        self.qty += order.qty;
    }

    // remove order id from price level
    pub fn remove_order(&mut self, order: Order) {
        // get order from oid map
        self.orders.retain(|&x| x != order.order_id);
        self.qty -= order.qty;
    }
}

impl OrderBook {
    pub fn new(stock_id: uuid::Uuid) -> Self {
        let orderbook: OrderBook = OrderBook {
            stock_id,
            bid_price_levels: BTreeMap::new(),
            ask_price_levels: BTreeMap::new(),
            oid_map: BTreeMap::new(),
        };

        orderbook
    }

    // helpers
    pub fn get_price_level(
        &mut self,
        order_side: OrderSide,
        price: f32,
    ) -> Option<&mut PriceLevel> {
        match order_side {
            OrderSide::BID => self
                .bid_price_levels
                .get_mut(&helpers::f32_to_string(price, 2)),
            OrderSide::ASK => self
                .ask_price_levels
                .get_mut(&helpers::f32_to_string(price, 2)),
        }
    }

    pub fn get_oid_map(&self) -> &BTreeMap<uuid::Uuid, Order> {
        &self.oid_map
    }

    pub fn get_stock_id(&self) -> uuid::Uuid {
        self.stock_id
    }

    pub fn add_order(&mut self, order: Order) -> Result<(), OrderError> {
        let price_key;
        match order.price {
            Some(price) => {
                price_key = helpers::f32_to_string(price, 2);
                self.oid_map.insert(order.order_id, order.clone());
            }
            None => {
                // get best price (depending on if it is a bid or ask) from price levels
                // and set price_key to that pricea
                match order.order_side {
                    OrderSide::BID => {
                        let best_price_level = self.ask_price_levels.iter().next();
                        match best_price_level {
                            Some((price, _)) => {
                                price_key = price.clone();
                                self.oid_map.insert(order.order_id, order.clone());
                            }
                            None => {
                                return Err(OrderError::InvalidPrice);
                            }
                        }
                    }
                    OrderSide::ASK => {
                        let best_price_level = self.bid_price_levels.iter().rev().next();
                        match best_price_level {
                            Some((price, _)) => {
                                price_key = price.clone();
                                self.oid_map.insert(order.order_id, order.clone());
                            }
                            None => {
                                return Err(OrderError::InvalidPrice);
                            }
                        }
                    }
                }
            }
        }

        let price_level: &mut PriceLevel = match order.order_side {
            OrderSide::BID => {
                self.bid_price_levels
                    .entry(price_key.clone())
                    .or_insert(PriceLevel::new(
                        price_key.parse().unwrap(),
                        order.clone().qty,
                    ))
            }
            OrderSide::ASK => {
                self.ask_price_levels
                    .entry(price_key.clone())
                    .or_insert(PriceLevel::new(
                        price_key.parse().unwrap(),
                        order.clone().qty,
                    ))
            }
        };

        price_level.add_order(order.clone());
        Ok(())
    }

    // delete an order (affects pricelevel and orderbook)
    pub fn delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let order: Order = match self.oid_map.get(&order_id) {
            Some(order) => order.clone(),
            None => return Err(OrderError::InvalidOrderID),
        };

        // remove order from price level
        let price_level = match self.get_price_level(order.order_side, order.price.unwrap()) {
            Some(price_level) => price_level,
            None => return Err(OrderError::InvalidPrice),
        };

        price_level.remove_order(order);

        // remove order from oid map
        self.oid_map.remove(&order_id);

        Ok(())
    }

    // modifying an order (affects pricelevel and orderbook)
    pub fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        new_qty: i32,
        new_price: Option<f32>,
    ) -> Result<(), OrderError> {
        // remove order if the new qty is 0 or less
        if new_qty <= 0 {
            return self.delete_order(order_id);
        }

        let mut order = match self.oid_map.get(&order_id) {
            Some(order) => order.clone(),
            None => return Err(OrderError::InvalidOrderID),
        };

        // remove order from price level
        let price_level = match self.get_price_level(order.order_side, order.price.unwrap()) {
            Some(price_level) => price_level,
            None => return Err(OrderError::InvalidPrice),
        };

        price_level.remove_order(order.clone());

        order.qty = new_qty;
        order.price = new_price;

        price_level.add_order(order.clone());

        // update order in oid map
        self.oid_map.insert(order.order_id, order);

        Ok(())
    }

    // match order against orderbook given order id in orderbook
    pub fn match_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let mut order: Order = match self.oid_map.get(&order_id) {
            Some(order) => order.clone(),
            None => return Err(OrderError::InvalidOrderID),
        };
        // print order
        dbg!("Matching order: {:?}", &order);

        let mut orders_to_remove: Vec<uuid::Uuid> = Vec::new();

        // match orders depending on order side, set price level to either bid or ask depending on order side (should be opposite of order side)
        match order.order_side {
            OrderSide::BID => {
                let mut stop_loop: bool = false;
                // get clone of ask price_levels
                let ask_price_levels: BTreeMap<String, PriceLevel> = self.ask_price_levels.clone();
                let it = ask_price_levels.iter();
                for (_, p_level) in it {
                    if stop_loop {
                        break;
                    }

                    for order_to_match_id in p_level.orders.iter() {
                        if order.qty == 0 {
                            orders_to_remove.push(order.order_id);
                            stop_loop = true;
                            break;
                        }

                        let order_to_match: Order = match self.oid_map.get(&order_to_match_id) {
                            Some(order) => order.clone(),
                            None => return Err(OrderError::InvalidOrderID),
                        };

                        // if order is a limit order, and the current price level is higher than limit order price, break
                        if order.order_type == OrderType::LIMIT
                            && order_to_match.price.unwrap() > order.price.unwrap()
                        {
                            stop_loop = true;
                            break;
                        }

                        if order_to_match.qty < order.qty {
                            order.qty -= order_to_match.qty;
                            orders_to_remove.push(*order_to_match_id);
                        } else {
                            // update order qty, and order_to_match with self.modify order
                            let order_to_match_qty: i32 = order_to_match.qty - order.qty;
                            match self.modify_order(
                                *order_to_match_id,
                                order_to_match_qty,
                                order_to_match.price,
                            ) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            order.qty = 0;
                        }
                    }
                }

                // modify order
                match self.modify_order(order_id, order.qty, order.price) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            OrderSide::ASK => {
                let mut stop_loop: bool = false;
                // get clone of bid price_levels
                let bid_price_levels: BTreeMap<String, PriceLevel> = self.bid_price_levels.clone();
                let it = bid_price_levels.iter();
                for (_, p_level) in it {
                    if stop_loop {
                        break;
                    }

                    for order_to_match_id in p_level.orders.iter() {
                        if order.qty == 0 {
                            orders_to_remove.push(order.order_id);
                            stop_loop = true;
                            break;
                        }

                        let order_to_match: Order = match self.oid_map.get(&order_to_match_id) {
                            Some(order) => order.clone(),
                            None => return Err(OrderError::InvalidOrderID),
                        };

                        // if order is a limit order, and the current price level is lower than limit order price, break
                        if order.order_type == OrderType::LIMIT
                            && order_to_match.price.unwrap() < order.price.unwrap()
                        {
                            stop_loop = true;
                            break;
                        }

                        if order_to_match.qty < order.qty {
                            order.qty -= order_to_match.qty;
                            orders_to_remove.push(*order_to_match_id);
                        } else {
                            // update order qty, and order_to_match with self.modify order
                            let order_to_match_qty: i32 = order_to_match.qty - order.qty;
                            match self.modify_order(
                                *order_to_match_id,
                                order_to_match_qty,
                                order_to_match.price,
                            ) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            order.qty = 0;
                        }
                    }
                }

                // modify order
                match self.modify_order(order_id, order.qty, order.price) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        // remove orders
        for order_id in orders_to_remove {
            match self.delete_order(order_id) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    // print orderbook, with asks and bids side by side in a table, along with quantities at each price level
    pub fn print_orderbook(&self) {
        let mut table = Table::new();
        table.add_row(row!["BID", "ASK"]);
        let mut bid_it = self.bid_price_levels.iter().rev();
        let mut ask_it = self.ask_price_levels.iter();
        loop {
            let bid = bid_it.next();
            let ask = ask_it.next();
            match (bid, ask) {
                (Some(bid), Some(ask)) => {
                    table.add_row(row![
                        format!("{} @ {}", bid.1.qty, bid.0),
                        format!("{} @ {}", ask.1.qty, ask.0)
                    ]);
                }
                (Some(bid), None) => {
                    table.add_row(row![format!("{} @ {}", bid.1.qty, bid.0), ""]);
                }
                (None, Some(ask)) => {
                    table.add_row(row!["", format!("{} @ {}", ask.1.qty, ask.0)]);
                }
                (None, None) => {
                    break;
                }
            }
        }
        table.printstd();
    }
}
