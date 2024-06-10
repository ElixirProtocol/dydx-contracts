use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminsResp)]
    Admins,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddAdmins { new_admins: Vec<String> },
    RemoveAdmins,
    CreateVault,
    FreezeVault,
    CloseVault,
    ModifyVault,
    CollectFeesFromVault,
    DepositIntoVault,
    WithdrawFromVault,
    PlaceOrder,
    CancelOrder,
    HaltTrading,
}

#[cw_serde]
pub struct AdminsResp {
    pub admin_addrs: Vec<Addr>,
}
