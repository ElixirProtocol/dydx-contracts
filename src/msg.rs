use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::{dydx::proto_structs::{Subaccount, SubaccountId}, state::{Trader, VaultStatus}};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TradersResp)]
    Traders,
    #[returns(VaultStateResp)]
    VaultState { perp_id: u32 },
    #[returns(DydxSubaccountResp)]
    DydxSubaccount { owner: String, number: u32 }
}

#[cw_serde]
pub enum ExecuteMsg {
    AddTraders { new_traders: Vec<String> },
    RemoveTraders { traders_to_remove: Vec<String> },
    CreateVault { perp_id: u32 },
    FreezeVault { perp_id: u32 },
    ThawVault { perp_id: u32 },
    ChangeVaultTrader { perp_id: u32, new_trader: String },
    // ModifyVaultFee,
    // CollectFeesFromVault,
    DepositIntoVault,
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
pub struct TradersResp {
    pub traders: Vec<(Addr, Trader)>,
}

#[cw_serde]
pub struct VaultStateResp {
    pub subaccount_owner: String,
    pub subaccount_number: u32,
    pub status: VaultStatus,
}

#[cw_serde]
pub struct DydxSubaccountResp {
    pub subaccount: Subaccount
}
