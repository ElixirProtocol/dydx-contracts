use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw20_base::msg::MigrateMsg;
use dydx::{msg::DydxMsg, query::DydxQueryWrapper};
use error::ContractResult;
use msg::{ExecuteMsg, InstantiateMsg};

pub mod contract;
pub mod dydx;
pub mod error;
pub mod execute;
pub mod msg;
pub mod query;
pub mod state;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<DydxMsg>> {
    contract::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<DydxQueryWrapper>, env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<DydxMsg>> {
    contract::execute(deps, env, info, msg)
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }
    
    // set the new version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    // see https://medium.com/cosmwasm/cosmwasm-for-ctos-ii-advanced-usage-ee04ce95d1d0 for migration details
    // and note that migrate is called on the new version of the code
    // the smart contract should:
    // 1. copy existing vaults
    // 2. copy LP token state
    // 3. copy withdrawal queues

    // since the smart contract address is the same, migration of funds in dYdX subaccounts is not necessary
    
    Ok(Response::default())
}
