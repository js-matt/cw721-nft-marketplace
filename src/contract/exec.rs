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
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  use contract::{
      cancel_auction_and_refund, finalize_auction_and_transfer_assets,
      handle_cw721_auction_start, submit_bid_for_auction,
  };
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

pub fn handle_cw721_auction_start(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_json(&msg.msg)? {
        Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time,
            duration,
            coin_denomination,
            min_bid,
        } => initialize_cw721_token_auction(
            deps,
            env,
            msg.sender,
            msg.token_id,
            info.sender.to_string(),
            start_time,
            duration,
            coin_denomination,
            min_bid,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn initialize_cw721_token_auction(
    deps: DepsMut,
    env: Env,
    sender: String,
    token_id: String,
    token_address: String,
    start_time: u64,
    duration: u64,
    coin_denomination: String,
    min_bid: Option<Uint128>,
) -> Result<Response, ContractError> {
    ensure!(
        start_time > 0 && duration > 0,
        ContractError::InvalidExpiration {}
    );

    let start_expiration = convert_milliseconds_to_expiration(start_time)?;
    let end_expiration = convert_milliseconds_to_expiration(start_time + duration)?;

    let block_time = set_expiration_from_block(&env.block, start_expiration).unwrap();
    ensure!(
        start_expiration.gt(&block_time),
        ContractError::InvalidStartTime {
            current_time: env.block.time.nanos() / 1000000,
            current_block: env.block.height,
        }
    );

    let auction_id = fetch_and_update_next_auction_id(deps.storage)?;
    let pk = token_id.to_owned() + &token_address;

    let mut auction_info = auction_details()
        .load(deps.storage, &pk)
        .unwrap_or_default();
    auction_info.push(auction_id);
    if auction_info.token_address.is_empty() {
        auction_info.token_address = token_address.to_owned();
        auction_info.token_id = token_id.to_owned();
    }
    auction_details().save(deps.storage, &pk, &auction_info)?;

    BIDS.save(deps.storage, auction_id.u128(), &vec![])?;

    NFT_AUCTION_STATE.save(
        deps.storage,
        auction_id.u128(),
        &NFTAuctionState {
            start: start_expiration,
            end: end_expiration,
            high_bidder_addr: Addr::unchecked(""),
            high_bidder_amount: Uint128::zero(),
            coin_denomination: coin_denomination.clone(),
            auction_id,
            min_bid,
            owner: sender,
            token_id,
            token_address,
            is_cancelled: false,
        },
    )?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "start_auction"),
        attr("start_time", start_expiration.to_string()),
        attr("end_time", end_expiration.to_string()),
        attr("coin_denomination", coin_denomination),
        attr("auction_id", auction_id.to_string()),
    ]))
}

pub fn submit_bid_for_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    token_address: String,
) -> Result<Response, ContractError> {
    let mut token_auction_state =
        fetch_latest_auction_state_for_token(deps.storage, &token_id, &token_address)?;

    ensure!(
        !token_auction_state.is_cancelled,
        ContractError::AuctionCancelled {}
    );

    ensure!(
        token_auction_state.start.is_expired(&env.block),
        ContractError::AuctionNotStarted {}
    );
    ensure!(
        !token_auction_state.end.is_expired(&env.block),
        ContractError::AuctionEnded {}
    );

    ensure!(
        token_auction_state.owner != info.sender,
        ContractError::TokenOwnerCannotBid {}
    );

    ensure!(
        info.funds.len() == 1,
        ContractError::InvalidFunds {
            msg: "Auctions require exactly one coin to be sent.".to_string(),
        }
    );

    ensure!(
        token_auction_state.high_bidder_addr != info.sender,
        ContractError::HighestBidderCannotOutBid {}
    );

    let coin_denomination = token_auction_state.coin_denomination.clone();
    let payment: &Coin = &info.funds[0];
    ensure!(
        payment.denom == coin_denomination && payment.amount > Uint128::zero(),
        ContractError::InvalidFunds {
            msg: format!("No {} assets are provided to auction", coin_denomination),
        }
    );
    ensure!(
        token_auction_state.high_bidder_amount < payment.amount,
        ContractError::BidSmallerThanHighestBid {}
    );

    let mut messages: Vec<CosmosMsg> = vec![];
    // Send back previous bid unless there was no previous bid.
    if token_auction_state.high_bidder_amount > Uint128::zero() {
        let bank_msg = BankMsg::Send {
            to_address: token_auction_state.high_bidder_addr.to_string(),
            amount: coins(
                token_auction_state.high_bidder_amount.u128(),
                token_auction_state.coin_denomination.clone(),
            ),
        };
        messages.push(CosmosMsg::Bank(bank_msg));
    }

    token_auction_state.high_bidder_addr = info.sender.clone();
    token_auction_state.high_bidder_amount = payment.amount;

    let key = token_auction_state.auction_id.u128();
    NFT_AUCTION_STATE.save(deps.storage, key.clone(), &token_auction_state)?;
    let mut bids_for_auction = BIDS.load(deps.storage, key.clone())?;
    bids_for_auction.push(Bid {
        bidder: info.sender.to_string(),
        amount: payment.amount,
        timestamp: env.block.time,
    });
    BIDS.save(deps.storage, key, &bids_for_auction)?;
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "bid"),
        attr("token_id", token_id),
        attr("bider", info.sender.to_string()),
        attr("amount", payment.amount.to_string()),
    ]))
}

