use cosmwasm_std::{
    Addr, Decimal, DepsMut, Env, Event, Fraction, MessageInfo, Response, StdResult, Uint128,
};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce};
use crate::dydx::proto_structs::SubaccountId;
use crate::dydx::querier::DydxQuerier;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::query::{lp_token_info, query_validated_dydx_position};
use crate::state::{VaultState, VaultStatus, LP_BALANCES, LP_TOKENS, VAULT_STATES_BY_PERP_ID};
use crate::{error::ContractError, state::STATE};

pub const USDC_ID: u32 = 0;
pub const USDC_DENOM: u32 = 6;
pub const USDC_COIN_TYPE: &str =
    "ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5";

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

    Ok(Response::new().add_attribute("method", "create_vault"))
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
    let amount = info.funds[0].amount;

    // assert that user is depositing only USDC with a non-zero amount
    if info.funds.len() != 1 {
        return Err(ContractError::CanOnlyDepositOneCointype {});
    }
    if info.funds[0].denom != USDC_COIN_TYPE {
        return Err(ContractError::InvalidCoin {
            coin_type: info.funds[0].denom.clone(),
        });
    }
    if amount <= Uint128::zero() {
        return Err(ContractError::InvalidDepositAmount {
            coin_type: info.funds[0].denom.clone(),
            amount: amount.into(),
        });
    }

    // assert vault exists

    let vp = query_validated_dydx_position(&querier, &env, perp_id)?;
    let subaccount_value = vp.asset_usdc_value + vp.perp_usdc_value;
    let deposit_value = Decimal::from_atomics(amount, USDC_DENOM).unwrap();
    let lp_token_info = lp_token_info(deps.as_ref(), perp_id)?;

    // calculate the new deposit's share of total value
    // TODO: longer comment, explain math
    let share_value_fraction = deposit_value / (deposit_value + subaccount_value);
    let outstanding_lp_tokens =
        Decimal::from_atomics(lp_token_info.total_supply, lp_token_info.decimals as u32).unwrap();
    let new_tokens = if share_value_fraction == Decimal::one() {
        // TODO: fix large initial deposit bug
        Uint128::new(10 as u128).pow(USDC_DENOM)
    } else {
        let token_amt_decimal = (share_value_fraction * outstanding_lp_tokens)
            / (Decimal::one() - share_value_fraction);
        decimal_to_native(token_amt_decimal, lp_token_info.decimals as u32)
    };

    // mint tokens to depositor
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    mint_lp_tokens(
        deps,
        sub_info,
        perp_id,
        info.sender.to_string(),
        new_tokens.into(),
    )
    .unwrap();

    assert!(amount.u128() < u64::MAX as u128);
    // Note that in general we cannot assume that Denom amount == quantums:
    // https://github.com/dydxprotocol/v4-chain/blob/c06db6fea945ad84fa4479df09078cee8feeba96/protocol/x/assets/types/asset.pb.go#L41
    // however for USDC this is the case:
    // https://github.com/dydxprotocol/v4-chain/blob/c06db6fea945ad84fa4479df09078cee8feeba96/protocol/x/assets/types/genesis.go#L18,
    let deposit = DydxMsg::DepositToSubaccountV1 {
        recipient: subaccount_id.clone(),
        asset_id: USDC_ID,
        quantums: amount.u128() as u64,
    };

    // TODO: more events, less debugging

    let event = Event::new("token_mint_math")
        .add_attribute("subaccount_value", subaccount_value.to_string())
        .add_attribute("deposit_value", deposit_value.to_string())
        .add_attribute("share_value_fraction", share_value_fraction.to_string())
        .add_attribute("outstanding_lp_tokens", outstanding_lp_tokens.to_string())
        .add_attribute("new_tokens", new_tokens.to_string());

    Ok(Response::new()
        .add_attribute("method", "deposit_into_vault")
        .add_event(event)
        .add_message(deposit))
}

