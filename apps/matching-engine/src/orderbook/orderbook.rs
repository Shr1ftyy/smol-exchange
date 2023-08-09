use std::collections::HashMap;
use std::collections::VecDeque;
// TODO: Is this ok?
use crate::errors::OrderError;

#[derive(Debug)]
pub enum OrderType {
    BUY,
    SELL,
}

#[derive(Debug)]
pub struct Order {
    pub order_id: i32,
    pub creator_id: i32,
    pub stock_id: i32,
    pub order_type: OrderType,
    pub price: i32,
    pub qty: i32,
    pub time_created: u32,
}

#[derive(Debug)]
pub struct PriceLevel {
    pub price: i32,
    pub qty: i32,
    pub orders: VecDeque<Order>,
}

#[derive(Debug)]
pub struct OrderBook {
    pub stock_id: i32,
    pub price_levels: HashMap<i32, PriceLevel>,
    pub oid_map: HashMap<i32, Order>,
}

pub struct OrderBooks {
    pub order_books: HashMap<i32, OrderBook>
}

impl Order {
    pub fn validate(&self) -> Result<(), OrderError> {
        if self.order_id <= 0 {
            return Err(OrderError::InvalidOrderID);
        }
        if self.creator_id <= 0 {
            return Err(OrderError::InvalidCreatorID);
        }
        if self.stock_id <= 0 {
            return Err(OrderError::InvalidStockID);
        }
        if self.price <= 0 {
            return Err(OrderError::InvalidPrice);
        }
        if self.qty == 0 {
            return Err(OrderError::InvalidQuantity);
        }
        if self.time_created == 0 {
            return Err(OrderError::InvalidTimeCreated);
        }
        
        Ok(())  // If all checks pass, return Ok(())
    }

    pub fn new(
        order_id: i32, 
        creator_id: i32, 
        stock_id: i32, 
        order_type: OrderType, 
        price: i32, 
        qty: i32, 
        time_created: u32
    ) -> Self {
        let order: Order = Order {
            order_id,
            creator_id,
            stock_id,
            order_type,
            price,
            qty,
            time_created
        };

        match order.validate() {
            Ok(_) => println!("Order is valid!"),
            Err(e) => println!("Error validating order: {:?}", e),
        }

        order
    }

}


impl OrderBook {
    // create orderbook
    pub fn new(id: i32) -> OrderBook {
        OrderBook {
            stock_id: id,
            price_levels: HashMap::<i32, PriceLevel>::new(),
            oid_map: HashMap::<i32, Order>::new(),
        }
    }

    // validate order
 

    // add order
    // pub fn add_order(&self, order: Order) {
    //     self.insert(order-)
    //     errors
    // }
    // modify order
    // delete order
    // match order
}
