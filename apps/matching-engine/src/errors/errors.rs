#[derive(Debug)]
pub enum OrderError {
    InvalidOrderID,
    InvalidCreatorID,
    InvalidStockID,
    InvalidPrice,
    InvalidQuantity,
    InvalidOrderType,
    InvalidTimeCreated,
    Other(String), // Catch-all for unexpected errors, with a descriptive message.
}

#[derive(Debug)]
pub enum PriceLevelError {
    InvalidPrice,
    InvalidQuantity,
    Other(String), // Catch-all for unexpected errors, with a descriptive message.
}