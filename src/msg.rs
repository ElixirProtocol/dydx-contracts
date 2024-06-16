use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::Trader;

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
}

#[cw_serde]
pub enum ExecuteMsg {
    AddTraders { new_traders: Vec<String> },
    RemoveTraders { traders_to_remove: Vec<String> },
    CreateVault { perp_id: u32 },
    HaltTrading { perp_id: u32 },
    ChangeVaultTrader,
    // ModifyVaultFee,
    // CollectFeesFromVault,
    DepositIntoVault,
    WithdrawFromVault,
    PlaceOrder,
    CancelOrder,
}

#[cw_serde]
pub struct TradersResp {
    pub traders: Vec<(Addr, Trader)>,
}
