use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{State, STATE, TRADER_ADDRS},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // assert sender is the contract deployer
    let owner = deps.api.addr_validate(&msg.owner)?;
    if owner != &info.sender {
        return Err(ContractError::InvalidOwnerDuringInstantiation { owner });
    }

    TRADER_ADDRS.save(deps.storage, &owner, &Empty {})?;
    let state = State { owner };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::AddTraders { new_traders } => {
            crate::execute::add_traders(deps, info, new_traders).map_err(Into::into)
        }
        ExecuteMsg::RemoveTraders { traders_to_remove } => {
            crate::execute::remove_traders(deps, info, traders_to_remove).map_err(Into::into)
        }
        ExecuteMsg::CreateVault { perp_id } => {
            crate::execute::create_vault(deps, info, perp_id).map_err(Into::into)
        }
        ExecuteMsg::ModifyVault => todo!(),
        ExecuteMsg::FreezeVault => todo!(),
        ExecuteMsg::CollectFeesFromVault => todo!(),
        ExecuteMsg::HaltTrading => todo!(),
        ExecuteMsg::DepositIntoVault => todo!(),
        ExecuteMsg::WithdrawFromVault => todo!(),
        ExecuteMsg::PlaceOrder => todo!(),
        ExecuteMsg::CancelOrder => todo!(),
        // ExecuteMsg::PlaceOrder { order } => {
        //     crate::execute::place_order(deps, info, order).map_err(Into::into)
        // },
        // ExecuteMsg::CancelOrder {  order_id, good_til_oneof } => {
        //     crate::execute::cancel_order(deps, info, order_id, good_til_oneof).map_err(Into::into)
        // },
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Traders => to_json_binary(&crate::query::admins(deps)?),
    }
}
