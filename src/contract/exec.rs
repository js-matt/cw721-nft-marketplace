use crate::{
    contract::helper::{
        fetch_and_update_next_auction_id, fetch_latest_auction_state_for_token, query_token_owner,
    },
    error::ContractError,
    msg::Cw721CustomMsg,
    state::{
        load_auction_details, load_bids, save_auction_details, save_bids, save_nft_auction_state,
        Bid, NFTAuctionState,
    },
};
use cosmwasm_std::{
    attr, coins, ensure, from_json, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env,
    MessageInfo, Response, Timestamp, Uint128, WasmMsg,
};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

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
        ContractError::InValidTime {}
    );
    let end_timestamp = Timestamp::from_seconds(start_time + duration);
    let start_timestamp = Timestamp::from_seconds(start_time);

    ensure!(
        start_timestamp.gt(&env.block.time),
        ContractError::InvalidStartTime {}
    );

    let auction_id = fetch_and_update_next_auction_id(deps.storage)?;
    let pk = token_id.to_owned() + &token_address;

    let mut auction_info = load_auction_details(deps.storage, &pk).unwrap_or_default();
    auction_info.push(auction_id);
    if auction_info.token_address.is_empty() {
        auction_info.token_address = token_address.to_owned();
        auction_info.token_id = token_id.to_owned();
    }

    save_auction_details(deps.storage, pk, auction_info)?;
    save_bids(deps.storage, auction_id.u128(), vec![])?;

    save_nft_auction_state(
        deps.storage,
        auction_id.u128(),
        NFTAuctionState {
            start: start_timestamp,
            end: end_timestamp,
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
        attr("start_time", start_timestamp.to_string()),
        attr("end_time", end_timestamp.to_string()),
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
        token_auction_state.start.gt(&env.block.time),
        ContractError::AuctionNotStarted {}
    );
    ensure!(
        !token_auction_state.end.ge(&env.block.time),
        ContractError::AuctionEnded {}
    );

    ensure!(
        token_auction_state.owner != info.sender,
        ContractError::TokenOwnerCannotBid {}
    );

    ensure!(
        info.funds.len() == 1,
        ContractError::InvalidFunds {
            msg: "Auctions require you to send exactly one coin".to_string(),
        }
    );

    ensure!(
        token_auction_state.high_bidder_addr != info.sender,
        ContractError::HighestBidderCannotBeOutbid {}
    );

    let coin_denomination = token_auction_state.coin_denomination.clone();
    let payment: &Coin = &info.funds[0];
    ensure!(
        payment.denom == coin_denomination && payment.amount > Uint128::zero(),
        ContractError::InvalidFunds {
            msg: format!(
                "No {} assets are provided for the auction",
                coin_denomination
            ),
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
    save_nft_auction_state(deps.storage, key.clone(), token_auction_state)?;
    let mut bids_for_auction = load_bids(deps.storage, key.clone())?;
    bids_for_auction.push(Bid {
        bidder: info.sender.to_string(),
        amount: payment.amount,
        timestamp: env.block.time,
    });
    save_bids(deps.storage, key, bids_for_auction)?;
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
        !token_auction_state.end.gt(&env.block.time),
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
    save_nft_auction_state(
        deps.storage,
        token_auction_state.auction_id.u128(),
        token_auction_state,
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
        token_auction_state.end.gt(&env.block.time),
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
