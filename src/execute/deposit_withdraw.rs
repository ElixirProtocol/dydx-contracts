use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Response, SignedDecimal, Uint128};

use crate::dydx::msg::DydxMsg;
use crate::dydx::query::DydxQueryWrapper;
use crate::error::ContractResult;
use crate::execute::helpers::{
    burn_lp_tokens, decimal_to_native_round_down, get_contract_subaccount_id, mint_lp_tokens,
};
use crate::execute::{USDC_COIN_TYPE, USDC_DENOM, USDC_ID};
use crate::query::{lp_token_info, query_dydx_position};
use crate::state::{WithdrawalRequest, VAULTS_BY_PERP_ID, WITHDRAWAL_QUEUES};
use crate::{error::ContractError, state::STATE};

use super::helpers::{
    decimal_to_native_round_up, get_user_and_outstanding_lp_tokens,
    transfer_lp_tokens_from_withdrawal_queue, transfer_lp_tokens_to_withdrawal_queue,
};

/// Processes a user to deposit into the market-making vault.
/// This consists of minting LP tokens such that the % of LP tokens that 
/// a user owns is the same as their % of deposit value in the pool.
pub fn deposit_into_vault(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {

    let subaccount_id = get_contract_subaccount_id(&env, perp_id);

    // assert that user is depositing only USDC with amount > 0
    if info.funds.len() != 1 {
        return Err(ContractError::CanOnlyDepositOneCointype {});
    }
    if info.funds[0].denom != USDC_COIN_TYPE {
        return Err(ContractError::InvalidCoin {
            coin_type: info.funds[0].denom.clone(),
        });
    }

    let amount = info.funds[0].amount;
    if amount <= Uint128::zero() {
        return Err(ContractError::InvalidDepositAmount {
            coin_type: info.funds[0].denom.clone(),
            amount: amount.into(),
        });
    }

    // assert vault exists
    if !VAULTS_BY_PERP_ID.has(deps.storage, perp_id) {
        return Err(ContractError::VaultNotInitialized { perp_id });
    }

    let pos = query_dydx_position(deps.as_ref(), perp_id)?;
    let subaccount_value_signed = pos.asset_usdc_value + pos.perp_usdc_value;
    let subaccount_value = subaccount_value_signed.abs_diff(SignedDecimal::zero());
    let deposit_value = Decimal::from_atomics(amount, USDC_DENOM).unwrap();
    let lp_token_info = lp_token_info(deps.as_ref(), perp_id)?;

    // calculate the new deposit's share of total value using the following:
    //      new_tokens / (new_tokens + outstanding_lp_tokens) = deposit_value / (deposit_value + subaccount_value).
    let share_value_fraction = deposit_value / (deposit_value + subaccount_value);
    let outstanding_lp_tokens =
        Decimal::from_atomics(lp_token_info.total_supply, lp_token_info.decimals as u32).unwrap();
    let new_tokens = if share_value_fraction == Decimal::one() {
        amount
    } else {
        let token_amt_decimal = (share_value_fraction * outstanding_lp_tokens)
            / (Decimal::one() - share_value_fraction);
        decimal_to_native_round_down(token_amt_decimal, lp_token_info.decimals as u32).unwrap()
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

    let event = Event::new("new_deposit")
        .add_attribute("depositor", info.sender.to_string())
        .add_attribute("perp_id", perp_id.to_string())
        .add_attribute("usdc_amount", amount.to_string())
        .add_attribute("minted_lp_tokens", new_tokens.to_string())
        .add_attribute(
            "total_lp_tokens",
            (lp_token_info.total_supply + new_tokens).to_string(),
        );

    Ok(Response::new()
        .add_attribute("method", "deposit_into_vault")
        .add_event(event)
        .add_message(deposit))
}

/// Requests a user withdrawal from the LP vault. Requires that the user has enough LP tokens to support their requested withdrawal.
/// If 0 is passed as the usdc_amount, the max possible withdrawal will be requested.
/// Since withdrawals are processed some time in the future, a user may receive more/less USDC than they initially requested,
/// depending on how the value of the subaccount fluctuates.
pub fn request_withdrawal(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    usdc_amount: u64,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let (
        user_lp_tokens,
        user_lp_tokens_decimal,
        _outstanding_lp_tokens,
        outstanding_lp_tokens_decimal,
        lp_token_info,
    ) = get_user_and_outstanding_lp_tokens(&deps, perp_id, &info.sender)?;

    let lp_token_amount = if usdc_amount == 0 {
        // withdraw all
        user_lp_tokens
    } else {
        // withdraw some
        let pos = query_dydx_position(deps.as_ref(), perp_id)?;
        let subaccount_value_signed = pos.asset_usdc_value + pos.perp_usdc_value;
        let subaccount_value = subaccount_value_signed.abs_diff(SignedDecimal::zero());
        let ownership_fraction = user_lp_tokens_decimal / outstanding_lp_tokens_decimal;

        let requested_withdraw_value = Decimal::from_atomics(usdc_amount, USDC_DENOM).unwrap();
        let max_withdraw_value = ownership_fraction * subaccount_value;

        let lp_token_ratio = requested_withdraw_value / max_withdraw_value;
        let withdraw_lp_tokens_decimal = user_lp_tokens_decimal * lp_token_ratio;

        if withdraw_lp_tokens_decimal > user_lp_tokens_decimal {
            return Err(ContractError::InvalidWithdrawalAmount {
                coin_type: USDC_COIN_TYPE.to_string(),
                amount: usdc_amount.into(),
            });
        }

        let lp_tokens =
            decimal_to_native_round_up(withdraw_lp_tokens_decimal, lp_token_info.decimals as u32)
                .unwrap();
        lp_tokens
    };

    // put LP tokens into queue
    let withdrawal = WithdrawalRequest {
        recipient_addr: info.sender.clone(),
        lp_tokens: lp_token_amount,
    };

    let mut withdrawal_queue = WITHDRAWAL_QUEUES
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::MissingWithdrawalQueue { perp_id })?;
    withdrawal_queue.push(withdrawal);
    WITHDRAWAL_QUEUES.save(deps.storage, perp_id, &withdrawal_queue)?;

    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };

    // transfer LP tokens from the withdrawer to the contract temporarily
    transfer_lp_tokens_to_withdrawal_queue(
        deps,
        sub_info,
        perp_id,
        info.sender.to_string(),
        lp_token_amount,
    )?;

    let event = Event::new("new_withdrawal_request")
        .add_attribute("withdrawer", info.sender.to_string())
        .add_attribute("perp_id", perp_id.to_string())
        .add_attribute("usdc_amount", usdc_amount.to_string())
        .add_attribute("transferred_lp_tokens", lp_token_amount.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("method", "request_withdrawal"))
}

