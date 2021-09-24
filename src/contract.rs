use crate::global::handle_update_global_index;
use crate::state::{Config, State, CONFIG, STATE};
use crate::user::{
    handle_claim_rewards, handle_receive, handle_unbound, handle_withdraw_stake,
    query_accrued_rewards, query_holder, query_holders,
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};

use crate::claim::query_claims;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        cw20_token_addr: deps.api.addr_canonicalize(&msg.cw20_token_addr.as_str())?,
        cw20_token_reward_addr: deps.api.addr_canonicalize(&msg.cw20_token_reward_addr.as_str())?,
        unbonding_period: msg.unbonding_period,
        daily_rewards: msg.daily_rewards,
        open_every_block_time: msg.open_every_block_time,
    };

    CONFIG.save(deps.storage, &conf)?;
    STATE.save(
        deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
            open_block_time: env.block.time.seconds(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => handle_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::UpdateGlobalIndex {} => handle_update_global_index(deps, env),
        ExecuteMsg::UnbondStake { amount } => handle_unbound(deps, env, info, amount),
        ExecuteMsg::WithdrawStake { cap } => handle_withdraw_stake(deps, env, info, cap),
        ExecuteMsg::Receive(msg) => handle_receive(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, _env, msg)?),
        QueryMsg::State {} => to_binary(&query_state(deps, _env, msg)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(deps, start_after, limit)?)
        }
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

pub fn query_config(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        cw20_token_addr: deps.api.addr_humanize(&config.cw20_token_addr)?.to_string(),
        cw20_token_reward_addr: deps.api.addr_humanize(&config.cw20_token_reward_addr)?.to_string(),
        unbonding_period: config.unbonding_period,
        daily_rewards: config.daily_rewards,
        open_every_block_time: config.open_every_block_time,
    })
}
pub fn query_state(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
        open_block_time: state.open_block_time,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
