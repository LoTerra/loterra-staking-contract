use crate::state::{read_config, read_state, store_state, State};

use crate::math::decimal_summation_in_256;
use cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Decimal, Env, Extern, HandleResponse, Querier, StdError,
    StdResult, Storage, WasmQuery,
};
use cw20::BalanceResponse;
use cw20::Cw20QueryMsg as Cw20Query;
/// Increase global_index according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn handle_update_global_index<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut state: State = read_state(&deps.storage)?;
    let config = read_config(&deps.storage)?;
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
    let balance_query = Cw20Query::Balance {
        address: env.contract.address,
    };
    let query_msg = WasmQuery::Smart {
        contract_addr: deps.api.human_address(&config.cw20_token_reward_addr)?,
        msg: to_binary(&balance_query)?,
    };
    // Load the reward contract balance
    let res: BalanceResponse = deps.querier.query(&query_msg.into())?;

    let previous_balance = state.prev_reward_balance;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = (res.balance - previous_balance)?;

    state.prev_reward_balance = res.balance;

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
