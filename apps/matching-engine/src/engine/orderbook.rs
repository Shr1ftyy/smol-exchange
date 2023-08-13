use core::fmt;
use std::collections::BTreeMap;
use std::collections::VecDeque;
// TODO: Is this ok?
use crate::errors::OrderError;
use crate::errors::StockError;
use crate::helpers::helpers;
use serde::{Deserialize, Serialize};
use prettytable::{Table, row};

extern crate redis;
use redis::Commands;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    BID,
    ASK,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    pub orders: VecDeque<Order>,
}

#[derive(Debug)]
pub struct OrderBook {
    pub stock_id: uuid::Uuid,
    pub bid_price_levels: BTreeMap<String, PriceLevel>,
    pub ask_price_levels: BTreeMap<String, PriceLevel>,
    pub oid_map: BTreeMap<uuid::Uuid, Order>,
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

    pub fn add_order(&mut self, order: Order) {
        self.orders.push_back(order.clone());
        self.qty += order.qty;
    }

    pub fn remove_order(&mut self, order_id: uuid::Uuid) {
        // update quantity
        let order: &Order = self
            .orders
            .iter()
            .find(|order| order.order_id == order_id)
            .unwrap();
        self.qty -= order.qty;
        self.orders.retain(|order| order.order_id != order_id);
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

    pub fn add_order(&mut self, order: Order) -> Result<(), OrderError> {
        match order.price {
            Some(price) => {
                let price_key = helpers::f32_to_string(price, 2);
                let price_level: &mut PriceLevel = match order.order_side {
                    OrderSide::BID => self
                        .bid_price_levels
                        .entry(price_key.clone())
                        .or_insert(PriceLevel::new(price, order.qty)),
                    OrderSide::ASK => self
                        .ask_price_levels
                        .entry(price_key.clone())
                        .or_insert(PriceLevel::new(price, order.qty)),

                };

                price_level.add_order(order.clone());
                self.oid_map.insert(order.clone().order_id, order);

                Ok(())
            },
            None => {
                return Err(OrderError::InvalidPrice);
            }
        }
    }

    pub fn remove_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let order: &Order = self.oid_map.get(&order_id).unwrap();
        let price_level: &mut PriceLevel = match order.order_side {
            OrderSide::BID => self
                .bid_price_levels
                .get_mut(&helpers::f32_to_string(order.price.unwrap(), 2))
                .unwrap(),
            OrderSide::ASK => self
                .ask_price_levels
                .get_mut(&helpers::f32_to_string(order.price.unwrap(), 2))
                .unwrap(),
        };

        price_level.remove_order(order_id);
        self.oid_map.remove(&order_id);

        Ok(())
    }

    // helpers
    pub fn get_price_level(&self, order_side: OrderSide, price: f32) -> Option<&PriceLevel> {
        match order_side {
            OrderSide::BID => self.bid_price_levels.get(&helpers::f32_to_string(price, 2)),
            OrderSide::ASK => self.ask_price_levels.get(&helpers::f32_to_string(price, 2)),
        }
    }

    pub fn get_oid_map(&self) -> &BTreeMap<uuid::Uuid, Order> {
        &self.oid_map
    }

    pub fn get_stock_id(&self) -> uuid::Uuid {
        self.stock_id
    }

    // match order against orderbook given order id in orderbook
    pub fn match_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let order: &Order = self.oid_map.get(&order_id)
            .ok_or(OrderError::InvalidOrderID)?;
        let p_level_to_search = match order.order_side {
            OrderSide::BID => &mut self.ask_price_levels,
            OrderSide::ASK => &mut self.bid_price_levels,
        };

        let mut qty: i32 = order.qty;
        let mut orders_to_remove: VecDeque<uuid::Uuid> = VecDeque::new();

