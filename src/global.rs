use crate::state::{read_config, read_state, store_state, Config, State};

use crate::math::decimal_summation_in_256;
use cosmwasm_std::{
    log, Api, Decimal, Env, Extern, HandleResponse, Querier, StdError, StdResult,
    Storage,
};
use terra_cosmwasm::TerraMsgWrapper;

/// Increase global_index according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn handle_update_global_index<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: Config = read_config(&deps.storage)?;
    let mut state: State = read_state(&deps.storage)?;

    // anybody can trigger update_global_index
    /*
    if config.lottery_contract != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }
     */

    // Zero staking balance check
    if state.total_balance.is_zero() {
        return Err(StdError::generic_err("No asset is bonded by Hub"));
    }

    let reward_denom = read_config(&deps.storage)?.reward_denom;

    // Load the reward contract balance
    let balance = deps
        .querier
        .query_balance(env.contract.address, reward_denom.as_str())
        .unwrap();

    let previous_balance = state.prev_reward_balance;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = (balance.amount - previous_balance)?;

    state.prev_reward_balance = balance.amount;

    // global_index += claimed_rewards / total_balance;
    state.global_index = decimal_summation_in_256(
        state.global_index,
        Decimal::from_ratio(claimed_rewards, state.total_balance),
    );
    store_state(&mut deps.storage, &state)?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "update_global_index"),
            log("claimed_rewards", claimed_rewards),
        ],
        data: None,
    };

    Ok(res)
}
