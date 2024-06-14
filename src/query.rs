use cosmwasm_std::{Deps, StdResult};

use crate::{msg::TradersResp, state::TRADER_ADDRS};

pub fn admins(deps: Deps) -> StdResult<TradersResp> {
    let admins = TRADER_ADDRS.load(deps.storage)?;
    Ok(TradersResp {
        trader_addrs: Vec::from_iter(admins),
    })
}
