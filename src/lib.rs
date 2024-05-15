mod contract;
mod error;
#[cfg(test)]
pub mod mock;
pub mod msg;
mod state;
mod testing;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};
use {
    contract::{
        exec::{
            cancel_auction_and_refund, finalize_auction_and_transfer_assets,
            handle_cw721_auction_start, submit_bid_for_auction,
        },
        query::{get_auction_details, get_auction_state_by_id, get_bids_for_auction},
    },
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::save_next_auction_id,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    save_next_auction_id(deps.storage, Uint128::from(1u128))?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AuctionStart(msg) => handle_cw721_auction_start(deps, env, info, msg),
        ExecuteMsg::SubmitBid {
            token_id,
            token_address,
        } => submit_bid_for_auction(deps, env, info, token_id, token_address),
        ExecuteMsg::CancelAuctionAndRefund {
            token_id,
            token_address,
        } => cancel_auction_and_refund(deps, env, info, token_id, token_address),
        ExecuteMsg::FinalizeAuctionAndTranferAssets {
            token_id,
            token_address,
        } => finalize_auction_and_transfer_assets(deps, env, info, token_id, token_address),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> Result<Binary, ContractError> {
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
