use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::dydx::msg::{DydxMsg, Order};
use crate::dydx::proto_structs::SubaccountId;
use crate::state::{Trader, VaultState, VaultStatus, VAULT_STATES_BY_PERP_ID};
use crate::{
    error::{ContractError, ContractResult},
    state::{STATE, TRADERS},
};

pub fn add_traders(
    deps: DepsMut,
    info: MessageInfo,
    new_traders: Vec<String>,
) -> ContractResult<Response> {
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
            TRADERS.save(deps.storage, &new_trader, &Trader { num_markets: 0 })?;
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
    deps: DepsMut,
    info: MessageInfo,
    traders_to_remove: Vec<String>,
) -> ContractResult<Response> {
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
pub fn create_vault(deps: DepsMut, info: MessageInfo, perp_id: u32) -> ContractResult<Response> {
    const AMOUNT: u64 = 1;
    const USDC_ID: u32 = 1;

    if !TRADERS.has(deps.storage, &info.sender) {
        return Err(ContractError::SenderCannotCreateVault {
            sender: info.sender,
        });
    }

    if VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultAlreadyInitialized { perp_id });
    }

    // update # of markets on trader
    let mut trader = TRADERS.load(deps.storage, &info.sender)?;
    let subaccount_number = trader.num_markets.clone();

    trader.num_markets += 1;
    TRADERS.save(deps.storage, &info.sender, &trader)?;

    let subaccount_id = SubaccountId {
        owner: info.sender.to_string(),
        number: subaccount_number,
    };

    let vault_state = VaultState {
        subaccount_id: subaccount_id.clone(),
        status: VaultStatus::Open,
    };

    // save new vault
    VAULT_STATES_BY_PERP_ID.save(deps.storage, perp_id, &vault_state)?;

    // deposit smallest amount of USDC in dYdX contract to create account
    let _deposit = DydxMsg::DepositToSubaccount {
        sender: info.sender.to_string(),
        recipient: subaccount_id.clone(),
        asset_id: USDC_ID,
        quantums: AMOUNT,
    };

    // withdraw so that user deposits always start accounting from 0
    let _withdraw = DydxMsg::WithdrawFromSubaccount {
        sender: subaccount_id,
        recipient: info.sender.to_string(),
        asset_id: USDC_ID,
        quantums: AMOUNT,
    };

    // TODO: figure out dYdX calling convention
    // let x = WasmMsg::Execute { contract_addr: (), msg: (), funds: () } {};
    // or
    // Ok(Response::new().add_messages([ResponseMsg::Dydx(deposit), ResponseMsg::Dydx(withdraw)]))

    // TODO: more events

    Ok(Response::new().add_attribute("method", "create_vault"))
}

/// Places an order on dYdX.
/// Requires the sender to have trader permissions and an existing vault that corresponds to the dYdX market.
pub fn place_order(deps: DepsMut, info: MessageInfo, order: Order) -> ContractResult<Response> {
    // validate market
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, order.order_id.clob_pair_id) {
        return Err(ContractError::InvalidMarket {
            perp_id: order.order_id.clob_pair_id,
        });
    }

    // validate sender (must be configured trader)
    let vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, order.order_id.clob_pair_id)?;
    if vault_state.subaccount_id.owner != info.sender.to_string() {
        return Err(ContractError::SenderCannotPlaceTrade {
            sender: info.sender,
            expected: vault_state.subaccount_id.owner,
            perp_id: order.order_id.clob_pair_id,
        });
    }

    let events = vec![order.get_place_event()];
    let _place_order = DydxMsg::PlaceOrder { order };

    // TODO: figure out dYdX calling convention
    // let x = WasmMsg::Execute { contract_addr: (), msg: (), funds: () } {};
    // or
    // Ok(Response::new().add_messages([ResponseMsg::Dydx(deposit), ResponseMsg::Dydx(withdraw)]))

    Ok(Response::new()
        .add_events(events)
        .add_attribute("method", "place_order"))
}
