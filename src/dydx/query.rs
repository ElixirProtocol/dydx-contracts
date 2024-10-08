use cosmwasm_std::CustomQuery;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::proto_structs::{LiquidityTier, MarketPrice, PerpetualClobDetails, Subaccount};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DydxRoute {
    MarketPrice,
    Subaccount,
    PerpetualClobDetails,
    LiquidityTiers,
}

/// SeiQueryWrapper is an override of QueryRequest::Custom to access Sei-specific modules
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DydxQueryWrapper {
    pub route: DydxRoute,
    pub query_data: DydxQuery,
}

// implement custom query
impl CustomQuery for DydxQueryWrapper {}

/// SeiQuery is defines available query datas
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DydxQuery {
    MarketPrice { id: u32 },
    Subaccount { owner: String, number: u32 },
    PerpetualClobDetails { id: u32 },
    LiquidityTiers,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubaccountResponse {
    pub subaccount: Subaccount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketPriceResponse {
    pub market_price: MarketPrice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PerpetualClobDetailsResponse {
    pub perpetual_clob_details: PerpetualClobDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidityTiersResponse {
    pub liquidity_tiers: Vec<LiquidityTier>,
}
