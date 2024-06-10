use cosmwasm_std::{Deps, StdResult};

use crate::{msg::AdminsResp, state::ADMIN_ADDRS};

pub fn admins(deps: Deps) -> StdResult<AdminsResp> {
    let admins = ADMIN_ADDRS.load(deps.storage)?;
    Ok(AdminsResp {
        admin_addrs: Vec::from_iter(admins),
    })
}
