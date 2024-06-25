use cosmwasm_std::{BankMsg, CosmosMsg, CustomMsg, Event};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use strum_macros::{Display, EnumString};

use super::proto_structs::SubaccountId;

// TODO(OTE-407): handle issue with `GoodTilOneof` in `PlaceOrder` and `CancelOrder` not serializing correctly

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Transfer {
    pub sender: SubaccountId,
    pub recipient: SubaccountId,
    pub asset_id: u32,
    pub amount: u64,
}

#[derive(
    Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema, EnumString, Display,
)]
#[repr(u32)]
pub enum OrderSide {
    Unspecified = 0,
    Buy = 1,
    Sell = 2,
}

#[derive(
    Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema, EnumString, Display,
)]
#[repr(u32)]
pub enum OrderTimeInForce {
    Unspecified = 0,
    Ioc = 1,
    PostOnly = 2,
    FillOrKill = 3,
}

#[derive(
    Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema, EnumString, Display,
)]
#[repr(u32)]
pub enum OrderConditionType {
    Unspecified = 0,
    StopLoss = 1,
    TakeProfit = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OrderId {
    pub subaccount_id: SubaccountId,
    pub client_id: u32,
    pub order_flags: u32,
    pub clob_pair_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Order {
    pub order_id: OrderId,
    pub side: OrderSide,
    pub quantums: u64,
    pub subticks: u64,
    pub good_til_oneof: GoodTilOneof,
    pub time_in_force: OrderTimeInForce,
    pub reduce_only: bool,
    pub client_metadata: u32,
    pub condition_type: OrderConditionType,
    pub conditional_order_trigger_subticks: u64,
}

impl Order {
    pub fn get_place_event(&self) -> Event {
        Event::new("placed_order")
            .add_attribute("owner", self.order_id.subaccount_id.owner.clone())
            .add_attribute(
                "subaccount_number",
                self.order_id.subaccount_id.number.to_string(),
            )
            .add_attribute("side", self.side.to_string())
            .add_attribute("quantums", self.quantums.to_string())
            .add_attribute("subticks", self.subticks.to_string())
            .add_attribute("time_in_force", self.time_in_force.to_string())
            .add_attribute("reduce_only", self.reduce_only.to_string())
            .add_attribute("client_metadata", self.client_metadata.to_string())
            .add_attribute("condition_type", self.condition_type.to_string())
            .add_attribute(
                "conditional_order_trigger_subticks",
                self.conditional_order_trigger_subticks.to_string(),
            )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoodTilOneof {
    GoodTilBlock(u32),
    GoodTilBlockTime(u32),
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DydxMsg {
    CreateTransfer {
        transfer: Transfer,
    },
    DepositToSubaccount {
        sender: String,
        recipient: SubaccountId,
        asset_id: u32,
        quantums: u64,
    },
    WithdrawFromSubaccount {
        sender: SubaccountId,
        recipient: String,
        asset_id: u32,
        quantums: u64,
    },
    PlaceOrder {
        order: Order,
    },
    CancelOrder {
        order_id: OrderId,
        good_til_oneof: GoodTilOneof,
    },
}

impl From<DydxMsg> for CosmosMsg<DydxMsg> {
    fn from(original: DydxMsg) -> Self {
        CosmosMsg::Custom(original)
    }
}

impl CustomMsg for DydxMsg {}
