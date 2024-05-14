use crate::{
    error::ContractError,
    msg::Cw721CustomMsg,
    state::{
        auction_details, get_bids, read_auction_details, AuctionDetails, Bid, NFTAuctionState,
        OrderBy, BIDS, NEXT_AUCTION_ID, NFT_AUCTION_STATE,
    },
};
use cosmwasm_std::{
    attr, coins, ensure, from_json, to_json_binary, Addr, BankMsg, BlockInfo, Coin, CosmosMsg,
    Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Response, Storage, Timestamp,
    Uint128, WasmMsg, WasmQuery,
};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, Cw721ReceiveMsg, Expiration, OwnerOfResponse};

fn fetch_and_update_next_auction_id(storage: &mut dyn Storage) -> Result<Uint128, ContractError> {
    let next_auction_id = NEXT_AUCTION_ID.load(storage)?;

    let incremented_next_auction_id = next_auction_id.checked_add(Uint128::from(1u128))?;
    NEXT_AUCTION_ID.save(storage, &incremented_next_auction_id)?;

    Ok(next_auction_id)
}

fn convert_milliseconds_to_expiration(time: u64) -> Result<Expiration, ContractError> {
    ensure!(
        time <= u64::MAX / 1000000,
        ContractError::InvalidExpiration {}
    );

    Ok(Expiration::AtTime(Timestamp::from_nanos(time * 1000000)))
}

fn set_expiration_from_block(block: &BlockInfo, model: Expiration) -> Option<Expiration> {
    match model {
        Expiration::AtTime(_) => Some(Expiration::AtTime(block.time)),
        Expiration::AtHeight(_) => Some(Expiration::AtHeight(block.height)),
        Expiration::Never {} => None,
    }
}

fn fetch_latest_auction_state_for_token(
    storage: &dyn Storage,
    token_id: &str,
    token_address: &str,
) -> Result<NFTAuctionState, ContractError> {
    let key = token_id.to_owned() + token_address;
    let latest_auction_id: Uint128 = match auction_details().may_load(storage, &key)? {
        None => return Err(ContractError::AuctionDoesNotExist {}),
        Some(auction_info) => *auction_info.latest().unwrap(),
    };
    let token_auction_state = NFT_AUCTION_STATE.load(storage, latest_auction_id.u128())?;

    Ok(token_auction_state)
}

fn query_token_owner(
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
