mod exec;
mod query;
mod helper;

use cosmwasm_std::{
  entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};

use {
  error::ContractError,
  msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
  state::NEXT_AUCTION_ID,
};

#[entry_point]
pub fn instantiate(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  NEXT_AUCTION_ID.save(deps.storage, &Uint128::from(1u128))?;
  Ok(Response::new())
}
