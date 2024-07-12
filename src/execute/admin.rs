use cosmwasm_std::{
    Addr, CheckedMultiplyFractionError, Decimal, DepsMut, Env, Event, Fraction, MessageInfo,
    Response, StdResult, Uint128,
};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce};
use crate::dydx::proto_structs::{OrderBatch, SubaccountId};
use crate::dydx::querier::DydxQuerier;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::msg::TokenInfoResponse;
use crate::query::{lp_token_info, query_validated_dydx_position};
use crate::state::{
    VaultState, VaultStatus, Withdrawal, LP_BALANCES, LP_TOKENS, VAULT_STATES_BY_PERP_ID,
    WITHDRAWAL_QUEUES,
};
use crate::{error::ContractError, state::STATE};

use super::helpers::{get_contract_subaccount_id, validate_addr_string, verify_owner_or_trader};
use super::USDC_DENOM;



pub fn set_trader(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    new_trader: String,
) -> ContractResult<Response<DydxMsg>> {
    let mut state = STATE.load(deps.storage)?;
    let old_trader_addr = &state.trader;

    verify_owner_or_trader(&info.sender, &state.owner, &state.trader)?;
    let new_trader_addr = validate_addr_string(&deps, new_trader.clone())?;

    // new trader must not be old trader
    if new_trader_addr == info.sender {
        return Err(ContractError::NewTraderMustNotBeCurrentTrader);
    }

    let event = Event::new("trader_set")
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
pub fn create_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;
    verify_owner_or_trader(&info.sender, &state.owner, &state.trader)?;

    if VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultAlreadyInitialized { perp_id });
    }

    let subaccount_id = get_contract_subaccount_id(&env, perp_id);

    let vault_state = VaultState {
        subaccount_id: subaccount_id.clone(),
        status: VaultStatus::Open,
    };

    // save new vault
    VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;
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

    // TODO: more events

    Ok(Response::new().add_attribute("method", "create_vault"))
}

/// Freezes the vault (prevents placing any orders). For now, deposits/withdrawals and cancelling orders are allowed.
/// This can only be done by the current trader.
pub fn freeze_vault(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }

    let mut vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    let trader_addr = deps.api.addr_validate(&vault_state.subaccount_id.owner)?;
    // sender must be current trader
    if trader_addr != info.sender {
        return Err(ContractError::SenderCannotFreezeVault {
            sender: info.sender,
        });
    }

    match vault_state.status {
        VaultStatus::Open => {
            vault_state.status = VaultStatus::Frozen;
            VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;
        }
        VaultStatus::Frozen => return Err(ContractError::VaultAlreadyFrozen { perp_id }),
    }

    let event = Event::new("vault_frozen").add_attribute("id", perp_id.to_string());

    Ok(Response::new()
        .add_attribute("method", "freeze_vault")
        .add_event(event))
}

/// Thaws the vault (allow placing orders).
/// This can only be done by the current trader.
pub fn thaw_vault(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }

    let mut vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    let trader_addr = deps.api.addr_validate(&vault_state.subaccount_id.owner)?;
    // sender must be current trader
    if trader_addr != info.sender {
        return Err(ContractError::SenderCannotThawVault {
            sender: info.sender,
        });
    }

    match vault_state.status {
        VaultStatus::Open => return Err(ContractError::VaultAlreadyOpen { perp_id }),
        VaultStatus::Frozen => {
            vault_state.status = VaultStatus::Open;
            VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;
        }
    }

    let event = Event::new("vault_thawed").add_attribute("id", perp_id.to_string());

    Ok(Response::new()
        .add_attribute("method", "thaw_vault")
        .add_event(event))
}