/// User withdrawal from the LP vault. Requires that the user has enough LP tokens to support their requested withdrawal.
/// If 0 is passed as the withdrawal_amount, the max possible withdrawal will be executed.
pub fn withdraw_from_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    amount: u64,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let querier = DydxQuerier::new(&deps.querier);

    let vp = query_validated_dydx_position(&querier, &env, perp_id)?;
    let subaccount_value = vp.asset_usdc_value + vp.perp_usdc_value;

    // derive withdrawal value and LP burn amount

    let (withdraw_quantums, lp_burn_amount) = if amount == 0 {
        // withdraw all
        let (
            user_lp_tokens,
            user_lp_tokens_decimal,
            _outstanding_lp_tokens,
            outstanding_lp_tokens_decimal,
        ) = get_user_and_outstanding_lp_tokens(&deps, perp_id, &info.sender)?;
        let ownership_fraction = user_lp_tokens_decimal / outstanding_lp_tokens_decimal;

        let withdraw_value = ownership_fraction * subaccount_value;
        let withdraw_quantums = decimal_to_native(withdraw_value, USDC_DENOM);
        if withdraw_quantums >= u64::MAX.into() {
            return Err(ContractError::InvalidWithdrawalAmount {
                coin_type: USDC_COIN_TYPE.to_string(),
                amount: amount.into(),
            });
        }

        (withdraw_quantums.u128() as u64, user_lp_tokens)
    } else {
        // withdraw some
        let (
            _user_lp_tokens,
            user_lp_tokens_decimal,
            _outstanding_lp_tokens,
            outstanding_lp_tokens_decimal,
        ) = get_user_and_outstanding_lp_tokens(&deps, perp_id, &info.sender)?;
        let ownership_fraction = user_lp_tokens_decimal / outstanding_lp_tokens_decimal;

        let requested_withdraw_value = Decimal::from_atomics(amount, USDC_DENOM).unwrap();
        let max_withdraw_value = ownership_fraction * subaccount_value;

        let withdraw_quantums = decimal_to_native(requested_withdraw_value, USDC_DENOM);
        if withdraw_quantums >= u64::MAX.into() || requested_withdraw_value > max_withdraw_value {
            return Err(ContractError::InvalidWithdrawalAmount {
                coin_type: USDC_COIN_TYPE.to_string(),
                amount: amount.into(),
            });
        }

        let lp_token_burn_ratio = requested_withdraw_value / max_withdraw_value;
        let lp_burn_decimal = user_lp_tokens_decimal * lp_token_burn_ratio;
        let lp_token_info = lp_token_info(deps.as_ref(), perp_id)?;
        // always round up LP token burn
        let lp_burn_tokens =
            decimal_to_native(lp_burn_decimal, lp_token_info.decimals as u32) + Uint128::one();

        (withdraw_quantums.u128() as u64, lp_burn_tokens)
    };

    // burn withdrawer's LP tokens
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    burn_lp_tokens(
        deps,
        sub_info,
        perp_id,
        info.sender.to_string(),
        lp_burn_amount.into(),
    )
    .unwrap();

    let withdraw = DydxMsg::WithdrawFromSubaccountV1 {
        subaccount_number: perp_id,
        recipient: info.sender.to_string(),
        asset_id: USDC_ID,
        quantums: withdraw_quantums,
    };

    // let event = Event::new("")
    // .add_attribute("subaccount_value", subaccount_value.to_string())
    // .add_attribute("withdraw_value", deposit_value.to_string())
    // .add_attribute("share_value_fraction", share_value_fraction.to_string())
    // .add_attribute("outstanding_tokens", outstanding_tokens.to_string())
    // .add_attribute("new_tokens", new_tokens.to_string());

    Ok(Response::new()
        .add_attribute("method", "deposit_into_vault")
        // .add_event(event)
        .add_message(withdraw))
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

fn mint_lp_tokens(
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

fn burn_lp_tokens(
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

    assert!(amount <= config.total_supply); // TODO: proper error
                                            // update supply
    config.total_supply -= amount;
    LP_TOKENS.save(deps.storage, perp_id, &config)?;

    // remove amount from recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &rcpt_addr),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() - amount) },
    )?;

    Ok(())
}

/// Returns user and outstanding token balances.
/// Returns both raw and decimal versions.
fn get_user_and_outstanding_lp_tokens(
    deps: &DepsMut<DydxQueryWrapper>,
    perp_id: u32,
    user_addr: &Addr,
) -> ContractResult<(Uint128, Decimal, Uint128, Decimal)> {
    let lp_token_info = lp_token_info(deps.as_ref(), perp_id)?;
    let outstanding_lp_tokens =
        Decimal::from_atomics(lp_token_info.total_supply, lp_token_info.decimals as u32).unwrap();

    let ulp = match LP_BALANCES.may_load(deps.storage, (perp_id, &user_addr))? {
        Some(x) => x,
        None => {
            return Err(ContractError::LpTokensNotFound {
                user: user_addr.clone(),
                perp_id,
            })
        }
    };
    let user_lp_tokens = Decimal::from_atomics(ulp, lp_token_info.decimals as u32).unwrap();
    Ok((
        ulp,
        user_lp_tokens,
        lp_token_info.total_supply,
        outstanding_lp_tokens,
    ))
}

/// convert a decimal to native Uint128. Implicitly rounds down
fn decimal_to_native(decimal: Decimal, denom: u32) -> Uint128 {
    decimal.numerator() / Uint128::new(10 as u128).pow(Decimal::DECIMAL_PLACES - denom)
}
