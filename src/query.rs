use cosmwasm_std::{Deps, StdResult};

use crate::{
    dydx::{querier::DydxQuerier, query::DydxQueryWrapper},
    msg::{DydxSubaccountResp, TraderResp, VaultStateResp},
    state::{STATE, VAULT_STATES_BY_PERP_ID},
};

pub fn trader(deps: Deps<DydxQueryWrapper>) -> StdResult<TraderResp> {
    let state = STATE.load(deps.storage)?;
    Ok(TraderResp {
        trader: state.trader,
    })
}

pub fn vault_state(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<VaultStateResp> {
    let vault = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    Ok(VaultStateResp {
        subaccount_owner: vault.subaccount_id.owner,
        subaccount_number: vault.subaccount_id.number,
        status: vault.status,
    })
}

pub fn dydx_subaccount(
    deps: Deps<DydxQueryWrapper>,
    owner: String,
    number: u32,
) -> StdResult<DydxSubaccountResp> {
    let querier = DydxQuerier::new(&deps.querier);
    let subaccount = querier.query_subaccount(owner.clone(), number)?.subaccount;
    Ok(DydxSubaccountResp { subaccount })
}
