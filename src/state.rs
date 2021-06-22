use crate::msg::HolderResponse;
use cosmwasm_std::{CanonicalAddr, Decimal, Deps, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub cw20_token_addr: CanonicalAddr,
    pub reward_denom: String,
    pub unbonding_period: u64,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}
pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Holder {
    pub balance: Uint128,
    pub index: Decimal,
    pub pending_rewards: Decimal,
}

pub const PREFIXED_HOLDERS: Map<&[u8], Holder> = Map::new("holders");
// This is similar to HashMap<holder's address, Hodler>
pub fn store_holder(
    storage: &mut dyn Storage,
    holder_address: &CanonicalAddr,
    holder: &Holder,
) -> StdResult<()> {
    PREFIXED_HOLDERS.save(storage, holder_address.as_slice(), holder)
}

pub fn read_holder(deps: &Deps, holder_address: &CanonicalAddr) -> StdResult<Holder> {
    let res: Option<Holder> = PREFIXED_HOLDERS.may_load(deps.storage, holder_address.as_slice())?;

    match res {
        Some(holder) => Ok(holder),
        None => Ok(Holder {
            balance: Uint128::zero(),
            index: Decimal::zero(),
            pending_rewards: Decimal::zero(),
        }),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_holders(
    deps: Deps,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Vec<HolderResponse>> {
    let holder_bucket = PREFIXED_HOLDERS;
    //let holder_bucket: ReadonlyBucket<S, Holder> = bucket_read(PREFIX_HOLDERS, &deps.storage);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match calc_range_start(start_after){
        Some(start) => Some(Bound::Exclusive(start)),
        None => None
    };

    holder_bucket
        .range(
            deps.storage,
            start,
            None,
            Order::Descending,
        )
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let address = deps.api.addr_humanize(&CanonicalAddr::from(k))?;
            Ok(HolderResponse {
                address,
                balance: v.balance,
                index: v.index,
                pending_rewards: v.pending_rewards,
            })
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_slice().to_vec();
        v.push(1);
        v
    })
}
