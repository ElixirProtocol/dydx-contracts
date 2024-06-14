use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
// use dydx_cosmwasm::{Order, OrderId, SubaccountId};

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
    FreezeVault,
    ModifyVault,
    CollectFeesFromVault,
    DepositIntoVault,
    WithdrawFromVault,
    PlaceOrder,
    CancelOrder,
    HaltTrading,
}

// #[cw_serde]
// pub enum DydxMsg {
//   DepositToSubaccount {
//     sender: String,
//     recipient: SubaccountId,
//     asset_id: u32,
//     quantums: u64,
//   },
//     WithdrawFromSubaccount {
//         sender: SubaccountId,
//         recipient: String,
//         asset_id: u32,
//         quantums: u64,
//     },
//   PlaceOrder {
//     order: Order,
//   },
//   CancelOrder {
//     order_id: OrderId,
//     good_til_oneof: GoodTilOneof,
//   }
// }

// impl From<DydxMsg> for CosmosMsg<DydxMsg> {
//   fn from(original: DydxMsg) -> Self {
//     CosmosMsg::Any(original)
//   }
// }

// TODO: make public upstream ?
#[cw_serde]
pub enum GoodTilOneof {
    GoodTilBlock(u32),
    GoodTilBlockTime(u32),
}

#[cw_serde]
pub struct TradersResp {
    pub trader_addrs: Vec<Addr>,
}
