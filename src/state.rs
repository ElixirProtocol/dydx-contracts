use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw20_base::state::TokenInfo;
use cw_storage_plus::{Item, Map};

use crate::dydx::proto_structs::SubaccountId;

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub trader: Addr,
}

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

#[cw_serde]
pub struct Withdrawal {
    pub recipient_addr: Addr,
    pub lp_tokens: Uint128,
}

/// Keyed by perp_id
pub const LP_TOKENS: Map<u32, TokenInfo> = Map::new("lp_tokens");
/// Keyed by perp_id, addr
pub const LP_BALANCES: Map<(u32, &Addr), Uint128> = Map::new("balance");
pub const VAULT_STATES_BY_PERP_ID: Map<u32, VaultState> = Map::new("vault_states_by_perp_id");
pub const STATE: Item<State> = Item::new("state");
pub const WITHDRAWAL_QUEUES: Map<u32, Vec<Withdrawal>> = Map::new("withdrawal_queues");