/// Cancels all user withdrawal requests from the LP vault.
/// Returns the user's LP tokens to them.
pub fn cancel_withdrawal_requests(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
) -> ContractResult<Response<DydxMsg>> {
    let mut withdrawal_queue = WITHDRAWAL_QUEUES
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::MissingWithdrawalQueue { perp_id })?;

    let mut i = 0;
    let mut restored_lp_tokens = Uint128::zero();
    while i < withdrawal_queue.len() {
        if withdrawal_queue[i].recipient_addr == info.sender {
            restored_lp_tokens += withdrawal_queue[i].lp_tokens;
            withdrawal_queue.remove(i);
        } else {
            i += 1;
        }
    }
    WITHDRAWAL_QUEUES.save(deps.storage, perp_id, &withdrawal_queue)?;

    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };

    if restored_lp_tokens > Uint128::zero() {
        // transfer LP tokens from the contract to the withdrawer
        transfer_lp_tokens_from_withdrawal_queue(
            deps,
            sub_info,
            perp_id,
            info.sender.to_string(),
            restored_lp_tokens,
        )?;
    }

    let event = Event::new("cancelled_withdrawal_requests")
        .add_attribute("withdrawer", info.sender.to_string())
        .add_attribute("perp_id", perp_id.to_string())
        .add_attribute("restored_lp_tokens", restored_lp_tokens.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("method", "cancel_withdrawal_requests"))
}

