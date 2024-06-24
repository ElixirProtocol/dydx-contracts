use cosmwasm_std::{to_json_binary, Addr, DepsMut, Env, Event, MessageInfo, Response, WasmMsg};

use crate::dydx::msg::{DydxMsg, Order};
use crate::dydx::proto_structs::SubaccountId;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::state::{
    Trader, VaultState, VaultStatus, DEFAULT_TRADER_CAPACITY, VAULT_STATES_BY_PERP_ID,
};
use crate::{
    error::ContractError,
    state::{STATE, TRADERS},
};

pub fn add_traders(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    new_traders: Vec<String>,
) -> ContractResult<Response<DydxMsg>> {
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
        if !TRADERS.has(deps.storage, &new_trader) {
            TRADERS.save(
                deps.storage,
                &new_trader,
                &Trader {
                    markets: Vec::with_capacity(DEFAULT_TRADER_CAPACITY),
                },
            )?;
            events.push(Event::new("trader_added").add_attribute("addr", new_trader))
        }
    }
    let added_count = events.len();

    let resp = Response::new()
        .add_events(events)
        .add_attribute("method", "add_traders")
        .add_attribute("added_count", added_count.to_string());

    Ok(resp)
}

pub fn remove_traders(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    traders_to_remove: Vec<String>,
) -> ContractResult<Response<DydxMsg>> {
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
        if TRADERS.has(deps.storage, &admin) {
            TRADERS.remove(deps.storage, &admin);
            events.push(Event::new("trader_removed").add_attribute("addr", admin))
        }
    }
    let added_count = events.len();

    let resp = Response::new()
        .add_events(events)
        .add_attribute("method", "remove_traders")
        .add_attribute("removed_count", added_count.to_string());

    Ok(resp)
}

/// Creates a vault and the associated dYdX subaccount required for trading.
/// The sender must be a pre-approved trader. The sender will become the only eligible trader for the market.
pub fn create_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    const AMOUNT: u64 = 1;
    const USDC_ID: u32 = 0;

    // TODO: fix
    if !TRADERS.has(deps.storage, &info.sender) {
        return Err(ContractError::SenderCannotCreateVault {
            sender: info.sender,
        });
    }

    if VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultAlreadyInitialized { perp_id });
    }


    let subaccount_id = SubaccountId {
        owner: env.contract.address.to_string(),
        number: perp_id,
    };

    let vault_state = VaultState {
        subaccount_id: subaccount_id.clone(),
        status: VaultStatus::Open,
    };

    // save new vault
    VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;

    // deposit smallest amount of USDC in dYdX contract to create account
    let deposit = DydxMsg::DepositToSubaccount {
        sender: info.sender.to_string(),
        recipient: subaccount_id.clone(),
        asset_id: USDC_ID,
        quantums: AMOUNT,
    };

    // withdraw so that user deposits always start accounting from 0
    let withdraw = DydxMsg::WithdrawFromSubaccount {
        sender: subaccount_id,
        recipient: info.sender.to_string(),
        asset_id: USDC_ID,
        quantums: AMOUNT,
    };

    // TODO: more events

    Ok(Response::new()
    .add_attribute("method", "create_vault")
    .add_message(deposit)
    .add_message(withdraw)
)
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

/// Changes the vault trader. Must be called by the current trader.
/// This function requires that the dYdX subaccount does not have any open orders
/// and that the vault is currently frozen. The vault will remain frozen after this function.
pub fn change_vault_trader(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
    new_trader_str: String,
) -> ContractResult<Response<DydxMsg>> {
    let new_trader_addr = deps.api.addr_validate(&new_trader_str)?;
    // check new trader can trade
    if !TRADERS.has(deps.storage, &new_trader_addr) {
        return Err(ContractError::NewVaultTraderMustBeApproved {
            new_trader: new_trader_addr,
            perp_id,
        });
    }

    // check vault initialized
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }

    let vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    let old_trader_addr = deps.api.addr_validate(&vault_state.subaccount_id.owner)?;
    // check sender is the current trader
    if old_trader_addr != info.sender {
        return Err(ContractError::SenderCannotChangeVaultTrader {
            sender: info.sender,
        });
    }
    // new trader must not be old trader
    if new_trader_addr == info.sender {
        return Err(ContractError::NewVaultTraderMustNotBeCurrentTrader { perp_id });
    }


    // TODO:
    // assert!(asset_positions only USDC
    // assert!(perpetual_positions.len() == 0) or there is nothing in there

    // TODO: fix
    // update the new trader
    let mut new_trader = TRADERS.load(deps.storage, &new_trader_addr)?;
    let subaccount_number = new_trader.markets.len() as u32;
    new_trader.markets.push(perp_id);
    TRADERS.save(deps.storage, &new_trader_addr, &new_trader)?;

    // update the vault
    let subaccount_id = SubaccountId {
        owner: env.contract.address.to_string(),
        number: subaccount_number,
    };
    let vault_state = VaultState {
        subaccount_id: subaccount_id.clone(),
        status: VaultStatus::Frozen,
    };
    VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;

    let event = Event::new("vault_trader_update")
        .add_attribute("old", old_trader_addr.to_string())
        .add_attribute("new", new_trader_str);

    Ok(Response::new()
        .add_attribute("method", "change_vault_trader")
        .add_event(event))
}

