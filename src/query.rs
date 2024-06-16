use cosmwasm_std::{Deps, StdResult};

use crate::{msg::TradersResp, state::TRADER_ADDRS};

pub fn admins(deps: Deps) -> StdResult<TradersResp> {
    let trader_addrs: Result<Vec<_>, _> = TRADER_ADDRS.keys(deps.storage, None, None, cosmwasm_std::Order::Ascending).collect();
    Ok(TradersResp {
        trader_addrs: trader_addrs?,
    })
}
