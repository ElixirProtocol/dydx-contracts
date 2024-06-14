use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::{
    error::{ContractError, ContractResult},
    state::{STATE, TRADER_ADDRS},
};

pub fn add_traders(
    deps: DepsMut,
    info: MessageInfo,
    new_traders: Vec<String>,
) -> ContractResult<Response> {
    let mut curr_admins = TRADER_ADDRS.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::SenderIsNotAdmin {
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
        return Err(ContractError::SenderIsNotAdmin {
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
