#[allow(unused_imports)]
use crate::state::{AuctionDetails, Bid, NFTAuctionState, OrderBy};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    AuctionStart(cw721::Cw721ReceiveMsg),
    SubmitBid {
        token_id: String,
        token_address: String,
    },
    CancelAuctionAndRefund {
        token_id: String,
        token_address: String,
    },
    FinalizeAuctionAndTranferAssets {
        token_id: String,
        token_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AuctionDetails)]
    AuctionDetails {
        token_address: Option<String>,
        start_after: Option<String>,
        limit: Option<u64>,
    },
    #[returns(NFTAuctionState)]
    AuctionState { auction_id: Uint128 },
    #[returns(Vec<Bid>)]
    Bids {
        auction_id: Uint128,
        start_after: Option<u64>,
        limit: Option<u64>,
        order_by: Option<OrderBy>,
    },
}

#[cw_serde]
pub enum Cw721CustomMsg {
    InitializeCW721TokenAuction {
        start_time: u64,
        duration: u64,
        coin_denomination: String,
        min_bid: Option<Uint128>,
    },
}
