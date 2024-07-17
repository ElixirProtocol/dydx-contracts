use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomQuery, Decimal, Uint128};

use crate::{
    dydx::{proto_structs::Subaccount, query::LiquidityTiersResponse},
    execute::market_make::NewOrder,
};

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TraderResponse)]
    Trader,
    #[returns(LpTokenBalanceResponse)]
    UserLpTokens { perp_id: u32, user: String },
    #[returns(VaultsResponse)]
    Vaults,
    #[returns(VaultOwnershipResponse)]
    VaultOwnership { perp_id: u32, depositor: String },
    #[returns(WithdrawalsResponse)]
    Withdrawals { perp_id: u32 },
    #[returns(DydxSubaccountResponse)]
    DydxSubaccount { owner: String, number: u32 },
    #[returns(LiquidityTiersResponse)]
    LiquidityTiers,
}

impl CustomQuery for QueryMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetTrader {
        new_trader: String,
    },
    CreateVault {
        perp_id: u32,
    },
    ModifyVaultFee {
        perp_id: u32,
    },
    CollectFeesFromVault {
        perp_id: u32,
    },
    DepositIntoVault {
        perp_id: u32,
    },
    RequestWithdrawal {
        perp_id: u32,
        usdc_amount: u64,
    },
    CancelWithdrawalRequests {
        perp_id: u32,
    },
    ProcessWithdrawals {
        perp_id: u32,
        max_num_withdrawals: u32,
    },
    MarketMake {
        subaccount_number: u32,
        clob_pair_id: u32,
        new_orders: Vec<NewOrder>,
        cancel_client_ids: Vec<u32>,
        cancel_good_til_block: u32,
    },
}

#[cw_serde]
pub struct TraderResponse {
    pub trader: Addr,
}

#[cw_serde]
pub struct WithdrawalResponse {
    pub recipient_addr: Addr,
    pub lp_tokens: Uint128,
    pub usdc_equivalent: Decimal,
}

#[cw_serde]
pub struct WithdrawalsResponse {
    pub withdrawal_queue: Vec<WithdrawalResponse>,
}

#[cw_serde]
pub struct VaultsResponse {
    /// perp ids
    pub vaults: Vec<u32>,
}

#[cw_serde]
pub struct VaultOwnershipResponse {
    pub subaccount_owner: String,
    pub subaccount_number: u32,
    pub asset_usdc_value: Decimal,
    pub perp_usdc_value: Decimal,
    pub depositor_lp_tokens: Uint128,
    pub outstanding_lp_tokens: Uint128,
}

#[cw_serde]
pub struct DydxSubaccountResponse {
    pub subaccount: Subaccount,
}

#[cw_serde]
pub struct LpTokenBalanceResponse {
    pub perp_id: u32,
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
    use crate::{
        dydx::{
            msg::{DydxMsg, OrderConditionType, OrderSide, OrderTimeInForce},
            proto_structs::SubaccountId,
        },
        execute::market_make::NewOrder,
        msg::{ExecuteMsg, QueryMsg},
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

        let msg = DydxMsg::PlaceOrderV1 {
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
    fn example_serialize_dydx_deposit() {
        let msg = DydxMsg::DepositToSubaccountV1 {
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

    #[test]
    fn example_serialize_request_withdrawal() {
        let msg = ExecuteMsg::RequestWithdrawal {
            perp_id: 0,
            usdc_amount: 100,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_process_withdrawals() {
        let msg = ExecuteMsg::ProcessWithdrawals {
            perp_id: 0,
            max_num_withdrawals: 1,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_market_make_place_order() {
        let msg = ExecuteMsg::MarketMake {
            subaccount_number: 0,
            clob_pair_id: 0,
            new_orders: vec![NewOrder {
                client_id: 101,
                side: OrderSide::Buy,
                quantums: 1000000,
                subticks: 100000,
                good_til_block_time: 1720791702,
                time_in_force: OrderTimeInForce::Unspecified,
                reduce_only: false,
                client_metadata: 0,
                conditional_order_trigger_subticks: 0,
            }],
            cancel_client_ids: vec![],
            cancel_good_til_block: 0,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_market_make_cancel_order() {
        let msg = ExecuteMsg::MarketMake {
            subaccount_number: 0,
            clob_pair_id: 0,
            new_orders: vec![],
            cancel_client_ids: vec![101],
            cancel_good_til_block: 0,
        };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_cancel_withdrawals() {
        let msg = ExecuteMsg::CancelWithdrawalRequests { perp_id: 0 };

        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }

    #[test]
    fn example_serialize_trader() {
        let msg = QueryMsg::Trader {};
        let serialized_msg = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized_msg);
    }
}
