use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::dydx::msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce};
use crate::dydx::querier::DydxQuerier;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::state::{VaultStatus, VAULT_STATES_BY_PERP_ID};
use crate::{error::ContractError, state::STATE};

#[cw_serde]
pub struct NewOrder {
    pub client_id: u32,
    pub side: OrderSide,
    pub quantums: u64,
    pub subticks: u64,
    pub good_til_block_time: u32,
    pub time_in_force: OrderTimeInForce,
    pub reduce_only: bool,
    pub client_metadata: u32,
    pub conditional_order_trigger_subticks: u64,
}

pub const LONG_TERM_ORDER_FLAGS: u32 = 64;

/// Batch cancels and places up to 3 bids and 3 asks on dYdX.
/// Requires the sender to be the trader and the orders to be placed in an existing vault.
/// This entrypoint will only send messages passed in as arguments. This means that it can be used selectively to only place or cancel orders.
pub fn market_make(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    info: MessageInfo,
    subaccount_number: u32,
    clob_pair_id: u32,
    new_orders: Vec<NewOrder>,
    cancel_client_ids: Vec<u32>,
    cancel_good_til_block: u32,
) -> ContractResult<Response<DydxMsg>> {
    let perp_id = subaccount_number;

    let state = STATE.load(deps.storage)?;

    let querier = DydxQuerier::new(&deps.querier);
    let perp_details = querier.query_perpetual_clob_details(perp_id)?;

    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderIsNotTrader {
            addr: info.sender.to_string(),
        });
    }

    // validate clob id corresponds to perp market
    if perp_details.perpetual_clob_details.clob_pair.id != clob_pair_id {
        return Err(ContractError::PerpMarketClobIdMismatch {
            queried_id: perp_details.perpetual_clob_details.clob_pair.id,
            supplied_id: clob_pair_id,
            perp_id,
        });
    }

    // validate vault
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }
    let vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    if vault_state.status != VaultStatus::Open {
        return Err(ContractError::VaultIsNotOpen { perp_id });
    }

    let mut response = Response::new().add_attribute("method", "market_make");

    // first add batch cancel
    if cancel_client_ids.len() > 0 {
        for cancel_client_id in cancel_client_ids {
            // let event = order_id.get_batch_cancel_event();
            let cancel_msg = DydxMsg::CancelOrderV1 {
                subaccount_number,
                client_id: cancel_client_id,
                order_flags: LONG_TERM_ORDER_FLAGS,
                clob_pair_id,
                good_til_block_time: cancel_good_til_block,
            };
            response = response // .add_event(event)
                .add_message(cancel_msg);
        }
    }

    // then add new orders
    if new_orders.len() > 0 {
        for new_order in new_orders {
            // let event = order.get_place_event();
            let place_msg = DydxMsg::PlaceOrderV1 {
                subaccount_number,
                client_id: new_order.client_id,
                order_flags: LONG_TERM_ORDER_FLAGS,
                clob_pair_id,
                side: new_order.side,
                quantums: new_order.quantums,
                subticks: new_order.subticks,
                good_til_block_time: new_order.good_til_block_time,
                time_in_force: new_order.time_in_force,
                reduce_only: new_order.reduce_only,
                client_metadata: new_order.client_metadata,
                condition_type: OrderConditionType::Unspecified,
                conditional_order_trigger_subticks: new_order.conditional_order_trigger_subticks,
            };
            response = response // .add_event(event)
                .add_message(place_msg);
        }
    }
    Ok(response)
}
