use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::dydx::msg::DydxMsg;
use crate::dydx::proto_structs::SubaccountId;
use crate::state::VAULT_SUBACCOUNTS_BY_PERP_ID;
use crate::{
    error::{ContractError, ContractResult}, state::{NUM_VAULTS, STATE, TRADER_ADDRS}
};

pub fn add_traders(
    deps: DepsMut,
    info: MessageInfo,
    new_traders: Vec<String>,
) -> ContractResult<Response> {
    let mut curr_admins = TRADER_ADDRS.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::SenderIsNotOwner {
            sender: info.sender,
        });
    }

    let new_traders: Vec<Addr> = new_traders
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr).unwrap())
        .collect();

    let mut events = Vec::with_capacity(new_traders.len());
    for new_trader in new_traders {
        if curr_admins.insert(new_trader.clone()) {
            events.push(Event::new("trader_added").add_attribute("addr", new_trader))
        }
    }
    let added_count = events.len();

    TRADER_ADDRS.save(deps.storage, &curr_admins)?;

    let resp = Response::new()
        .add_events(events)
        .add_attribute("action", "add_traders")
        .add_attribute("added_count", added_count.to_string());

    Ok(resp)
}

pub fn remove_traders(
    deps: DepsMut,
    info: MessageInfo,
    traders_to_remove: Vec<String>,
) -> ContractResult<Response> {
    let mut curr_admins = TRADER_ADDRS.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if traders_to_remove.contains(&state.owner.to_string()) {
        return Err(ContractError::CannotRemoveContractDeployerAsTrader);
    }

    if info.sender != state.owner {
        return Err(ContractError::SenderIsNotOwner {
            sender: info.sender,
        });
    }

    let traders_to_remove_addrs: Vec<Addr> = traders_to_remove
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr).unwrap())
        .collect();

    let mut events = Vec::with_capacity(traders_to_remove_addrs.len());
    for admin in traders_to_remove_addrs {
        if curr_admins.remove(&admin) {
            events.push(Event::new("trader_removed").add_attribute("addr", admin))
        }
    }
    let added_count = events.len();

    TRADER_ADDRS.save(deps.storage, &curr_admins)?;

    let resp = Response::new()
        .add_events(events)
        .add_attribute("action", "remove_traders")
        .add_attribute("removed_count", added_count.to_string());

    Ok(resp)
}

/// Creates a vault and the associated dYdX subaccount required for trading.
/// 
pub fn create_vault(
    deps: DepsMut,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response> {
    let state = STATE.load(deps.storage)?;
    let num_vaults = NUM_VAULTS.load(deps.storage)?;
    const AMOUNT: u64 = 1;
    let asset_id = 0; // TODO: USDC

    if info.sender != state.owner {
        return Err(ContractError::SenderIsNotOwner {
            sender: info.sender,
        });
    }

    if VAULT_SUBACCOUNTS_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultAlreadyInitialized { perp_id });
    } 

    let subaccount_id = SubaccountId {
        owner: state.owner.to_string(),
        number: num_vaults
    };

    // update contract state
    NUM_VAULTS.save(deps.storage, &(num_vaults + 1))?;
    VAULT_SUBACCOUNTS_BY_PERP_ID.save(deps.storage, perp_id, &subaccount_id)?;

    // deposit smallest amount of USDC in dYdX contract to create account
    let _deposit = DydxMsg::DepositToSubaccount {
        sender: info.sender.to_string(),
        recipient: subaccount_id.clone(),
        asset_id,
        quantums: AMOUNT,
    };

    // withdraw so that user deposits always start accounting from 0
    let _withdraw = DydxMsg::WithdrawFromSubaccount { 
        sender: subaccount_id, 
        recipient: state.owner.to_string(), 
        asset_id, 
        quantums: AMOUNT 
    };

    // TODO: figure out dYdX calling convention
    // let x = WasmMsg::Execute { contract_addr: (), msg: (), funds: () } {};
    // or
    // Ok(Response::new().add_messages([ResponseMsg::Dydx(deposit), ResponseMsg::Dydx(withdraw)]))

    Ok(Response::new()
    .add_attribute("method", "create_vault"))
}