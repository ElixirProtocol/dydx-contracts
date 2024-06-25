use cosmwasm_std::{
    to_json_binary, Addr, Decimal, DepsMut, Env, Event, Fraction, MessageInfo, Response, StdResult, Uint128, WasmMsg
};
use cw20_base::state::{MinterData, TokenInfo};

use crate::dydx::msg::{DydxMsg, Order};
use crate::dydx::proto_structs::SubaccountId;
use crate::dydx::querier::{self, DydxQuerier};
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::query::{lp_token_info, query_validated_dydx_position};
use crate::state::{VaultState, VaultStatus, LP_BALANCES, LP_TOKENS, VAULT_STATES_BY_PERP_ID};
use crate::{error::ContractError, state::STATE};

pub const USDC_ID: u32 = 0;
pub const USDC_DENOM: u32 = 6;

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
    const AMOUNT: u64 = 1;

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
        .add_message(withdraw))
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

/// Allows a user to deposit into the market-making vault.
///
pub fn deposit_into_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
    amount: u64,
) -> ContractResult<Response<DydxMsg>> {
    let subaccount_id = get_contract_subaccount_id(&env, perp_id);
    let querier = DydxQuerier::new(&deps.querier);

    // assert that user is depositing USDC

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
        Uint128::new(10 as u128).pow(USDC_DENOM)
    } else {
        let token_amt_decimal =
            (share_value_fraction * outstanding_tokens) / (Decimal::one() - share_value_fraction);
            token_amt_decimal.numerator() / Uint128::new(10 as u128).pow(Decimal::DECIMAL_PLACES - USDC_DENOM)
    };

    // mint tokens to depositor
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    do_mint(deps, sub_info, perp_id, info.sender.to_string(), new_tokens.into()).unwrap();

    let deposit = DydxMsg::DepositToSubaccount {
        sender: info.sender.to_string(),
        recipient: subaccount_id.clone(),
        asset_id: USDC_ID,
        quantums: amount,
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