/// Places an order on dYdX.
/// Requires the sender to have trader permissions and an existing vault that corresponds to the dYdX market.
pub fn place_order(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    order: Order,
) -> ContractResult<Response<DydxMsg>> {
    // // validate market
    // if !VAULT_STATES_BY_PERP_ID.has(deps.storage, order.order_id.clob_pair_id) {
    //     return Err(ContractError::InvalidMarket {
    //         perp_id: order.order_id.clob_pair_id,
    //     });
    // }

    // // validate sender (must be configured trader)
    // let vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, order.order_id.clob_pair_id)?;
    // if vault_state.subaccount_id.owner != info.sender.to_string() {
    //     return Err(ContractError::SenderCannotPlaceTrade {
    //         sender: info.sender,
    //         expected: vault_state.subaccount_id.owner,
    //         perp_id: order.order_id.clob_pair_id,
    //     });
    // }

    // let events = vec![order.get_place_event()];
    // let _place_order = DydxMsg::PlaceOrder { order };

    // // TODO: figure out dYdX calling convention
    // // let x = WasmMsg::Execute { contract_addr: (), msg: (), funds: () } {};
    // // or
    // // Ok(Response::new().add_messages([ResponseMsg::Dydx(deposit), ResponseMsg::Dydx(withdraw)]))

    Ok(Response::new())
    // .add_events(events)
    // .add_attribute("method", "place_order"))
}

// /// Normal deposit
// pub fn a( 
//     _deps: DepsMut<DydxQueryWrapper>,
//     _env: Env,
//     info: MessageInfo,
//     perp_id: u32,
// ) -> ContractResult<Response<DydxMsg>> {
//     const AMOUNT: u64 = 1;
//     const USDC_ID: u32 = 0;

//     let subaccount_id = SubaccountId {
//         owner: info.sender.to_string(),
//         number: perp_id,
//     };

//     let deposit = DydxMsg::DepositToSubaccount {
//         sender: info.sender.to_string(),
//         recipient: subaccount_id.clone(),
//         asset_id: USDC_ID,
//         quantums: AMOUNT,
//     };


//     Ok(Response::new()
//     .add_attribute("method", "create_vault")
//     .add_message(deposit))
// }

// /// Smart contract deposit
// pub fn b( 
//     _deps: DepsMut<DydxQueryWrapper>,
//     env: Env,
//     info: MessageInfo,
//     perp_id: u32,
// ) -> ContractResult<Response<DydxMsg>> {
//     const AMOUNT: u64 = 1;
//     const USDC_ID: u32 = 0;

//     let subaccount_id = SubaccountId {
//         owner: env.contract.address.to_string(), // Why can't the contract have a subaccount?
//         number: perp_id,
//     };

//     let deposit = DydxMsg::DepositToSubaccount {
//         sender: info.sender.to_string(),
//         recipient: subaccount_id.clone(),
//         asset_id: USDC_ID,
//         quantums: AMOUNT,
//     };


//     Ok(Response::new()
//     .add_attribute("method", "create_vault")
//     .add_message(deposit))
// }

// ///  deposit
// pub fn c( 
//     _deps: DepsMut<DydxQueryWrapper>,
//     env: Env,
//     info: MessageInfo,
//     perp_id: u32,
// ) -> ContractResult<Response<DydxMsg>> {
//     const AMOUNT: u64 = 1;
//     const USDC_ID: u32 = 0;

//     let subaccount_id = SubaccountId {
//         owner: env.contract.address.to_string(),
//         number: perp_id,
//     };

//     let deposit = DydxMsg::DepositToSubaccount {
//         sender: info.sender.to_string(),
//         recipient: subaccount_id.clone(),
//         asset_id: USDC_ID,
//         quantums: AMOUNT,
//     };


//     Ok(Response::new()
//     .add_attribute("method", "deposit_example")
//     .add_message(deposit))
// }

// /// withdraw
// pub fn d( 
//     _deps: DepsMut<DydxQueryWrapper>,
//     env: Env,
//     info: MessageInfo,
//     perp_id: u32,
// ) -> ContractResult<Response<DydxMsg>> {
//     const AMOUNT: u64 = 1;
//     const USDC_ID: u32 = 0;

//     let subaccount_id = SubaccountId {
//         owner: env.contract.address.to_string(),
//         number: perp_id,
//     };

//     let withdraw = DydxMsg::WithdrawFromSubaccount {
//         sender: subaccount_id,
//         recipient: info.sender.to_string(),
//         asset_id: USDC_ID,
//         quantums: AMOUNT,
//     };

//     Ok(Response::new()
//     .add_attribute("method", "withdraw_example")
//     .add_message(withdraw))
// }