use crate::{
    error::ContractError,
    state::{
        auction_details, load_next_auction_id, load_nft_auction_state, save_next_auction_id,
        NFTAuctionState,
    },
};
use cosmwasm_std::{to_json_binary, QuerierWrapper, QueryRequest, Storage, Uint128, WasmQuery};
use cw721::{Cw721QueryMsg, OwnerOfResponse};

pub fn fetch_and_update_next_auction_id(
    storage: &mut dyn Storage,
) -> Result<Uint128, ContractError> {
    let next_auction_id = load_next_auction_id(storage)?;

    let incremented_next_auction_id = next_auction_id.checked_add(Uint128::from(1u128))?;
    save_next_auction_id(storage, incremented_next_auction_id)?;

    Ok(next_auction_id)
}

pub fn fetch_latest_auction_state_for_token(
    storage: &dyn Storage,
    token_id: &str,
    token_address: &str,
) -> Result<NFTAuctionState, ContractError> {
    let key = token_id.to_owned() + token_address;
    let latest_auction_id: Uint128 = match auction_details().may_load(storage, &key)? {
        None => return Err(ContractError::AuctionDoesNotExist {}),
        Some(auction_info) => *auction_info.latest().unwrap(),
    };
    let token_auction_state = load_nft_auction_state(storage, latest_auction_id.u128())?;

    Ok(token_auction_state)
}

pub fn query_token_owner(
    querier: QuerierWrapper,
    token_addr: String,
    token_id: String,
) -> Result<OwnerOfResponse, ContractError> {
    let res: OwnerOfResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_addr,
        msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
            token_id,
            include_expired: None,
        })?,
    }))?;

    Ok(res)
}
