use crate::global::handle_update_global_index;
use crate::state::{Config, State, CONFIG, STATE};
use crate::user::{
    handle_claim_rewards, handle_receive, handle_unbound, handle_withdraw_stake,
    query_accrued_rewards, query_holder, query_holders,
};
use cosmwasm_std::{to_binary, Binary, Decimal, Env, StdResult, Uint128, DepsMut, MessageInfo, Response, entry_point, Deps};

use crate::claim::query_claims;
use crate::msg::{ConfigResponse, ExecuteMsg, MigrateMsg, QueryMsg, StateResponse, InstantiateMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let conf = Config {
        cw20_token_addr: deps.api.addr_canonicalize(&msg.cw20_token_addr.as_str())?,
        reward_denom: msg.reward_denom,
        unbonding_period: msg.unbonding_period,
    };

    CONFIG.save(deps.storage, &conf)?;
    STATE.save(deps.storage, &State {
        global_index: Decimal::zero(),
        total_balance: Uint128::zero(),
        prev_reward_balance: Uint128::zero(),
    })?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {

    match msg {
        ExecuteMsg::ClaimRewards { recipient } => handle_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::UpdateGlobalIndex {} => handle_update_global_index(deps, env),
        ExecuteMsg::UnbondStake { amount } => handle_unbound(deps, env, info, amount),
        ExecuteMsg::WithdrawStake { cap } => handle_withdraw_stake(deps, env, info, cap),
        ExecuteMsg::Receive(msg) => handle_receive(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps,  _env, msg)?),
        QueryMsg::State {} => to_binary(&query_state(deps,_env, msg)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(deps, start_after, limit)?)
        }
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

pub fn query_config(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        cw20_token_addr: deps.api.addr_humanize(&config.cw20_token_addr)?,
        reward_denom: config.reward_denom,
        unbonding_period: config.unbonding_period,
    })
}
pub fn query_state(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

