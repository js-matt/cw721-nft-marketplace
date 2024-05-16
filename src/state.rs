use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp;

const MAX_LIMIT: u64 = 70;
const DEFAULT_LIMIT: u64 = 20;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NFTAuctionState {
    pub start: Timestamp,
    pub end: Timestamp,
    pub high_bidder_addr: Addr,
    pub high_bidder_amount: Uint128,
    pub coin_denomination: String,
    pub auction_id: Uint128,
    pub min_bid: Option<Uint128>,
    pub owner: String,
    pub token_id: String,
    pub token_address: String,
    pub is_cancelled: bool,
}

#[cw_serde]
pub struct Bid {
    pub bidder: String,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

pub const NEXT_AUCTION_ID: Item<Uint128> = Item::new("next_auction_id");

pub const BIDS: Map<u128, Vec<Bid>> = Map::new("bids");

pub fn save_bids(storage: &mut dyn Storage, auction_id: u128, bid: Vec<Bid>) -> StdResult<()> {
    BIDS.save(storage, auction_id, &bid)?;
    Ok(())
}

pub fn load_bids(storage: &dyn Storage, auction_id: u128) -> StdResult<Vec<Bid>> {
    BIDS.load(storage, auction_id)
}

pub const NFT_AUCTION_STATE: Map<u128, NFTAuctionState> = Map::new("nft_auction_state");

pub fn save_nft_auction_state(
    storage: &mut dyn Storage,
    auction_id: u128,
    auction_state: NFTAuctionState,
) -> StdResult<()> {
    NFT_AUCTION_STATE.save(storage, auction_id, &auction_state)?;
    Ok(())
}

pub fn load_nft_auction_state(
    storage: &dyn Storage,
    auction_id: u128,
) -> StdResult<NFTAuctionState> {
    NFT_AUCTION_STATE.load(storage, auction_id)
}

pub fn save_next_auction_id(storage: &mut dyn Storage, auction_id: Uint128) -> StdResult<()> {
    NEXT_AUCTION_ID.save(storage, &auction_id)?;
    Ok(())
}

pub fn load_next_auction_id(storage: &dyn Storage) -> StdResult<Uint128> {
    NEXT_AUCTION_ID.load(storage)
}

#[cw_serde]
pub enum OrderBy {
    Asc,
    Desc,
}

#[cw_serde]
#[derive(Default)]
pub struct AuctionDetails {
    pub auction_ids: Vec<Uint128>,
    pub token_address: String,
    pub token_id: String,
}

impl AuctionDetails {
    pub fn latest(&self) -> Option<&Uint128> {
        self.auction_ids.last()
    }

    pub fn push(&mut self, e: Uint128) {
        self.auction_ids.push(e)
    }
}
impl<'a> IndexList<AuctionDetails> for AuctionIdIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AuctionDetails>> + '_> {
        let v: Vec<&dyn Index<AuctionDetails>> = vec![&self.token];
        Box::new(v.into_iter())
    }
}

pub struct AuctionIdIndices<'a> {
    pub token: MultiIndex<'a, String, AuctionDetails, String>,
}

pub fn auction_details<'a>() -> IndexedMap<'a, &'a str, AuctionDetails, AuctionIdIndices<'a>> {
    let indexes = AuctionIdIndices {
        token: MultiIndex::new(
            |_pk: &[u8], r| r.token_address.clone(),
            "ownership",
            "token_index",
        ),
    };
    IndexedMap::new("ownership", indexes)
}

pub fn save_auction_details(
    storage: &mut dyn Storage,
    pk: String,
    details: AuctionDetails,
) -> StdResult<()> {
    auction_details().save(storage, &pk, &details)?;
    Ok(())
}

pub fn load_auction_details(
    storage: &mut dyn Storage,
    pk: &std::string::String,
) -> StdResult<AuctionDetails> {
    auction_details().load(storage, pk)
}

pub fn get_bids(
    storage: &dyn Storage,
    auction_id: u128,
    start_after: Option<u64>,
    limit: Option<u64>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<Bid>> {
    let mut bids = load_bids(storage, auction_id)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = match start_after {
        None => 0,
        Some(x) => (x as usize) + 1usize,
    };

    let (start, end, order_by) = match order_by {
        Some(OrderBy::Desc) => (
            bids.len() - cmp::min(bids.len(), start + limit),
            bids.len() - cmp::min(start, bids.len()),
            OrderBy::Desc,
        ),
        _ => (
            cmp::min(bids.len(), start),
            cmp::min(start + limit, bids.len()),
            OrderBy::Asc,
        ),
    };

    let slice = &mut bids[start..end];
    if order_by == OrderBy::Desc {
        slice.reverse();
    }

    Ok(slice.to_vec())
}

pub fn read_auction_details(
    storage: &dyn Storage,
    token_address: Option<String>,
    start_after: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<AuctionDetails>, ContractError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let keys: Vec<String> = match token_address {
        Some(val) => auction_details()
            .idx
            .token
            .prefix(val)
            .keys(storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<Result<Vec<String>, _>>()?,
        None => auction_details()
            .idx
            .token
            .keys(storage, None, None, Order::Ascending)
            .take(limit)
            .collect::<Result<Vec<String>, _>>()?,
    };
    let mut res: Vec<AuctionDetails> = vec![];
    for key in keys.iter() {
        res.push(auction_details().load(storage, key)?);
    }
    Ok(res)
}
