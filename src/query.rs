use crate::{
    dydx::{querier::DydxQuerier, query::{DydxQueryWrapper, PerpetualClobDetailsResponse}},
    error::{ContractError, ContractResult},
    execute::{USDC_DENOM, USDC_ID},
    msg::{
        DydxSubaccountResp, LpTokenBalanceResponse, TokenInfoResponse, TraderResp,
        VaultOwnershipResp, VaultStateResp,
    },
    state::{LP_BALANCES, LP_TOKENS, STATE, VAULT_STATES_BY_PERP_ID},
};
use cosmwasm_std::{Decimal, Deps, Env, StdResult, Uint128};
use num_traits::ToPrimitive;

pub fn other(deps: Deps<DydxQueryWrapper>,
    perp_id: u32) -> StdResult<PerpetualClobDetailsResponse>  {
        let querier = DydxQuerier::new(&deps.querier);
        querier.query_perpetual_clob_details(perp_id)
    }

pub fn trader(deps: Deps<DydxQueryWrapper>) -> StdResult<TraderResp> {
    let state = STATE.load(deps.storage)?;
    Ok(TraderResp {
        trader: state.trader,
    })
}

pub fn vault_state(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<VaultStateResp> {
    let vault = VAULT_STATES_BY_PERP_ID.load(deps.storage, perp_id)?;
    Ok(VaultStateResp {
        subaccount_owner: vault.subaccount_id.owner,
        subaccount_number: vault.subaccount_id.number,
        status: vault.status,
    })
}

pub fn vault_ownership(
    deps: Deps<DydxQueryWrapper>,
    env: Env,
    perp_id: u32,
    depositor: String,
) -> StdResult<VaultOwnershipResp> {
    let querier = DydxQuerier::new(&deps.querier);
    let vp = query_validated_dydx_position(&querier, &env, perp_id).unwrap();

    let raw_depositor_balance = balance(deps, perp_id, depositor)?;
    let lp_token_info = lp_token_info(deps, perp_id)?;

    Ok(VaultOwnershipResp {
        subaccount_owner: env.contract.address.to_string(),
        subaccount_number: perp_id,
        asset_usdc_value: vp.asset_usdc_value,
        perp_usdc_value: vp.perp_usdc_value,
        depositor_lp_tokens: raw_depositor_balance.balance,
        outstanding_lp_tokens: lp_token_info.total_supply,
    })
}

pub fn dydx_subaccount(
    deps: Deps<DydxQueryWrapper>,
    owner: String,
    number: u32,
) -> StdResult<DydxSubaccountResp> {
    let querier = DydxQuerier::new(&deps.querier);
    let subaccount = querier.query_subaccount(owner.clone(), number)?.subaccount;
    Ok(DydxSubaccountResp { subaccount })
}

pub fn balance(deps: Deps<DydxQueryWrapper>, perp_id: u32, address: String) -> StdResult<LpTokenBalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = LP_BALANCES
        .may_load(deps.storage, (perp_id, &address))?
        .unwrap_or_default();
    Ok(LpTokenBalanceResponse { balance })
}

pub fn lp_token_info(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<TokenInfoResponse> {
    let info = LP_TOKENS.may_load(deps.storage, perp_id)?
    .ok_or(ContractError::MissingLpToken {perp_id}).unwrap();
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub struct ValidatedDydxPosition {
    pub asset_usdc_value: Decimal,
    pub perp_usdc_value: Decimal,
}

/// Queries dYdX for a subaccount owned by the smart contract and market price of the perp.
/// Throws an error if the subaccount has any unexpected assets.
pub fn query_validated_dydx_position(
    querier: &DydxQuerier,
    env: &Env,
    perp_id: u32,
) -> ContractResult<ValidatedDydxPosition> {
    // TODO: ensure that this function works when money is in unfilled orders

    // query subaccount + price state from dYdX
    let clob_resp = querier.query_perpetual_clob_details(perp_id)?;
    let perp_params = clob_resp.perpetual_clob_details.perpetual.params;
    let market_price_resp = querier.query_market_price(perp_params.market_id)?;
    let subaccount_resp = querier.query_subaccount(env.contract.address.to_string(), perp_id)?;
    let subaccount = subaccount_resp.subaccount;

    if market_price_resp.market_price.exponent > 0 {
        return Err(ContractError::InvalidPriceExponent {
            exponent: market_price_resp.market_price.exponent,
            perp_id,
        });
    };
    let price_exponent = (-1 * market_price_resp.market_price.exponent) as u32;
    let price = Decimal::from_atomics(market_price_resp.market_price.price, price_exponent).unwrap();

    let usdc_position = subaccount
        .asset_positions
        .iter()
        .find(|p| p.asset_id == USDC_ID);
    let asset_usdc_value = match usdc_position {
        Some(p) => {
            let quantums: u128 = p.quantums.i.to_u128().unwrap();
            Decimal::from_atomics(quantums, USDC_DENOM).unwrap()
        }
        None => Decimal::zero(),
    };

    if perp_params.atomic_resolution > 0 {
        return Err(ContractError::InvalidPerpExponent {
            exponent: perp_params.atomic_resolution,
            perp_id,
        });
    };
    let perp_exponent = (-1 * perp_params.atomic_resolution) as u32;
    let perp_position = subaccount
        .perpetual_positions
        .iter()
        .find(|p| p.perpetual_id == perp_id);
    let perp_usdc_value = match perp_position {
        Some(p) => {
            let quantums: u128 = p.quantums.i.to_u128().unwrap();
            let position = Decimal::from_atomics(quantums, perp_exponent).unwrap();
            position * price
        }
        None => Decimal::zero(),
    };

    let validated_position = ValidatedDydxPosition {
        asset_usdc_value,
        perp_usdc_value,
    };

    Ok(validated_position)
}
