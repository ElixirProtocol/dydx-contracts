use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw20_base::state::TokenInfo;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub admin: Addr,
    pub trader: Addr,
    pub contract: Addr,
}

#[cw_serde]
pub struct WithdrawalRequest {
    pub recipient_addr: Addr,
    pub lp_tokens: Uint128,
}

/// A map of tracks LP tokens and their metadata Keyed by perp_id.
pub const LP_TOKENS: Map<u32, TokenInfo> = Map::new("lp_tokens");
/// A map that tracks user balances of LP tokens. Keyed by a tuple of (perp_id, Addr) and values are the raw LP token amount.
pub const LP_BALANCES: Map<(u32, &Addr), Uint128> = Map::new("balance");
/// A map that tracks the existence of perp markets.
pub const VAULTS_BY_PERP_ID: Map<u32, bool> = Map::new("vaults_by_perp_id");
/// A struct containing permissioned addresses for the smart contract.
pub const STATE: Item<State> = Item::new("state");
/// A map of withdrawals requests for each market. Withdrawal requests are a FIFO queue.
pub const WITHDRAWAL_QUEUES: Map<u32, Vec<WithdrawalRequest>> = Map::new("withdrawal_queues");
