use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::{
    dydx::{
        msg::{OrderConditionType, OrderSide, OrderTimeInForce},
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
    PerpClobDetails { perp_id: u32 },
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
    },
    WithdrawFromVault {
        perp_id: u32,
        amount: u64,
    },
    PlaceOrder {
        subaccount_number: u32,
        client_id: u32,
        order_flags: u32,
        clob_pair_id: u32,
        side: OrderSide,
        quantums: u64,
        subticks: u64,
        good_til_block_time: u32,
        time_in_force: OrderTimeInForce,
        reduce_only: bool,
        client_metadata: u32,
        condition_type: OrderConditionType,
        conditional_order_trigger_subticks: u64,
    },
    CancelOrder {
        subaccount_number: u32,
        client_id: u32,
        order_flags: u32,
        clob_pair_id: u32,
        good_til_block_time: u32,
    },
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
        msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce},
        proto_structs::SubaccountId,
    };

    #[test]

    fn example_serialize_place_order() {
        let side = OrderSide::Buy;
        let quantums = 1000000;
        let subticks = 100000;
        let time_in_force = OrderTimeInForce::Unspecified;
        let reduce_only = false;
        let client_metadata = 0;
        let condition_type = OrderConditionType::Unspecified;
        let conditional_order_trigger_subticks = 0;

        let msg = DydxMsg::PlaceOrder {
            subaccount_number: 0,
            client_id: 101,
            order_flags: 64,
            clob_pair_id: 0,
            side,
            quantums,
            subticks,
            good_til_block_time: 1234,
            time_in_force,
            reduce_only,
            client_metadata,
            condition_type,
            conditional_order_trigger_subticks,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_deposit() {
        let msg = DydxMsg::DepositToSubaccount {
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
