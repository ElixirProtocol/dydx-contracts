use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::{
    dydx::{
        msg::{GoodTilOneof, Order, OrderId},
        proto_structs::Subaccount,
        query::PerpetualClobDetailsResponse,
    },
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
    Other { perp_id: u32 },
}

#[cw_serde]
pub enum ExecuteMsg {
    SetTrader {
        new_trader: String,
    },
    CreateVault {
        perp_id: u32,
    },
    FreezeVault {
        perp_id: u32,
    },
    ThawVault {
        perp_id: u32,
    },
    // ModifyVaultFee,
    // CollectFeesFromVault,
    DepositIntoVault {
        perp_id: u32,
        amount: u64,
    },
    WithdrawFromVault {
        perp_id: u32,
        amount: u64,
    },
    PlaceOrder {
        order: Order,
    },
    CancelOrder {
        order_id: OrderId,
        good_til_oneof: GoodTilOneof,
    },
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

#[cfg(test)]
mod tests {
    use crate::dydx::{
        msg::{
            DydxMsg, GoodTilOneof, Order, OrderConditionType, OrderId, OrderSide, OrderTimeInForce,
        },
        proto_structs::SubaccountId,
    };

    #[test]

    fn example_serialize_place_order() {
        let order = Order {
            order_id: OrderId {
                subaccount_id: SubaccountId {
                    owner: "dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j"
                        .to_string(),
                    number: 0,
                },
                client_id: 101,
                order_flags: 64,
                clob_pair_id: 0,
            },
            side: OrderSide::Buy,
            quantums: 1000000,
            subticks: 100000,
            good_til_oneof: GoodTilOneof::GoodTilBlockTime(u32::MAX),
            time_in_force: OrderTimeInForce::Unspecified,
            reduce_only: false,
            client_metadata: 0,
            condition_type: OrderConditionType::Unspecified,
            conditional_order_trigger_subticks: 0,
        };
        let msg = DydxMsg::PlaceOrder { order };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_deposit() {
        let msg = DydxMsg::DepositToSubaccount {
            sender: "dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j".to_string(),
            recipient: SubaccountId {
                owner: "dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j"
                    .to_string(),
                number: 0,
            },
            asset_id: 0,
            quantums: 0,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }
}
