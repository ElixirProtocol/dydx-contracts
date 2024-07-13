use crate::dydx::proto_structs::SubaccountId;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractError;
use crate::error::ContractResult;
use crate::msg::TokenInfoResponse;
use crate::query::lp_token_info;
use crate::state::{LP_BALANCES, LP_TOKENS};
use cosmwasm_std::{
    Addr, CheckedMultiplyFractionError, Decimal, DepsMut, Env, Fraction, MessageInfo, StdResult,
    Uint128,
};

pub fn verify_trader(sender: &Addr, trader: &Addr) -> ContractResult<()> {
    if sender != trader {
        return Err(ContractError::SenderIsNotTrader {
            sender: sender.clone(),
        });
    } else {
        Ok(())
    }
}

pub fn validate_addr_string(
    deps: &DepsMut<DydxQueryWrapper>,
    addr_string: String,
) -> ContractResult<Addr> {
    match deps.api.addr_validate(&addr_string) {
        Ok(a) => Ok(a),
        Err(_) => return Err(ContractError::InvalidAddress { addr: addr_string }),
    }
}

pub fn get_contract_subaccount_id(env: &Env, perp_id: u32) -> SubaccountId {
    SubaccountId {
        owner: env.contract.address.to_string(),
        number: perp_id,
    }
}

pub fn mint_lp_tokens(
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

pub fn burn_lp_tokens(
    deps: &mut DepsMut<DydxQueryWrapper>,
    info: &MessageInfo,
    perp_id: u32,
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

    // remove amount from sender balance (always the smart contract)
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &info.sender),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() - amount) },
    )?;

    Ok(())
}

pub fn transfer_lp_tokens_to_withdrawal_queue(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    perp_id: u32,
    withdrawer: String,
    amount: Uint128,
) -> ContractResult<()> {
    let config = LP_TOKENS
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

    // remove amount from withdrawer's balance
    let rcpt_addr = deps.api.addr_validate(&withdrawer)?;
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &rcpt_addr),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() - amount) },
    )?;

    // add it to the contract's LP balance
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &info.sender), // guaranteed to be smart contract
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(())
}

pub fn transfer_lp_tokens_from_withdrawal_queue(
    deps: DepsMut<DydxQueryWrapper>,
    info: MessageInfo,
    perp_id: u32,
    withdrawer: String,
    amount: Uint128,
) -> ContractResult<()> {
    let config = LP_TOKENS
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

    // remove amount from contracts's balance
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &info.sender), // guaranteed to be smart contract
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() - amount) },
    )?;

    // add it to the withdrawer's LP balance
    let withdrawer_addr = deps.api.addr_validate(&withdrawer)?;
    LP_BALANCES.update(
        deps.storage,
        (perp_id, &withdrawer_addr),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(())
}

/// Returns user and outstanding token balances.
/// Returns both raw and decimal versions.
pub fn get_user_and_outstanding_lp_tokens(
    deps: &DepsMut<DydxQueryWrapper>,
    perp_id: u32,
    user_addr: &Addr,
) -> ContractResult<(Uint128, Decimal, Uint128, Decimal, TokenInfoResponse)> {
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
        lp_token_info,
    ))
}

/// convert a decimal to native Uint128. Rounds down
pub fn decimal_to_native_round_down(
    decimal: Decimal,
    denom: u32,
) -> Result<Uint128, CheckedMultiplyFractionError> {
    let frac = (
        Uint128::new(10 as u128).pow(Decimal::DECIMAL_PLACES - denom),
        Uint128::one(),
    );
    decimal.numerator().checked_div_floor(frac)
}
/// convert a decimal to native Uint128. Rounds up
pub fn decimal_to_native_round_up(
    decimal: Decimal,
    denom: u32,
) -> Result<Uint128, CheckedMultiplyFractionError> {
    let frac = (
        Uint128::new(10 as u128).pow(Decimal::DECIMAL_PLACES - denom),
        Uint128::one(),
    );
    decimal.numerator().checked_div_ceil(frac)
}