/// Processes user withdrawal requests as long as the dYdX subaccount allows it.
/// Burns LP tokens upon withdrawal.
/// Can only be called by the Trader
pub fn process_withdrawals(
    mut deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    perp_id: u32,
    mut max_num_withdrawals: u32,
) -> ContractResult<Response<DydxMsg>> {
    let state = STATE.load(deps.storage)?;
    // validate sender (must be configured trader)
    if info.sender != &state.trader {
        return Err(ContractError::SenderCannotProcessWithdrawals {
            sender: info.sender,
        });
    }

    let pos = query_dydx_position(deps.as_ref(), perp_id)?;
    let mut asset_value = pos.asset_usdc_value.clone();
    let perp_value = pos.perp_usdc_value;
    let subaccount_value_signed = pos.asset_usdc_value + pos.perp_usdc_value;
    let mut subaccount_value = subaccount_value_signed.abs_diff(SignedDecimal::zero());

    let (
        _queued_lp_tokens,
        _queued_lp_tokens_decimal,
        _outstanding_lp_tokens,
        outstanding_lp_tokens_decimal,
        lp_token_info,
    ) = get_user_and_outstanding_lp_tokens(&deps, perp_id, &env.contract.address)?;

    let mut withdrawal_queue = WITHDRAWAL_QUEUES
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::MissingWithdrawalQueue { perp_id })?;

    let mut withdraw_msgs = vec![];
    let mut withdraw_events = vec![];
    while max_num_withdrawals > 0 && withdrawal_queue.len() > 0 {
        let lp_amount = withdrawal_queue[0].lp_tokens;
        let lp_amount_decimal =
            Decimal::from_atomics(lp_amount, lp_token_info.decimals as u32).unwrap();
        let recipient = &withdrawal_queue[0].recipient_addr;

        // make quantums
        let ownership_fraction = lp_amount_decimal / outstanding_lp_tokens_decimal;
        let withdraw_value = ownership_fraction * subaccount_value;
        assert!(withdraw_value <= subaccount_value.into());
        assert!(ownership_fraction <= Decimal::one());

        let withdraw_quantums = decimal_to_native_round_down(withdraw_value, USDC_DENOM).unwrap();

        subaccount_value -= withdraw_value;
        asset_value -= SignedDecimal::try_from(withdraw_value).unwrap();
        // validate withdraw amount
        if withdraw_quantums >= u64::MAX.into() {
            return Err(ContractError::InvalidWithdrawalAmount {
                coin_type: USDC_COIN_TYPE.to_string(),
                amount: withdraw_quantums.into(),
            });
        };

        // validate health after withdrawal
        if perp_value > asset_value {
            return Err(ContractError::WithdrawalWouldIncreaseLeverageTooMuch { perp_id });
        };

        // make withdrawal message
        let withdraw_message = DydxMsg::WithdrawFromSubaccountV1 {
            subaccount_number: perp_id,
            recipient: recipient.to_string(),
            asset_id: USDC_ID,
            quantums: withdraw_quantums.u128() as u64,
        };
        withdraw_msgs.push(withdraw_message);

        let event = Event::new("processed_withdrawal")
            .add_attribute("recipient", recipient.to_string())
            .add_attribute("perp_id", perp_id.to_string())
            .add_attribute("withdrawn_usdc", withdraw_quantums.to_string())
            .add_attribute("burnt_lp_tokens", lp_amount.to_string());
        withdraw_events.push(event);

        // burn LP tokens
        let sub_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        burn_lp_tokens(&mut deps, &sub_info, perp_id, lp_amount)?;

        // pop from vec
        withdrawal_queue.remove(0);

        max_num_withdrawals -= 1;
    }
    WITHDRAWAL_QUEUES.save(deps.storage, perp_id, &withdrawal_queue)?;

    Ok(Response::new()
        .add_attribute("method", "process_withdrawals")
        .add_events(withdraw_events)
        .add_messages(withdraw_msgs))
}
