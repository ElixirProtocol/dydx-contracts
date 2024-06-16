use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

use crate::dydx::proto_structs::SubaccountId;

#[cw_serde]
pub struct State {
    pub owner: Addr,
}

pub const TRADER_ADDRS: Map<&Addr, Empty> = Map::new("trader_addrs");
pub const NUM_VAULTS: Item<u32> = Item::new("num_vaults");
pub const VAULT_SUBACCOUNTS_BY_PERP_ID: Map<u32, SubaccountId> = Map::new("vault_subaccounts_by_perp_id");
pub const STATE: Item<State> = Item::new("state");
