use crate::{
    error::ContractError,
    helper::{
        convert_milliseconds_to_expiration, fetch_and_update_next_auction_id,
        fetch_latest_auction_state_for_token, query_token_owner, set_expiration_from_block,
    },
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

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> Result<Binary, ContractError> {
  use contract::{get_auction_details, get_auction_state_by_id, get_bids_for_auction};
  match msg {
      QueryMsg::AuctionDetails {
          token_address,
          start_after,
          limit,
      } => to_json_binary(&get_auction_details(
          deps,
          token_address,
          start_after,
          limit,
      )?)
      .map_err(|err| err.into()),
      QueryMsg::Bids {
          auction_id,
          start_after,
          limit,
          order_by,
      } => to_json_binary(&get_bids_for_auction(
          deps,
          auction_id,
          start_after,
          limit,
          order_by,
      )?)
      .map_err(|err| err.into()),
      QueryMsg::AuctionState { auction_id } => {
          to_json_binary(&get_auction_state_by_id(deps, auction_id)?).map_err(|err| err.into())
      }
  }
}

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
    let token_auction_state = NFT_AUCTION_STATE.load(deps.storage, auction_id.u128())?;
    Ok(token_auction_state)
}
