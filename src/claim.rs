use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{BlockInfo, StdResult, Storage, Uint128, CanonicalAddr};
// use cw_storage_plus::Map;
use cosmwasm_storage::{Bucket, ReadonlyBucket, bucket, bucket_read};
use cw20::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimsResponse {
    pub claims: Vec<Claim>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Claim {
    pub amount: Uint128,
    pub release_at: Expiration,
}

impl Claim {
    pub fn new(amount: u128, released: Expiration) -> Self {
        Claim {
            amount: amount.into(),
            release_at: released,
        }
    }
}

static CLAIM_KEY: &[u8] = b"claims";

pub fn claim_storage<T: Storage>(storage: &mut T) -> Bucket<T, Vec<Claim>> {
    bucket(CLAIM_KEY, storage)
}

pub fn claim_storage_read<T: Storage>(storage: &mut T) -> ReadonlyBucket<T, Vec<Claim>> {
    bucket_read(CLAIM_KEY, storage)
}


/// This creates a claim, such that the given address can claim an amount of tokens after
/// the release date.
pub fn create_claim<S: Storage>(
    storage: &mut S,
    addr: CanonicalAddr,
    amount: Uint128,
    release_at: Expiration,
) -> StdResult<()> {
    // add a claim to this user to get their tokens after the unbonding period
    claim_storage(storage).update(addr.as_slice(), |old| -> StdResult<_> {
        let mut claims = old.unwrap_or_default();
        claims.push(Claim { amount, release_at });
        Ok(claims)
    })?;

    Ok(())
}

/// This iterates over all mature claims for the address, and removes them, up to an optional cap.
/// it removes the finished claims and returns the total amount of tokens to be released.
/*
    TODO: claim stake need a Transfer WasmMsg::Execute in order
     to transfer cw-20 from the staking contract to claimer address
 */
pub fn claim_tokens<S: Storage>(
    storage: &mut S,
    addr: CanonicalAddr,
    block: &BlockInfo,
    cap: Option<Uint128>,
) -> StdResult<Uint128> {
    let mut to_send = Uint128(0);
    claim_storage(storage).update(addr.as_slice(), |claim| -> StdResult<_> {
        let (_send, waiting): (Vec<_>, _) =
            claim.unwrap_or_default().iter().cloned().partition(|c| {
                // if mature and we can pay fully, then include in _send
                if c.release_at.is_expired(block) {
                    if let Some(limit) = cap {
                        if to_send + c.amount > limit {
                            return false;
                        }
                    }
                    // TODO: handle partial paying claims?
                    to_send += c.amount;
                    true
                } else {
                    // not to send, leave in waiting and save again
                    false
                }
            });
        Ok(waiting)
    })?;
    Ok(to_send)
}