pub fn cancel_auction_and_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    token_address: String,
) -> Result<Response, ContractError> {
    let mut token_auction_state =
        fetch_latest_auction_state_for_token(deps.storage, &token_id, &token_address)?;
    ensure!(
        info.sender == token_auction_state.owner,
        ContractError::Unauthorized {}
    );
    ensure!(
        !token_auction_state.end.is_expired(&env.block),
        ContractError::AuctionEnded {}
    );
    let mut messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_auction_state.token_address.clone(),
        msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id,
        })?,
        funds: vec![],
    })];

    // Refund highest bid, if it exists.
    if !token_auction_state.high_bidder_amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: token_auction_state.high_bidder_addr.to_string(),
            amount: coins(
                token_auction_state.high_bidder_amount.u128(),
                token_auction_state.coin_denomination.clone(),
            ),
        }));
    }

    token_auction_state.is_cancelled = true;
    NFT_AUCTION_STATE.save(
        deps.storage,
        token_auction_state.auction_id.u128(),
        &token_auction_state,
    )?;

    Ok(Response::new().add_messages(messages))
}

pub fn finalize_auction_and_transfer_assets(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    token_id: String,
    token_address: String,
) -> Result<Response, ContractError> {
    let token_auction_state =
        fetch_latest_auction_state_for_token(deps.storage, &token_id, &token_address)?;
    ensure!(
        token_auction_state.end.is_expired(&env.block),
        ContractError::AuctionNotEnded {}
    );
    let token_owner = query_token_owner(
        deps.querier,
        token_auction_state.token_address.clone(),
        token_id.clone(),
    )?
    .owner;
    ensure!(
        token_owner == env.contract.address,
        ContractError::AuctionAlreadyClaimed {}
    );

    if token_auction_state.high_bidder_addr.to_string().is_empty()
        || token_auction_state.high_bidder_amount.is_zero()
    {
        return Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_auction_state.token_address.clone(),
                msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: token_auction_state.owner.clone(),
                    token_id: token_id.clone(),
                })?,
                funds: vec![],
            }))
            .add_attribute("action", "claim")
            .add_attribute("token_id", token_id)
            .add_attribute("token_contract", token_auction_state.token_address)
            .add_attribute("recipient", token_auction_state.owner)
            .add_attribute("winning_bid_amount", token_auction_state.high_bidder_amount)
            .add_attribute("auction_id", token_auction_state.auction_id));
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: token_auction_state.owner,
            amount: coins(
                token_auction_state.high_bidder_amount.u128(),
                token_auction_state.coin_denomination.clone(),
            ),
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_auction_state.token_address.clone(),
            msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: token_auction_state.high_bidder_addr.to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        }))
        .add_attribute("action", "claim")
        .add_attribute("token_id", token_id)
        .add_attribute("token_contract", token_auction_state.token_address)
        .add_attribute("recipient", &token_auction_state.high_bidder_addr)
        .add_attribute("winning_bid_amount", token_auction_state.high_bidder_amount)
        .add_attribute("auction_id", token_auction_state.auction_id))
}
