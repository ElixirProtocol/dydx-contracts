use cosmwasm_std::{Deps, StdResult};

use crate::{msg::TradersResp, state::TRADERS};

pub fn admins(deps: Deps) -> StdResult<TradersResp> {
    let traders: Result<Vec<_>, _> = TRADERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();
    Ok(TradersResp { traders: traders? })
}
