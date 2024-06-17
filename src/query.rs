use cosmwasm_std::{Deps, StdResult};

use crate::{
    dydx::query::DydxQueryWrapper,
    msg::{TradersResp, VaultStateResp},
    state::{TRADERS, VAULT_STATES_BY_PERP_ID},
};

pub fn admins(deps: Deps<DydxQueryWrapper>) -> StdResult<TradersResp> {
    let traders: Result<Vec<_>, _> = TRADERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();
    Ok(TradersResp { traders: traders? })
}

pub fn vault_state(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<VaultStateResp> {
    let vault = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    Ok(VaultStateResp {
        subaccount_owner: vault.subaccount_id.owner,
        subaccount_number: vault.subaccount_id.number,
        status: vault.status,
    })
}
