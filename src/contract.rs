use crate::global::handle_update_global_index;
use crate::state::{read_config, read_state, store_config, store_state, Config, State};
use crate::user::{
    handle_claim_rewards, handle_receive, handle_unbound, handle_withdraw_stake,
    query_accrued_rewards, query_holder, query_holders,
};
use cosmwasm_std::{to_binary, Api, Binary, Decimal, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult, Querier, StdResult, Storage, Uint128, DepsMut, MessageInfo, Response, Deps};

use crate::claim::query_claims;
use crate::msg::{ConfigResponse, HandleMsg, InitMsg, MigrateMsg, QueryMsg, StateResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let conf = Config {
        cw20_token_addr: deps.api.addr_canonicalize(&msg.cw20_token_addr)?,
        reward_denom: msg.reward_denom,
        unbonding_period: msg.unbonding_period,
    };

    store_config(deps.storage, &conf)?;
    store_state(
        deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {

    match msg {
        HandleMsg::ClaimRewards { recipient } => handle_claim_rewards(deps, env, info, recipient),
        HandleMsg::UpdateGlobalIndex {} => handle_update_global_index(deps, env, info),
        HandleMsg::UnbondStake { amount } => handle_unbound(deps, env, info, amount),
        HandleMsg::WithdrawStake { cap } => handle_withdraw_stake(deps, env, info, cap),
        HandleMsg::Receive(msg) => handle_receive(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(&deps)?),
        QueryMsg::State {} => to_binary(&query_state(&deps)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(&deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(&deps, address)?),
        QueryMsg::Holders { start_after, limit } => {
            to_binary(&query_holders(&deps, start_after, limit)?)
        }
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

pub fn query_config(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config: Config = read_config(&deps.storage)?;
    Ok(ConfigResponse {
        cw20_token_addr: deps.api.addr_humanize(&config.cw20_token_addr)?,
        reward_denom: config.reward_denom,
        unbonding_period: config.unbonding_period,
    })
}
fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(ConfigResponse {
        cw20_token_addr: deps.api.addr_humanize(&config.cw20_token_addr)?,
        reward_denom: config.reward_denom,
        unbonding_period: config.unbonding_period,
    })
}

fn query_state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<StateResponse> {
    let state: State = read_state(&deps.storage)?;
    Ok(StateResponse {
        global_index: state.global_index,
        total_balance: state.total_balance,
        prev_reward_balance: state.prev_reward_balance,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

