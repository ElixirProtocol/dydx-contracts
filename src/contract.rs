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
            crate::execute::admin::set_trader(deps, info, new_trader).map_err(Into::into)
        }
        ExecuteMsg::CreateVault { perp_id } => {
            crate::execute::admin::create_vault(deps, env, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::ModifyVaultFee { perp_id } => {
            crate::execute::admin::modify_vault_fee(deps, env, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::CollectFeesFromVault { perp_id } => {
            crate::execute::admin::collect_fees_from_vault(deps, env, info, perp_id)
                .map_err(Into::into)
        }
        ExecuteMsg::DepositIntoVault { perp_id } => {
            crate::execute::deposit_withdraw::deposit_into_vault(deps, env, info, perp_id)
                .map_err(Into::into)
        }
        ExecuteMsg::RequestWithdrawal {
            usdc_amount,
            perp_id,
        } => crate::execute::deposit_withdraw::request_withdrawal(
            deps,
            env,
            info,
            usdc_amount,
            perp_id,
        )
        .map_err(Into::into),
        ExecuteMsg::CancelWithdrawalRequests { perp_id } => {
            crate::execute::deposit_withdraw::cancel_withdrawal_requests(deps, env, info, perp_id)
                .map_err(Into::into)
        }
        ExecuteMsg::ProcessWithdrawals {
            perp_id,
            max_num_withdrawals,
        } => crate::execute::deposit_withdraw::process_withdrawals(
            deps,
            env,
            info,
            perp_id,
            max_num_withdrawals,
        )
        .map_err(Into::into),
        ExecuteMsg::MarketMake {
            subaccount_number,
            clob_pair_id,
            new_orders,
            cancel_client_ids,
            cancel_good_til_block,
        } => crate::execute::order::market_make(
            deps,
            env,
            info,
            subaccount_number,
            clob_pair_id,
            new_orders,
            cancel_client_ids,
            cancel_good_til_block,
        )
        .map_err(Into::into),
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
        },
        LiquidityTiers => to_json_binary(&crate::query::liquidity_tiers(deps)?)
    }
}
