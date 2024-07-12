use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint128};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::DydxMsg;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::state::{
    VaultState, VaultStatus, LP_TOKENS, VAULT_STATES_BY_PERP_ID, WITHDRAWAL_QUEUES,
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
