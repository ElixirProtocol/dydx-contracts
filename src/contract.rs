use std::collections::HashSet;

use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{State, ADMIN_ADDRS, STATE},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

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

    let mut admins = HashSet::new();
    admins.insert(owner.clone());

    let state = State { owner };
    STATE.save(deps.storage, &state)?;
    ADMIN_ADDRS.save(deps.storage, &admins)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::AddAdmins { new_admins } => {
            crate::execute::add_admins(deps, info, new_admins).map_err(Into::into)
        }
        ExecuteMsg::RemoveAdmins => todo!(),
        ExecuteMsg::CreateVault => todo!(),
        ExecuteMsg::FreezeVault => todo!(),
        ExecuteMsg::CloseVault => todo!(),
        ExecuteMsg::ModifyVault => todo!(),
        ExecuteMsg::CollectFeesFromVault => todo!(),
        ExecuteMsg::DepositIntoVault => todo!(),
        ExecuteMsg::WithdrawFromVault => todo!(),
        ExecuteMsg::PlaceOrder => todo!(),
        ExecuteMsg::CancelOrder => todo!(),
        ExecuteMsg::HaltTrading => todo!(),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Admins => to_json_binary(&crate::query::admins(deps)?),
    }
}
