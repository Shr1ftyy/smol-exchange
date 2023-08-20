use std::fmt;

#[derive(Debug)]
pub enum OrderError {
    InvalidOrderID,
    InvalidCreatorID,
    InvalidStockID,
    InvalidPrice,
    InvalidQuantity,
    InvalidOrderSide,
    InvalidTimeCreated,
    OrderQueueEmpty,
    Other(String), // Catch-all for unexpected errors, with a descriptive message.
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderError::InvalidOrderID => write!(f, "InvalidOrderID"),
            OrderError::InvalidCreatorID => write!(f, "InvalidCreatorID"),
            OrderError::InvalidStockID => write!(f, "InvalidStockID"),
            OrderError::InvalidPrice => write!(f, "InvalidPrice"),
            OrderError::InvalidQuantity => write!(f, "InvalidQuantity"),
            OrderError::InvalidOrderSide => write!(f, "InvalidOrderSide"),
            OrderError::InvalidTimeCreated => write!(f, "InvalidTimeCreated"),
            OrderError::OrderQueueEmpty => write!(f, "OrderQueueEmpty"),
            OrderError::Other(e) => write!(f, "Other: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum StockError {
    InvalidStockID,
    InvalidName,
    InvalidTicker,
    InvalidTimeCreated,
    InvalidTotalIssued,
    InvalidOutstandingShares,
    Other(String), // Catch-all for unexpected errors, with a descriptive message.
}

impl fmt::Display for StockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StockError::InvalidStockID => write!(f, "InvalidStockID"),
            StockError::InvalidName => write!(f, "InvalidName"),
            StockError::InvalidTicker => write!(f, "InvalidTicker"),
            StockError::InvalidTimeCreated => write!(f, "InvalidTimeCreated"),
            StockError::InvalidTotalIssued => write!(f, "InvalidTotalIssued"),
            StockError::InvalidOutstandingShares => write!(f, "InvalidOutstandingShares"),
            StockError::Other(e) => write!(f, "Other: {}", e),
        }
    }
}

pub enum OrderQueueError {
    OrderQueueEmpty,
    Other(String), // Catch-all for unexpected errors, with a descriptive message.
}