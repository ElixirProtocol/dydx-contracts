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
        ExecuteMsg::DepositIntoVault { amount, perp_id } => {
            crate::execute::deposit_into_vault(deps, env, info, perp_id, amount).map_err(Into::into)
        }
        ExecuteMsg::WithdrawFromVault => todo!(),
        ExecuteMsg::PlaceOrder => todo!(),
        ExecuteMsg::CancelOrder => todo!(),
        // ExecuteMsg::A { perp_id } => crate::execute::a(deps, env, info, perp_id).map_err(Into::into),
        // ExecuteMsg::B { perp_id } => crate::execute::b(deps, env, info, perp_id).map_err(Into::into),
        // ExecuteMsg::C { perp_id } => crate::execute::c(deps, env, info, perp_id).map_err(Into::into),
        // ExecuteMsg::D { perp_id } => crate::execute::d(deps, env, info, perp_id).map_err(Into::into),
        // ExecuteMsg::E { perp_id } => crate::execute::c(deps, env, info, perp_id).map_err(Into::into),
        // ExecuteMsg::PlaceOrder { order } => {
        //     crate::execute::place_order(deps, info, order).map_err(Into::into)
        // },
        // ExecuteMsg::CancelOrder {  order_id, good_til_oneof } => {
        //     crate::execute::cancel_order(deps, info, order_id, good_til_oneof).map_err(Into::into)
        // },
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
        Other { perp_id } => {
            to_json_binary(&crate::query::other(deps, perp_id)?)
        }
    }
}
