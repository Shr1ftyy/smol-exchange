use core::fmt;
use std::collections::BTreeMap;
use std::collections::VecDeque;
// TODO: Is this ok?
use crate::errors::OrderError;
use crate::errors::StockError;
use crate::helpers::helpers;
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};
use serde_json::json;

extern crate redis;

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
    pub creator_id: uuid::Uuid,
    pub stock: Stock,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub qty: i32,
    pub time_created: u32,
    pub price: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f32,
    pub qty: i32,
    pub orders: VecDeque<uuid::Uuid>,
}

#[derive(Debug)]
pub struct OrderBook {
    pub stock_id: uuid::Uuid,
    pub stock_info: Stock,
    pub bid_price_levels: BTreeMap<String, PriceLevel>,
    pub ask_price_levels: BTreeMap<String, PriceLevel>,
    pub oid_map: BTreeMap<uuid::Uuid, Order>,
    pub order_queue: VecDeque<Order>,
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
        // the rest of the parameters are optional, so only check them if they are Some()
        if let Some(total_issued) = self.total_issued {
            if total_issued <= 0 {
                return Err(StockError::InvalidTotalIssued);
            }
        }
        if let Some(outstanding_shares) = self.outstanding_shares {
            if outstanding_shares <= 0 {
                return Err(StockError::InvalidOutstandingShares);
            }
        }
        if let Some(time_created) = self.time_created {
            if time_created <= 0 {
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
            Ok(_) => {}
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
        if self.creator_id <= uuid::Uuid::nil() {
            return Err(OrderError::InvalidCreatorID);
        }
        if self.stock.stock_id == uuid::Uuid::nil() {
            return Err(OrderError::InvalidStockID);
        }
        match self.price {
            Some(price) => {
                if price <= 0.0 {
                    return Err(OrderError::InvalidPrice);
                }
            }
            None => {}
        }
        match self.qty {
            qty if qty <= 0 => return Err(OrderError::InvalidQuantity),
            qty if qty > 1000000 => return Err(OrderError::InvalidQuantity),
            _ => {}
        }
        match self.time_created {
            time if time <= 0 => return Err(OrderError::InvalidTimeCreated),
            _ => {}
        }

        Ok(()) // If all checks pass, return Ok(())
    }

    pub fn new(
        order_id: uuid::Uuid,
        creator_id: uuid::Uuid,
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
            Ok(_) => {}
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
    // create new orderbook given stock
    pub fn new(stock: Stock) -> Self {
        let orderbook: OrderBook = OrderBook {
            stock_id: stock.stock_id,
            stock_info: stock,
            bid_price_levels: BTreeMap::new(),
            ask_price_levels: BTreeMap::new(),
            oid_map: BTreeMap::new(),
            order_queue: VecDeque::new(),
            last_market_price: None,
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

    pub fn get_stock(&self) -> Result<Stock, StockError> {
        let stock = self.oid_map.values().next().unwrap().stock.clone();
        Ok(stock)
    }

    pub fn queue_order(&mut self, order: Order) {
        self.order_queue.push_back(order);
    }

    pub fn add_order(&mut self, mut order: Order) -> Result<(), OrderError> {
        let price_key: String;

        match order.order_type {
            OrderType::MARKET => {
                // get last market price, and set order.price to the last market price
                match self.last_market_price {
                    Some(last_market_price) => {
                        order.price = Some(last_market_price);
                    }
                    None => {
                        // don't add order to orderbook and return error
                        return Err(OrderError::InvalidPrice);
                    }
                }
            }
            _ => {}
        }

        price_key = helpers::f32_to_string(order.price.unwrap(), 2);

        // add order to oid map and price level
        self.oid_map.insert(order.order_id, order.clone());
        match self.get_price_level(order.order_side, order.price.unwrap()) {
            Some(price_level) => {
                price_level.add_order(order);
            }
            None => {
                // create new price level
                let mut price_level: PriceLevel = PriceLevel::new(order.price.unwrap(), 0);
                price_level.add_order(order.clone());

                // add price level to orderbook
                match order.order_side {
                    OrderSide::BID => {
                        self.bid_price_levels.insert(price_key, price_level);
                    }
                    OrderSide::ASK => {
                        self.ask_price_levels.insert(price_key, price_level);
                    }
                }
            }
        }

        Ok(())
    }

    fn _remove_price_level(&mut self, order_side: OrderSide, price: f32) {
        match order_side {
            OrderSide::BID => {
                self.bid_price_levels
                    .remove(&helpers::f32_to_string(price, 2));
            }
            OrderSide::ASK => {
                self.ask_price_levels
                    .remove(&helpers::f32_to_string(price, 2));
            }
        }
    }

    // delete an order (affects pricelevel and orderbook)
    fn _delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let order: Order = match self.oid_map.get(&order_id) {
            Some(order) => order.clone(),
            None => return Err(OrderError::InvalidOrderID),
        };

        // remove order from price level
        let price_level =
            match self.get_price_level(order.clone().order_side, order.clone().price.unwrap()) {
                Some(price_level) => price_level,
                None => return Err(OrderError::InvalidPrice),
            };

        price_level.remove_order(order.clone());

        // delete price level if qty is les than 0
        if price_level.qty <= 0 {
            self._remove_price_level(order.clone().order_side, order.clone().price.unwrap());
        }

        // remove order from oid map
        self.oid_map.remove(&order_id);

        Ok(())
    }

    // wrapper for _delete_order that actually emits a deletion event
    pub fn delete_order(&mut self, order_id: uuid::Uuid) -> Result<(), OrderError> {
        self._delete_order(order_id)
    }

    // modifying an order (affects pricelevel and orderbook)
    fn _modify_order(
        &mut self,
        order_id: uuid::Uuid,
        new_qty: i32,
        new_price: Option<f32>,
    ) -> Result<(), OrderError> {
        // remove order if the new qty is 0 or less
        if new_qty <= 0 {
            return self._delete_order(order_id);
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

    // wrapper for _modify_order that actually emits a modification event
    pub fn modify_order(
        &mut self,
        order_id: uuid::Uuid,
        new_qty: i32,
        new_price: Option<f32>,
    ) -> Result<(), OrderError> {
        self._modify_order(order_id, new_qty, new_price)
    }

    // match order against orderbook given order id in orderbook
    pub fn match_order(&mut self, mut order: Order) -> Result<(), OrderError> {
        let price_level_to_search = match order.order_side {
            OrderSide::BID => self.ask_price_levels.clone(),
            OrderSide::ASK => self.bid_price_levels.clone(),
        };

        let it = price_level_to_search.iter();

        match order.order_side {
            OrderSide::BID => {
                for (_, p_level) in it {
                    if order.qty == 0
                        || (order.order_type == OrderType::LIMIT
                            && p_level.price > order.price.unwrap())
                    {
                        break;
                    }

                    let order_it = p_level.orders.iter();

                    for order_to_match_uuid in order_it {
                        if order.qty == 0 {
                            break;
                        }

                        let mut order_to_match = match self.oid_map.get(order_to_match_uuid) {
                            Some(order) => order.clone(),
                            None => return Err(OrderError::InvalidOrderID),
                        };

                        if order_to_match.qty > order.qty {
                            // order in orderbook has more qty than order to match
                            // subtract order qty from orderbook order qty
                            // subtract order qty from order qty
                            // add trade to tradebook
                            // delete order from orderbook
                            // modify order in orderbook
                            // return
                            let trade_qty = order.qty;
                            order.qty -= trade_qty;
                            order_to_match.qty -= trade_qty;
                            // instead of doing this, we create a new transaction, and eventually send it to the redis instance
                            // let trade = Trade::new(order.order_id, order.stock_id, order.price, trade_qty);
                            // self.tradebook.push_back(trade);
                            match self._modify_order(
                                order_to_match.order_id,
                                order_to_match.qty,
                                order_to_match.price,
                            ) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            // set last market price to the ask price
                            self.last_market_price = Some(order_to_match.price.unwrap());
                        } else {
                            // order in orderbook has less or equal qty than order to match
                            // subtract order qty from orderbook order qty
                            // subtract order qty from order qty
                            // add trade to tradebook
                            // delete order from orderbook
                            // modify order in orderbook
                            // continue
                            let trade_qty = order_to_match.qty;
                            order.qty -= trade_qty;
                            order_to_match.qty -= trade_qty;
                            // instead of doing this, we create a new transaction, and eventually send it to the redis instance
                            // let trade = Trade::new(order.order_id, order.stock_id, order.price, trade_qty);
                            // self.tradebook.push_back(trade);
                            match self._delete_order(order_to_match.order_id) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            // set last market price to the ask price
                            self.last_market_price = Some(order_to_match.price.unwrap());
                        }
                    }
                }
            }
            OrderSide::ASK => {
                for (_, p_level) in it.rev() {
                    if order.qty == 0
                        || (order.order_type == OrderType::LIMIT
                            && p_level.price < order.price.unwrap())
                    {
                        break;
                    }

                    let order_it = p_level.orders.iter();

                    for order_to_match_uuid in order_it {
                        if order.qty == 0 {
                            break;
                        }

                        let mut order_to_match = match self.oid_map.get(order_to_match_uuid) {
                            Some(order) => order.clone(),
                            None => return Err(OrderError::InvalidOrderID),
                        };

                        if order_to_match.qty > order.qty {
                            // order in orderbook has more qty than order to match
                            // subtract order qty from orderbook order qty
                            // subtract order qty from order qty
                            // add trade to tradebook
                            // delete order from orderbook
                            // modify order in orderbook
                            // return
                            let trade_qty = order.qty;
                            order.qty -= trade_qty;
                            order_to_match.qty -= trade_qty;
                            // instead of doing this, we create a new transaction, and eventually send it to the redis instance
                            // let trade = Trade::new(order.order_id, order.stock_id, order.price, trade_qty);
                            // self.tradebook.push_back(trade);
                            match self._modify_order(
                                order_to_match.order_id,
                                order_to_match.qty,
                                order_to_match.price,
                            ) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            // set last market price to the ask price
                            self.last_market_price = Some(order_to_match.price.unwrap());
                        } else {
                            // order in orderbook has less or equal qty than order to match
                            // subtract order qty from orderbook order qty
                            // subtract order qty from order qty
                            // add trade to tradebook
                            // delete order from orderbook
                            // modify order in orderbook
                            // continue
                            let trade_qty = order_to_match.qty;
                            order.qty -= trade_qty;
                            order_to_match.qty -= trade_qty;
                            // instead of doing this, we create a new transaction, and eventually send it to the redis instance
                            // let trade = Trade::new(order.order_id, order.stock_id, order.price, trade_qty);
                            // self.tradebook.push_back(trade);
                            match self._delete_order(order_to_match.order_id) {
                                Ok(_) => {}
                                Err(e) => return Err(e),
                            }

                            // set last market price to the ask price
                            self.last_market_price = Some(order_to_match.price.unwrap());
                        }
                    }
                }
            }
        }

        // add order to orderbook if it still has qty
        if order.qty > 0 {
            match self.add_order(order.clone()) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    // executes order from queue (matches order)
    pub fn execute_order(&mut self) -> Result<(), OrderError> {
        // get order from queue
        let order: Order = match self.order_queue.pop_front() {
            Some(order) => order,
            None => return Err(OrderError::OrderQueueEmpty),
        };

        // attempt to match order
        match self.match_order(order) {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    // execute all orders in queue until empty
    pub fn execute_all_orders(&mut self) -> Result<(), OrderError> {
        loop {
            match self.execute_order() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
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

impl Exchange {
    // create new exchange
    pub fn new() -> Self {
        let exchange: Exchange = Exchange {
            orderbooks: BTreeMap::new(),
        };
        exchange
    }

    // add stock to exchange
    pub fn add_stock(&mut self, stock: Stock) {
        self.orderbooks
            .insert(stock.stock_id.to_string(), OrderBook::new(stock));
    }

    // get stock from exchange
    pub fn get_stock(&mut self, stock_id: uuid::Uuid) -> Result<Stock, StockError> {
        match self.orderbooks.get_mut(&stock_id.to_string()) {
            Some(orderbook) => {
                // TODO: this might panic, fix later???
                let stock = orderbook.get_stock().unwrap();
                Ok(stock)
            }
            None => Err(StockError::InvalidStockID),
        }
    }

    // add order to exchange
    pub fn execute_order(&mut self, order: Order) -> Result<(), OrderError> {
        let stock_ticker = order.stock.ticker.clone();
        let orderbook = match self.orderbooks.get_mut(&stock_ticker) {
            Some(orderbook) => orderbook,
            None => return Err(OrderError::InvalidStockID),
        };

        // queue order
        orderbook.queue_order(order);

        // execute order
        match orderbook.execute_order() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // modify order
    pub fn modify_order(
        &mut self,
        stock: Stock,
        order_id: uuid::Uuid,
        new_qty: i32,
        new_price: Option<f32>,
    ) -> Result<(), OrderError> {
        // get stock id from oid map
        let orderbook = match self.orderbooks.get_mut(&stock.stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => return Err(OrderError::InvalidStockID),
        };

        // modify order
        match orderbook.modify_order(order_id, new_qty, new_price) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // delete order
    pub fn delete_order(&mut self, stock: Stock, order_id: uuid::Uuid) -> Result<(), OrderError> {
        let orderbook = match self.orderbooks.get_mut(&stock.stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => return Err(OrderError::InvalidStockID),
        };

        // delete order
        match orderbook.delete_order(order_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // print orderbook
    pub fn print_orderbook(&self, stock_id: uuid::Uuid) {
        let orderbook = match self.orderbooks.get(&stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => return,
        };

        orderbook.print_orderbook();
    }

    // print stock
    pub fn print_stock(&self, stock_id: uuid::Uuid) {
        let orderbook = match self.orderbooks.get(&stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => return,
        };

        let stock = orderbook.get_stock().unwrap();

        println!("Stock: {:?}", stock);
    }

    // print stocks
    pub fn print_stocks(&self) {
        for (_, orderbook) in self.orderbooks.iter() {
            let stock = orderbook.get_stock().unwrap();
            println!("Stock: {:?}", stock);
        }
    }

    // return json of bid and ask price levels with quantities in serde json
    pub fn get_orderbook_json(&self, stock_id: uuid::Uuid) -> String {
        let orderbook = match self.orderbooks.get(&stock_id.to_string()) {
            Some(orderbook) => orderbook,
            None => return String::from(""),
        };

        let mut bid_price_levels: Vec<PriceLevel> = Vec::new();
        let mut ask_price_levels: Vec<PriceLevel> = Vec::new();

        for (_, price_level) in orderbook.bid_price_levels.iter() {
            bid_price_levels.push(price_level.clone());
        }

        for (_, price_level) in orderbook.ask_price_levels.iter() {
            ask_price_levels.push(price_level.clone());
        }

        let orderbook_json = json!({
            "bid_price_levels": bid_price_levels,
            "ask_price_levels": ask_price_levels,
        });

        orderbook_json.to_string()
    }
}