        // match order against orders in price levels, the logic of how this is done depends on the order type (LIMIT vs MARKET) and it's side (BID vs ASK)
        match order.order_type {
            OrderType::LIMIT => {
                match order.order_side {
                    OrderSide::BID => {
                        let mut price_key = helpers::f32_to_string(order.price.unwrap(), 2); // Assuming you've converted the price to a string key
                        while let Some(p_level) = p_level_to_search.get_mut(&price_key) {
                            // break condition
                            if qty == 0 {
                                break;
                            }
                            let mut it = p_level.orders.iter_mut().peekable();
                            while let Some(order_to_match) = it.next() {
                                if qty == 0 || order_to_match.price > order.price {
                                    orders_to_remove.push_back(order.order_id);
                                    break;
                                }

                                if order_to_match.qty <= qty {
                                    qty -= order_to_match.qty;
                                    orders_to_remove.push_back(order_to_match.order_id);
                                } else {
                                    order_to_match.qty -= qty;
                                    // update quantity of price level
                                    p_level.qty -= qty;
                                    qty = 0;
                                }

                                if it.peek().is_none() {
                                    price_key =
                                        helpers::f32_to_string(price_key.parse::<f32>().unwrap() - 0.01, 2);
                                    break;
                                }
                            }
                        }
                        // update order in price levels, oid map, etc.
                    }
                    OrderSide::ASK => {
                        let mut price_key = helpers::f32_to_string(order.price.unwrap(), 2); // Assuming you've converted the price to a string key
                        while let Some(p_level) = p_level_to_search.get_mut(&price_key) {
                            // break condition
                            if qty == 0 {
                                break;
                            }
                            let mut it = p_level.orders.iter_mut().rev().peekable();
                            while let Some(order_to_match) = it.next() {
                                if qty == 0 || order_to_match.price < order.price {
                                    break;
                                }

                                if order_to_match.qty <= qty {
                                    qty -= order_to_match.qty;
                                    orders_to_remove.push_back(order_to_match.order_id);
                                } else {
                                    order_to_match.qty -= qty;
                                    p_level.qty -= qty;
                                    qty = 0;
                                }

                                if it.peek().is_none() {
                                    price_key =
                                        helpers::f32_to_string(price_key.parse::<f32>().unwrap() - 0.01, 2);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            OrderType::MARKET => {
                match order.order_side {
                    OrderSide::BID => {
                        // get market price (maximum bid price in p_level_to_search)
                        let mut price_key = p_level_to_search.iter().last().unwrap().0.clone();
                        while let Some(p_level) = p_level_to_search.get_mut(&price_key) {
                            // break condition
                            if qty == 0 {
                                break;
                            }
                            let mut it = p_level.orders.iter_mut().peekable();
                            while let Some(order_to_match) = it.next() {
                                if qty == 0 {
                                    break;
                                }

                                if order_to_match.qty <= qty {
                                    qty -= order_to_match.qty;
                                    orders_to_remove.push_back(order_to_match.order_id);
                                } else {
                                    order_to_match.qty -= qty;
                                    p_level.qty -= qty;
                                    qty = 0;
                                }

                                if it.peek().is_none() {
                                    price_key =
                                        helpers::f32_to_string(price_key.parse::<f32>().unwrap() - 0.01, 2);
                                    break;
                                }
                            }
                        }
                    }
                    OrderSide::ASK => {
                        let mut price_key = p_level_to_search.iter().last().unwrap().0.clone();
                        while let Some(p_level) = p_level_to_search.get_mut(&price_key) {
                            // break condition
                            if qty == 0 {
                                break;
                            }
                            let mut it = p_level.orders.iter_mut().rev().peekable();
                            while let Some(order_to_match) = it.next() {
                                if qty == 0 {
                                    break;
                                }

                                if order_to_match.qty <= qty {
                                    qty -= order_to_match.qty;
                                    orders_to_remove.push_back(order_to_match.order_id);
                                } else {
                                    order_to_match.qty -= qty;
                                    p_level.qty -= qty;
                                    qty = 0;
                                }

                                if it.peek().is_none() {
                                    price_key =
                                        helpers::f32_to_string(price_key.parse::<f32>().unwrap() - 0.01, 2);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // remove orders that have been filled
        for order_id in orders_to_remove {
            let res = self.remove_order(order_id);
            match res {
                Ok(_) => {
                    println!("Order {} removed", order_id);
                }
                Err(e) => {
                    return Err(e);
                }
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
                    table.add_row(row![format!("{} @ {}", bid.1.qty, bid.0), format!("{} @ {}", ask.1.qty, ask.0)]);
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
