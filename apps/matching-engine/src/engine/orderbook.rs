use std::collections::BTreeMap;
use std::collections::VecDeque;
// TODO: Is this ok?
use serde::{Serialize, Deserialize};
use crate::errors::OrderError;
use crate::errors::PriceLevelError;

extern crate redis;
use redis::Commands;

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderType {
    BUY,
    SELL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: uuid::Uuid,
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
    pub price_levels: BTreeMap<i32, PriceLevel>,
    pub oid_map: BTreeMap<i32, Order>,
}

// impl PriceLevel {
//     pub fn validate(p_level: PriceLevel) -> Result<(), PriceLevelError> {
//         if p_level.price <= 0 {
//             return Err(PriceLevelError::InvalidPrice);
//         }
//         if p_level.qty == 0 {
//             return Err(PriceLevelError::InvalidQuantity);
//         }
//         Ok(())
//     }

//     pub fn new(price: i32) -> Result<Self, PriceLevelError> {
//         if price < 0 {
//             return Err(PriceLevelError::InvalidPrice);
//         }

//         let price_level = PriceLevel {
//             price,
//             qty: 0,
//             orders: VecDeque::<Order>::new(),
//         };

//         Ok(price_level)
//     }
// }

impl Order {
    pub fn validate(&self) -> Result<(), OrderError> {
        if self.order_id.is_nil() {
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

        Ok(()) // If all checks pass, return Ok(())
    }

    pub fn new(
        order_id: uuid::Uuid,
        creator_id: i32,
        stock_id: i32,
        order_type: OrderType,
        price: i32,
        qty: i32,
        time_created: u32,
    ) -> Self {
        let order: Order = Order {
            order_id,
            creator_id,
            stock_id,
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

// impl OrderBook {
//     // create orderbook
//     pub fn new(id: i32) -> Self {
//         OrderBook {
//             stock_id: id,
//             price_levels: BTreeMap::<i32, PriceLevel>::new(),
//             oid_map: BTreeMap::<i32, Order>::new(),
//         }
//     }

//     // validate order

//     // add order
//     // pub fn add_order(&mut self, order: Order) {
//     //     let order: Option<Order> = self.oid_map.insert(order.order_id, order);
//     //     match order {
//     //         None => println!("insertion failed"),
//     //         Some(order) => {
//     //             let new_price_level: Result<PriceLevel, PriceLevelError> =
//     //                 PriceLevel::new(order.price);
//     //             match new_price_level {
//     //                 Ok(new_price_level) => {
//     //                     println!("new price level: {:?}", new_price_level);
//     //                     self.price_levels.insert(order.price, new_price_level);
//     //                 }
//     //                 Err(e) => println!("Error creating new price level: {:?}", e),
//     //             }
//     //         }
//     //     }
//     // }

//     pub fn add_order(&mut self, order: Order) {
//     }
//     // modify order
//     // delete order
//     // match order
// }
