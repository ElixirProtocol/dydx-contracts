use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::serializable_int::SerializableInt;

// TODO(OTE-408): standardize proto compilation

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MarketPrice {
    #[serde(default)]
    pub id: u32,
    pub exponent: i32,
    pub price: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AssetPosition {
    #[serde(default)]
    pub asset_id: u32,
    pub quantums: SerializableInt,
    #[serde(default)]
    pub index: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
pub struct PerpetualPosition {
    pub perpetual_id: u32,
    pub quantums: SerializableInt,
    pub funding_index: SerializableInt,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubaccountId {
    pub owner: String,
    // go uses omit empty, so we need to provide a default value if not set(which is 0 for u32)
    #[serde(default)]
    pub number: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Subaccount {
    pub id: Option<SubaccountId>,
    #[serde(default)]
    pub asset_positions: Vec<AssetPosition>,
    #[serde(default)]
    pub perpetual_positions: Vec<PerpetualPosition>,
    #[serde(default)]
    pub margin_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ClobPair {
    #[serde(default)]
    pub id: u32,
    // metadata first letter is capitalized to match JSON
    #[serde(default)]
    pub metadata: Metadata,
    /// Minimum increment in the size of orders on the CLOB, in base quantums.
    #[serde(default)]
    pub step_base_quantums: u64,
    /// Defines the tick size of the orderbook by defining how many subticks
    /// are in one tick. That is, the subticks of any valid order must be a
    /// multiple of this value. Generally this value should start `>= 100`to
    /// allow room for decreasing it.
    #[serde(default)]
    pub subticks_per_tick: u32,
    /// `10^Exponent` gives the number of QuoteQuantums traded per BaseQuantum
    /// per Subtick.
    #[serde(default)]
    pub quantum_conversion_exponent: i32,
    #[serde(default)]
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")] // Ensure field names match JSON case
pub enum Metadata {
    PerpetualClobMetadata(PerpetualClobMetadata),
    SpotClobMetadata(SpotClobMetadata),
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata::PerpetualClobMetadata(PerpetualClobMetadata { perpetual_id: 0 })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PerpetualClobMetadata {
    #[serde(default)]
    pub perpetual_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SpotClobMetadata {
    #[serde(default)]
    pub base_asset_id: u32,
    #[serde(default)]
    pub quote_asset_id: u32,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Unspecified = 0,
    Active = 1,
    Paused = 2,
    CancelOnly = 3,
    PostOnly = 4,
    Initializing = 5,
    FinalSettlement = 6,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Perpetual {
    pub params: PerpetualParams,
    pub funding_index: SerializableInt,
    pub open_interest: SerializableInt,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PerpetualParams {
    #[serde(default)]
    pub id: u32,
    pub ticker: String,
    /// The market associated with this `Perpetual`. It
    /// acts as the oracle price for the purposes of calculating
    /// collateral, margin requirements, and funding rates.
    #[serde(default)]
    pub market_id: u32,
    /// The exponent for converting an atomic amount (`size = 1`)
    /// to a full coin. For example, if `AtomicResolution = -8`
    /// then a `PerpetualPosition` with `size = 1e8` is equivalent to
    /// a position size of one full coin.
    #[serde(default)]
    pub atomic_resolution: i32,
    #[serde(default)]
    pub default_funding_ppm: i32,
    #[serde(default)]
    pub liquidity_tier: u32,
    #[serde(default)]
    pub market_type: PerpetualMarketType,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum PerpetualMarketType {
    #[default]
    Unspecified = 0,
    Cross = 1,
    Isolated = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PerpetualClobDetails {
    pub perpetual: Perpetual,
    pub clob_pair: ClobPair,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OrderBatch {
    /// The Clob Pair ID all orders in this order batch belong to.
    pub clob_pair_id: u32,
    /// List of client ids in this order batch.
    /// Note that this is serialized as a uint32 instead of a fixed32 to
    /// avoid issues when decoding repeated packed fixed32.
    pub client_ids: Vec<u32>,
}
