use cosmwasm_std::{
    Addr, Decimal, DepsMut, Env, Event, Fraction, MessageInfo, Response, StdResult, Uint128
};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::{
    DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce,
};
use crate::dydx::proto_structs::SubaccountId;
use crate::dydx::querier::DydxQuerier;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::query::{lp_token_info, query_validated_dydx_position};
use crate::state::{VaultState, VaultStatus, LP_BALANCES, LP_TOKENS, VAULT_STATES_BY_PERP_ID};
use crate::{error::ContractError, state::STATE};

pub const USDC_ID: u32 = 0;
pub const USDC_DENOM: u32 = 6;
pub const USDC_COIN_TYPE: &str = "ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5";

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

    Ok(Response::new()
        .add_attribute("method", "create_vault"))
}

/// Allows a user to deposit into the market-making vault.
///
pub fn deposit_into_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let subaccount_id = get_contract_subaccount_id(&env, perp_id);
    let querier = DydxQuerier::new(&deps.querier);

    // assert that user is depositing USDC
    assert!(info.funds.len() == 1);    
    assert!(info.funds[0].denom == USDC_COIN_TYPE);
    let amount = info.funds[0].amount;

    // assert vault exists

    let vp = query_validated_dydx_position(&querier, &env, perp_id)?;
    let subaccount_value = vp.asset_usdc_value + vp.perp_usdc_value;
    let deposit_value = Decimal::from_atomics(amount, USDC_DENOM).unwrap();
    let lp_token_info = lp_token_info(deps.as_ref(), perp_id)?;

    // calculate the new deposit's share of total value
    // TODO: longer comment, explain math
    let share_value_fraction = deposit_value / (deposit_value + subaccount_value);
    let outstanding_tokens = Decimal::from_atomics(lp_token_info.total_supply, USDC_DENOM).unwrap();
    let new_tokens = if share_value_fraction == Decimal::one() {
        // TODO: fix large initial deposit bug
        Uint128::new(10 as u128).pow(USDC_DENOM)
    } else {
        let token_amt_decimal =
            (share_value_fraction * outstanding_tokens) / (Decimal::one() - share_value_fraction);
        token_amt_decimal.numerator()
            / Uint128::new(10 as u128).pow(Decimal::DECIMAL_PLACES - USDC_DENOM)
    };

    // mint tokens to depositor
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    do_mint(
        deps,
        sub_info,
        perp_id,
        info.sender.to_string(),
        new_tokens.into(),
    )
    .unwrap();

    let deposit = DydxMsg::DepositToSubaccount {
        recipient: subaccount_id.clone(),
        asset_id: USDC_ID,
        quantums: amount.u128() as u64, // TODO: make nicer
    };

    // TODO: more events, less debugging

    let event = Event::new("token_mint_math")
        .add_attribute("subaccount_value", subaccount_value.to_string())
        .add_attribute("deposit_value", deposit_value.to_string())
        .add_attribute("share_value_fraction", share_value_fraction.to_string())
        .add_attribute("outstanding_tokens", outstanding_tokens.to_string())
        .add_attribute("new_tokens", new_tokens.to_string());

    Ok(Response::new()
        .add_attribute("method", "deposit_into_vault")
        .add_event(event)
        .add_message(deposit))
}

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
    condition_type: OrderConditionType,
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

    // // validate order
    // if order.order_id.subaccount_id.owner != env.contract.address {
    //     return Err(ContractError::InvalidOrderIdSubaccountOwner);
    // }

    // let event = order.get_place_event();
    let place_order = DydxMsg::PlaceOrder {
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
        condition_type,
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

    // // validate cancelling a smart-contract owned order
    // if order_id.subaccount_id.owner != env.contract.address {
    //     return Err(ContractError::InvalidOrderIdSubaccountOwner);
    // }

    // let event = order_id.get_cancel_event();
    let cancel_order = DydxMsg::CancelOrder {
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

fn verify_owner_or_trader(sender: &Addr, owner: &Addr, trader: &Addr) -> ContractResult<()> {
    if sender != owner && sender != trader {
        return Err(ContractError::SenderCannotModifyTrader {
            sender: sender.clone(),
        });
    } else {
        Ok(())
    }
}

fn validate_addr_string(
    deps: &DepsMut<DydxQueryWrapper>,
    addr_string: String,
) -> ContractResult<Addr> {
    match deps.api.addr_validate(&addr_string) {
        Ok(a) => Ok(a),
        Err(_) => return Err(ContractError::InvalidAddress { addr: addr_string }),
    }
}

fn get_contract_subaccount_id(env: &Env, perp_id: u32) -> SubaccountId {
    SubaccountId {
        owner: env.contract.address.to_string(),
        number: perp_id,
    }
}

fn do_mint(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    perp_id: u32,
    recipient: String,
    amount: Uint128,
) -> ContractResult<()> {
    let mut config = LP_TOKENS
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::Unauthorized {})?;

    if config
        .mint
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        .minter
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    LP_TOKENS.save(deps.storage, perp_id, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &rcpt_addr),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(())
}
