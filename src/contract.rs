use crate::{
    dydx::{msg::DydxMsg, query::DydxQueryWrapper},
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{State, STATE},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<DydxMsg>> {
    // assert sender is the contract deployer
    let owner = deps.api.addr_validate(&msg.owner)?;
    if owner != &info.sender {
        return Err(ContractError::InvalidOwnerDuringInstantiation { owner });
    }

    let state = State {
        owner: owner.clone(),
        trader: owner,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<DydxMsg>> {
    match msg {
        ExecuteMsg::SetTrader { new_trader } => {
            crate::execute::set_trader(deps, info, new_trader).map_err(Into::into)
        }
        ExecuteMsg::CreateVault { perp_id } => {
            crate::execute::create_vault(deps, env, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::FreezeVault { perp_id } => {
            crate::execute::freeze_vault(deps, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::ThawVault { perp_id } => {
            crate::execute::thaw_vault(deps, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::DepositIntoVault { perp_id } => {
            crate::execute::deposit_into_vault(deps, env, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::RequestWithdrawal {
            usdc_amount,
            perp_id,
        } => crate::execute::request_withdrawal(deps, env, info, usdc_amount, perp_id)
            .map_err(Into::into),
        ExecuteMsg::CancelWithdrawalRequests { perp_id } => {
            crate::execute::cancel_withdrawal_requests(deps, env, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::ProcessWithdrawals {
            perp_id,
            max_num_withdrawals,
        } => crate::execute::process_withdrawals(deps, env, info, perp_id, max_num_withdrawals)
            .map_err(Into::into),
        ExecuteMsg::PlaceOrder {
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
            conditional_order_trigger_subticks,
        } => crate::execute::place_order(
            deps,
            env,
            info,
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
            conditional_order_trigger_subticks,
        )
        .map_err(Into::into),
        ExecuteMsg::CancelOrder {
            subaccount_number,
            client_id,
            order_flags,
            clob_pair_id,
            good_til_block_time,
        } => crate::execute::cancel_order(
            deps,
            env,
            info,
            subaccount_number,
            client_id,
            order_flags,
            clob_pair_id,
            good_til_block_time,
        ),
        ExecuteMsg::BatchCancel {
            subaccount_number,
            order_batches,
            good_til_block,
        } => crate::execute::batch_cancel(
            deps,
            env,
            info,
            subaccount_number,
            order_batches,
            good_til_block,
        ),
    }
}

pub fn query(deps: Deps<DydxQueryWrapper>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Trader => to_json_binary(&crate::query::trader(deps)?),
        VaultState { perp_id } => to_json_binary(&crate::query::vault_state(deps, perp_id)?),
        VaultOwnership { perp_id, depositor } => to_json_binary(&crate::query::vault_ownership(
            deps, env, perp_id, depositor,
        )?),
        DydxSubaccount { owner, number } => {
            to_json_binary(&crate::query::dydx_subaccount(deps, owner, number)?)
        }
    }
}
