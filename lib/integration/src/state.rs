use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub owner: Addr,
}

pub const TRADER_ADDRS: Item<HashSet<Addr>> = Item::new("trader_addrs");
pub const STATE: Item<State> = Item::new("state");
