use core::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Response};

use super::USDC_DENOM;
use crate::dydx::msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce};
use crate::dydx::querier::DydxQuerier;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::query::query_validated_dydx_position;
use crate::state::VAULTS_BY_PERP_ID;
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

impl NewOrder {
    pub fn get_place_event(&self, perp_id: u32, clob_pair_id: u32) -> Event {
        Event::new("placed_order")
            .add_attribute("perp_id", perp_id.to_string())
            .add_attribute("client_id", self.client_id.to_string())
            .add_attribute("clob_pair_id", clob_pair_id.to_string())
            .add_attribute("side", self.side.to_string())
            .add_attribute("quantums", self.quantums.to_string())
            .add_attribute("subticks", self.subticks.to_string())
    }
}

impl fmt::Display for NewOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NewOrder {{ client_id: {}, side: {:?}, quantums: {}, subticks: {}, good_til_block_time: {}, time_in_force: {:?}, reduce_only: {}, client_metadata: {}, conditional_order_trigger_subticks: {} }}",
            self.client_id,
            self.side,
            self.quantums,
            self.subticks,
            self.good_til_block_time,
            self.time_in_force,
            self.reduce_only,
            self.client_metadata,
            self.conditional_order_trigger_subticks
        )
    }
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
    let vp = query_validated_dydx_position(deps.as_ref(), perp_id)?;
    let mut asset_value = vp.asset_usdc_value.clone();
    let mut perp_value = vp.perp_usdc_value.clone();

    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderIsNotTrader {
            sender: info.sender,
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
    if !VAULTS_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }

    let mut messages = vec![];
    let mut events = vec![];

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
            let cancel_event = Event::new("cancelled_order")
                .add_attribute("perp_id", subaccount_number.to_string())
                .add_attribute("client_id", cancel_client_id.to_string())
                .add_attribute("clob_pair_id", clob_pair_id.to_string())
                .add_attribute("cancel_good_til_block", cancel_good_til_block.to_string());
            messages.push(cancel_msg);
            events.push(cancel_event);
        }
    }

    let mut num_bids = 0;
    let mut num_asks = 0;
    // then add new orders
    if new_orders.len() > 0 {
        for new_order in new_orders {
            let order_value = Decimal::from_atomics(new_order.quantums, USDC_DENOM).unwrap();
            if order_value > asset_value || asset_value < perp_value + (order_value + order_value) {
                return Err(ContractError::NewOrderWouldIncreaseLeverageTooMuch {
                    perp_id,
                    new_order,
                });
            }

            match new_order.side {
                OrderSide::Unspecified => (),
                OrderSide::Buy => num_bids += 1,
                OrderSide::Sell => num_asks += 1,
            };

            asset_value -= order_value;
            perp_value += order_value;

            let place_event = new_order.get_place_event(subaccount_number, clob_pair_id);
            let place_msg = DydxMsg::PlaceOrderV1 {
                subaccount_number,
                client_id: new_order.client_id,
                order_flags: LONG_TERM_ORDER_FLAGS,
                clob_pair_id,
                side: new_order.side.clone(),
                quantums: new_order.quantums,
                subticks: new_order.subticks,
                good_til_block_time: new_order.good_til_block_time,
                time_in_force: new_order.time_in_force,
                reduce_only: new_order.reduce_only,
                client_metadata: new_order.client_metadata,
                condition_type: OrderConditionType::Unspecified,
                conditional_order_trigger_subticks: new_order.conditional_order_trigger_subticks,
            };
            messages.push(place_msg);
            events.push(place_event);
        }
    }

    // validate at most 3 orders per side
    if num_bids > 3 || num_asks > 3 {
        return Err(ContractError::CanOnlyPlaceThreeOrdersPerSide {});
    }

    Ok(Response::new()
        .add_attribute("method", "market_make")
        .add_events(events)
        .add_messages(messages))
}
