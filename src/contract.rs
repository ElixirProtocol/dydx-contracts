use crate::{
    dydx::{msg::DydxMsg, query::DydxQueryWrapper},
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{State, STATE},
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw20_base::msg::MigrateMsg;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut<DydxQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<DydxMsg>> {
    // assert sender is the contract deployer
    let owner = deps.api.addr_validate(&msg.owner)?;
    if owner != &info.sender {
        return Err(ContractError::InvalidOwnerDuringInstantiation { owner });
    }

    let state = State {
        admin: owner.clone(),
        trader: owner,
        contract: env.contract.address,
    };
    STATE.save(deps.storage, &state)?;
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        } => crate::execute::market_make::market_make(
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

pub fn query(deps: Deps<DydxQueryWrapper>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Trader => to_json_binary(&crate::query::trader(deps)?),
        Vaults => to_json_binary(&crate::query::vaults(deps)?),
        VaultOwnership { perp_id, depositor } => {
            to_json_binary(&crate::query::vault_ownership(deps, perp_id, depositor)?)
        }
        DydxSubaccount { owner, number } => {
            to_json_binary(&crate::query::dydx_subaccount(deps, owner, number)?)
        }
        LiquidityTiers => to_json_binary(&crate::query::liquidity_tiers(deps)?),
        Withdrawals { perp_id } => to_json_binary(&crate::query::withdrawals(deps, perp_id)?),
        UserLpTokens { perp_id, user } => {
            to_json_binary(&crate::query::lp_balance(deps, perp_id, user)?)
        }
    }
}

pub fn migrate(
    deps: DepsMut<DydxQueryWrapper>,
    _env: Env,
    _msg: MigrateMsg,
) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err(format!(
            "Cannot upgrade from a newer version {} -> {}",
            ver.version, CONTRACT_VERSION
        ))
        .into());
    }
    // set the new version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // See https://medium.com/cosmwasm/cosmwasm-for-ctos-ii-advanced-usage-ee04ce95d1d0 for migration details
    // and note that migrate is called on the new version of the code
    // As long as the state in state.rs is the same between the new and old contract, it will be copied over automatically.
    // Otherwise, see https://github.com/CosmWasm/cosmwasm/blob/a0cf296c43aa092b81457d96a9c6bc2ab223f6d3/contracts/hackatom/src/contract.rs#L37-L48
    // for an example of migration where state does not match

    // since the smart contract address is the same, migration of funds in dYdX subaccounts is not necessary

    Ok(Response::default())
}
