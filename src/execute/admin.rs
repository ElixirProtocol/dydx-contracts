use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint128};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::DydxMsg;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::state::{LP_TOKENS, VAULTS_BY_PERP_ID, WITHDRAWAL_QUEUES};
use crate::{error::ContractError, state::STATE};

use super::helpers::{validate_addr_string, verify_sender_is_trader};
use super::USDC_DENOM;

/// Set the permissioned trader.
/// Can only be called by the current trader.
pub fn set_trader(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    new_trader: String,
) -> ContractResult<Response<DydxMsg>> {
    let mut state = STATE.load(deps.storage)?;
    let old_trader_addr = &state.trader;

    verify_sender_is_trader(&info.sender, &state.trader)?;
    let new_trader_addr = validate_addr_string(&deps, new_trader.clone())?;

    let event = Event::new("new_trader")
        .add_attribute("old", old_trader_addr.to_string())
        .add_attribute("new", new_trader);

    state.trader = new_trader_addr;
    STATE.save(deps.storage, &state)?;

    let resp = Response::new()
        .add_event(event)
        .add_attribute("method", "set_trader");

    Ok(resp)
}

/// Creates a vault and the associated dYdX subaccount required for trading.
/// Also creates an LP token and withdrawal queue for the vault.
/// Vaults are unique for a dYdX perp market and as such use `perp_id` as their identifier throughout the contract.
pub fn create_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;
    verify_sender_is_trader(&info.sender, &state.trader)?;

    if VAULTS_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultAlreadyInitialized { perp_id });
    }

    // save new vault
    VAULTS_BY_PERP_ID.save(deps.storage, perp_id, &true)?;
    WITHDRAWAL_QUEUES.save(deps.storage, perp_id, &Vec::with_capacity(10))?;

    // create LP token using cw20-base format
    let data = TokenInfo {
        name: format!("Elixir LP Token: dYdX-{perp_id}"),
        symbol: format!("ELXR-LP-dYdX-{perp_id}"),
        decimals: USDC_DENOM as u8,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    LP_TOKENS.save(deps.storage, perp_id, &data)?;

    let event = Event::new("new_vault")
        .add_attribute("perp_id", perp_id.to_string())
        .add_attribute("lp_name", format!("Elixir LP Token: dYdX-{perp_id}"))
        .add_attribute("lp_symbol", format!("ELXR-LP-dYdX-{perp_id}"));

    Ok(Response::new()
        .add_event(event)
        .add_attribute("method", "create_vault"))
}

/// Changes the vault fee. For now this is method is unused and will throw an error if called.
pub fn modify_vault_fee(
    _deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
    _perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    return Err(ContractError::NotImplemented {});
}

/// Changes the vault fee. For now this is method is unused and will throw an error if called.
pub fn collect_fees_from_vault(
    _deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
    _perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    return Err(ContractError::NotImplemented {});
}
