use crate::{
    error::ContractError,
    state::{
        get_bids, load_nft_auction_state, read_auction_details, AuctionDetails, Bid,
        NFTAuctionState, OrderBy,
    },
};
use cosmwasm_std::{Deps, Uint128};

pub fn get_auction_details(
    deps: Deps,
    token_address: Option<String>,
    start_after: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<AuctionDetails>, ContractError> {
    read_auction_details(deps.storage, token_address, start_after, limit)
}

pub fn get_bids_for_auction(
    deps: Deps,
    auction_id: Uint128,
    start_after: Option<u64>,
    limit: Option<u64>,
    order_by: Option<OrderBy>,
) -> Result<Vec<Bid>, ContractError> {
    let bids = get_bids(
        deps.storage,
        auction_id.u128(),
        start_after,
        limit,
        order_by,
    )?;
    Ok(bids)
}

pub fn get_auction_state_by_id(
    deps: Deps,
    auction_id: Uint128,
) -> Result<NFTAuctionState, ContractError> {
    let token_auction_state = load_nft_auction_state(deps.storage, auction_id.u128())?;
    Ok(token_auction_state)
}
