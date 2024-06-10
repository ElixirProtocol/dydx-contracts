use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::{
    error::{ContractError, ContractResult},
    state::{ADMIN_ADDRS, STATE},
};

pub fn add_admins(
    deps: DepsMut,
    info: MessageInfo,
    new_admins: Vec<String>,
) -> ContractResult<Response> {
    let mut curr_admins = ADMIN_ADDRS.load(deps.storage)?;
    if !curr_admins.contains(&info.sender) {
        return Err(ContractError::SenderIsNotAdmin {
            sender: info.sender,
        });
    }

    let new_admins: Vec<Addr> = new_admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr).unwrap())
        .collect();

    let mut events = Vec::with_capacity(new_admins.len());
    for new_admin in new_admins {
        if curr_admins.insert(new_admin.clone()) {
            events.push(Event::new("admin_added").add_attribute("addr", new_admin))
        }
    }
    let added_count = events.len();

    ADMIN_ADDRS.save(deps.storage, &curr_admins)?;

    let resp = Response::new()
        .add_events(events)
        .add_attribute("action", "add_admins")
        .add_attribute("added_count", added_count.to_string());

    Ok(resp)
}

pub fn remove_admins(
    deps: DepsMut,
    info: MessageInfo,
    admins_to_remove: Vec<String>,
) -> ContractResult<Response> {
    let mut curr_admins = ADMIN_ADDRS.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if admins_to_remove.contains(&state.owner.to_string()) {
        return Err(ContractError::CannotRemoveContractDeployerAsAdmin);
    }

    if !curr_admins.contains(&info.sender) {
        return Err(ContractError::SenderIsNotAdmin {
            sender: info.sender,
        });
    }

    let admins_to_remove_addrs: Vec<Addr> = admins_to_remove
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr).unwrap())
        .collect();

    let mut events = Vec::with_capacity(admins_to_remove_addrs.len());
    for admin in admins_to_remove_addrs {
        if curr_admins.remove(&admin) {
            events.push(Event::new("admin_removed").add_attribute("addr", admin))
        }
    }
    let added_count = events.len();

    ADMIN_ADDRS.save(deps.storage, &curr_admins)?;

    let resp = Response::new()
        .add_events(events)
        .add_attribute("action", "remove_admins")
        .add_attribute("removed_count", added_count.to_string());

    Ok(resp)
}
