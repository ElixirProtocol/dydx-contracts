use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::dydx::proto_structs::SubaccountId;

#[cw_serde]
pub struct State {
    pub owner: Addr,
}

#[cw_serde]
#[derive(Default)]
pub struct Trader {
    /// Index is subaccount #, value is market_id
    pub markets: Vec<u32>,
}
pub const DEFAULT_TRADER_CAPACITY: usize = 8;

#[cw_serde]
#[repr(u8)]
pub enum VaultStatus {
    Open,
    Frozen,
}

#[cw_serde]
pub struct VaultState {
    pub subaccount_id: SubaccountId,
    pub status: VaultStatus,
}

pub const TRADERS: Map<&Addr, Trader> = Map::new("traders");
pub const VAULT_STATES_BY_PERP_ID: Map<u32, VaultState> = Map::new("vault_states_by_perp_id");
pub const STATE: Item<State> = Item::new("state");
