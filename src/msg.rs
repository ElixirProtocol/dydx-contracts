use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::{
    dydx::{proto_structs::{Subaccount, SubaccountId}, query::PerpetualClobDetailsResponse},
    state::VaultStatus,
};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TraderResp)]
    Trader,
    #[returns(VaultStateResp)]
    VaultState { perp_id: u32 },
    #[returns(VaultOwnershipResp)]
    VaultOwnership { perp_id: u32, depositor: String },
    #[returns(DydxSubaccountResp)]
    DydxSubaccount { owner: String, number: u32 },
    #[returns(PerpetualClobDetailsResponse)]
    Other  { perp_id: u32 },
}

#[cw_serde]
pub enum ExecuteMsg {
    SetTrader { new_trader: String },
    CreateVault { perp_id: u32 },
    FreezeVault { perp_id: u32 },
    ThawVault { perp_id: u32 },
    // ModifyVaultFee,
    // CollectFeesFromVault,
    DepositIntoVault { perp_id: u32, amount: u64 },
    WithdrawFromVault,
    PlaceOrder,
    CancelOrder,
    // A { perp_id: u32 },
    // B { perp_id: u32 },
    // C { perp_id: u32 },
    // D { perp_id: u32 },
    // E { perp_id: u32 },
}

#[cw_serde]
pub struct TraderResp {
    pub trader: Addr,
}

#[cw_serde]
pub struct VaultStateResp {
    pub subaccount_owner: String,
    pub subaccount_number: u32,
    pub status: VaultStatus,
}

#[cw_serde]
pub struct VaultOwnershipResp {
    pub subaccount_owner: String,
    pub subaccount_number: u32,
    pub asset_usdc_value: Decimal,
    pub perp_usdc_value: Decimal,
    pub depositor_lp_tokens: Uint128,
    pub outstanding_lp_tokens: Uint128,
}

#[cw_serde]
pub struct DydxSubaccountResp {
    pub subaccount: Subaccount,
}

#[cw_serde]
pub struct LpTokenBalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}
