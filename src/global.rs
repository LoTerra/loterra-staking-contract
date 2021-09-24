use crate::state::{CONFIG, STATE};

use crate::math::decimal_summation_in_256;
use cosmwasm_std::{Decimal, DepsMut, Env, Response, StdError, StdResult, WasmQuery, to_binary};
use cw20::{BalanceResponse, Cw20QueryMsg};
use std::ops::Add;

/// Increase global_index according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn handle_update_global_index(deps: DepsMut, env: Env) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // anybody can trigger update_global_index
    /*
    if config.lottery_contract != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }
     */

    if state.open_block_time > env.block.time.seconds() {
        return Err(StdError::generic_err("Unauthorized"));
    }
    // Zero staking balance check
    if state.total_balance.is_zero() {
        return Err(StdError::generic_err("No asset is bonded by Hub"));
    }
    let balance_query = Cw20QueryMsg::Balance  {
        address: env.contract.address.to_string(),
    };
    let query_msg = WasmQuery::Smart {
        contract_addr: deps.api.addr_humanize(&config.cw20_token_reward_addr)?.to_string(),
        msg: to_binary(&balance_query)?,
    };

    // Load the reward contract balance
    let res: BalanceResponse = deps.querier.query(&query_msg.into())?;
    let new_balance = if res.balance < config.daily_rewards {
        res.balance
    } else {
        config.daily_rewards
    };
    let previous_balance = state.prev_reward_balance;

    // New opening
    state.open_block_time = state.open_block_time + config.open_every_block_time;
    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = (/*res.balance*/new_balance.add(previous_balance) - previous_balance);

    state.prev_reward_balance = new_balance.add(previous_balance); //res.balance;

    // global_index += claimed_rewards / total_balance;
    state.global_index = decimal_summation_in_256(
        state.global_index,
        Decimal::from_ratio(claimed_rewards, state.total_balance),
    );
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "update_global_index")
        .add_attribute("claimed_rewards", claimed_rewards))
}
