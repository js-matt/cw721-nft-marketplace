use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Invalid funds: {msg}")]
    InvalidFunds { msg: String },

    #[error("Highest bidder cannot out bid")]
    HighestBidderCannotOutBid {},

    #[error("Bid smaller than highest bid")]
    BidSmallerThanHighestBid {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Auction not ended")]
    AuctionNotEnded {},

    #[error("Auction reward already claimed")]
    AuctionAlreadyClaimed {},
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid expiration")]
    InvalidExpiration {},

    #[error("Invalid Start time. Current time: {current_time}. Current block: {current_block}")]
    InvalidStartTime {
        current_time: u64,
        current_block: u64,
    },

    #[error("Overflow")]
    Overflow {},

    #[error("Auction does not exist")]
    AuctionDoesNotExist {},

    #[error("Auction cancelled")]
    AuctionCancelled {},

    #[error("Auction not started")]
    AuctionNotStarted {},

    #[error("Auction ended")]
    AuctionEnded {},

    #[error("Token owner cannot bid")]
    TokenOwnerCannotBid {},
}

impl From<OverflowError> for ContractError {
    fn from(_err: OverflowError) -> Self {
        ContractError::Overflow {}
    }
}
