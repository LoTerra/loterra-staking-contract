use cosmwasm_std::{Coin, Decimal, StdResult, Uint128, DepsMut};

use terra_cosmwasm::TerraQuerier;

static DECIMAL_FRACTION: Uint128 = Uint128(1_000_000_000_000_000_000u128);

pub fn compute_tax(
    deps: DepsMut,
    coin: &Coin,
) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap(coin.denom.to_string())?).cap;
    Ok(std::cmp::min(
        (coin.amount.checked_sub(
            coin.amount.multiply_ratio(
                DECIMAL_FRACTION,
                DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
            )))?,
        tax_cap,
    ))
}

pub fn deduct_tax(
    deps: DepsMut,
    coin: Coin,
) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: Uint128(coin.amount.u128() - tax_amount.u128()),
    })
}
