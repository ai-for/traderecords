use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TradeRecordError {
    #[error("expected at least {required} buy reasons, got {provided}")]
    NotEnoughBuyReasons { required: usize, provided: usize },
    #[error("expected at least {required} sell reasons, got {provided}")]
    NotEnoughSellReasons { required: usize, provided: usize },
    #[error("reason at index {index} is empty")]
    EmptyReason { index: usize },
    #[error("take profit price must be greater than zero")]
    InvalidTakeProfit,
    #[error("stop loss price must be greater than zero")]
    InvalidStopLoss,
}
