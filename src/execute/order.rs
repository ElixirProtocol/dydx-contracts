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


/// Places an order on dYdX.
/// Requires the sender to be the trader and the order to be placed in an existing vault.
pub fn place_order(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    info: MessageInfo,
    subaccount_number: u32,
    client_id: u32,
    order_flags: u32,
    clob_pair_id: u32,
    side: OrderSide,
    quantums: u64,
    subticks: u64,
    good_til_block_time: u32,
    time_in_force: OrderTimeInForce,
    reduce_only: bool,
    client_metadata: u32,
    conditional_order_trigger_subticks: u64,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;

    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderIsNotTrader {
            addr: info.sender.to_string(),
        });
    }

    let perp_id = subaccount_number;
    // validate vault
    if !VAULT_STATES_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }
    let vault_state = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    if vault_state.status != VaultStatus::Open {
        return Err(ContractError::VaultIsNotOpen { perp_id });
    }

    // let event = order.get_place_event();
    let place_order = DydxMsg::PlaceOrderV1 {
        subaccount_number,
        client_id,
        order_flags,
        clob_pair_id,
        side,
        quantums,
        subticks,
        good_til_block_time,
        time_in_force,
        reduce_only,
        client_metadata,
        condition_type: OrderConditionType::Unspecified,
        conditional_order_trigger_subticks,
    };

    Ok(Response::new()
        .add_attribute("method", "place_order")
        // .add_event(event)
        .add_message(place_order))
}

/// Cancels a dYdX order.
/// Requires the sender to be the trader and the order to have been placed by this smart contract in an existing vault.
pub fn cancel_order(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    info: MessageInfo,
    subaccount_number: u32,
    client_id: u32,
    order_flags: u32,
    clob_pair_id: u32,
    good_til_block_time: u32,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;

    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderIsNotTrader {
            addr: info.sender.to_string(),
        });
    }

    // let event = order_id.get_cancel_event();
    let cancel_order = DydxMsg::CancelOrderV1 {
        subaccount_number,
        client_id,
        order_flags,
        clob_pair_id,
        good_til_block_time,
    };

    Ok(Response::new()
        .add_attribute("method", "cancel_order")
        // .add_event(event)
        .add_message(cancel_order))
}

/// Cancels a batch of dYdX orders.
/// Requires the sender to be the trader.
/// Cancels will be performed optimistically even if some cancels are invalid or fail.
pub fn batch_cancel(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    info: MessageInfo,
    subaccount_number: u32,
    order_batches: Vec<OrderBatch>,
    good_til_block: u32,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;

    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderIsNotTrader {
            addr: info.sender.to_string(),
        });
    }

    // let event = order_id.get_batch_cancel_event();
    let batch_cancel = DydxMsg::BatchCancelV1 {
        subaccount_number: subaccount_number,
        short_term_cancels: order_batches,
        good_til_block,
    };

    Ok(Response::new()
        .add_attribute("method", "batch_cancel")
        // .add_event(event)
        .add_message(batch_cancel))
}