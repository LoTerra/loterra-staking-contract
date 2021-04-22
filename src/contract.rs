use crate::global::{handle_update_global_index};
use crate::state::{read_config, read_state, store_config, store_state, Config, State};
use crate::user::{handle_claim_rewards, handle_unbound, handle_bond, query_accrued_rewards, query_holder, query_holders, handle_withdraw_stake};
use cosmwasm_std::{to_binary, Api, Binary, Decimal, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult, Querier, StdResult, Storage, Uint128 };

use crate::msg::{ConfigResponse, HandleMsg, InitMsg, MigrateMsg, QueryMsg, StateResponse};
use crate::claim::query_claims;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let conf = Config {
        admin: deps.api.canonical_address(&env.message.sender)?,
        cw20_token_addr: deps
            .api
            .canonical_address(&msg.cw20_token_addr)?,
        reward_denom: msg.reward_denom,
        unbonding_period: msg.unbonding_period,
        safe_lock: msg.safe_lock
    };

    store_config(&mut deps.storage, &conf)?;
    store_state(
        &mut deps.storage,
        &State {
            global_index: Decimal::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::ClaimRewards { recipient } => handle_claim_rewards(deps, env, recipient),
        HandleMsg::UpdateGlobalIndex {} => handle_update_global_index(deps, env),
        HandleMsg::BondStake { amount } => handle_bond(deps, env, amount),
        HandleMsg::UnbondStake { amount } => handle_unbound(deps, env, amount),
        HandleMsg::WithdrawStake { cap } => handle_withdraw_stake(deps, env, cap)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(&deps)?),
        QueryMsg::State {} => to_binary(&query_state(&deps)?),
        QueryMsg::AccruedRewards { address } => to_binary(&query_accrued_rewards(&deps, address)?),
        QueryMsg::Holder { address } => to_binary(&query_holder(&deps, address)?),
        QueryMsg::Holders { start_after, limit } => to_binary(&query_holders(&deps, start_after, limit)?),
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let config: Config = read_config(&deps.storage)?;
    Ok(ConfigResponse {
        admin: deps.api.human_address(&config.admin)?,
        cw20_token_addr: deps.api.human_address(&config.cw20_token_addr)?,
        reward_denom: config.reward_denom,
        unbonding_period: 0
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

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
