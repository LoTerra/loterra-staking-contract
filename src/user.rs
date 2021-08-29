use crate::state::{
    read_config, read_holder, read_holders, read_state, store_holder, store_state, Config, Holder,
    State,
};

use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, Coin, CosmosMsg, Decimal, Env, Extern,
    HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::claim::{claim_tokens, create_claim};
use crate::math::{
    decimal_multiplication_in_256, decimal_subtraction_in_256, decimal_summation_in_256,
};
use crate::msg::{AccruedRewardsResponse, HolderResponse, HoldersResponse, ReceiveMsg};
use crate::taxation::deduct_tax;
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg, Expiration};
use std::str::FromStr;

pub fn handle_claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = read_config(&deps.storage)?;
    let contract_addr = env.contract.address;
    let holder_addr = env.message.sender.clone();
    let holder_addr_raw = deps.api.canonical_address(&holder_addr)?;
    let recipient = match recipient {
        Some(value) => value,
        None => env.message.sender,
    };

    let mut holder: Holder = read_holder(&deps.storage, &holder_addr_raw)?;
    let mut state: State = read_state(&deps.storage)?;
    let config: Config = read_config(&deps.storage)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);
    let decimals = get_decimals(all_reward_with_decimals).unwrap();

    let rewards = all_reward_with_decimals * Uint128(1);

    if rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet"));
    }

    let new_balance = (state.prev_reward_balance - rewards)?;
    state.prev_reward_balance = new_balance;
    store_state(&mut deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    store_holder(&mut deps.storage, &holder_addr_raw, &holder)?;

    let msg_execute = Cw20HandleMsg::Transfer {
        recipient,
        amount: rewards,
    };
    let wasm_execute = WasmMsg::Execute {
        contract_addr: deps.api.human_address(&config.cw20_token_reward_addr)?,
        msg: to_binary(&msg_execute)?,
        send: vec![],
    };

    Ok(HandleResponse {
        messages: vec![wasm_execute.into()],
        log: vec![
            log("action", "claim_reward"),
            log("holder_address", holder_addr),
            log("rewards", rewards),
        ],
        data: None,
    })
}

pub fn handle_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    wrapper: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let config = read_config(&deps.storage)?;

    // only loterra cw20 contract can send receieve msg
    if env.message.sender != deps.api.human_address(&config.cw20_token_addr)? {
        return Err(StdError::GenericErr {
            msg: "only loterra contract can send receive messages".to_string(),
            backtrace: None,
        });
    }

    let msg: ReceiveMsg = match wrapper.msg {
        Some(bin) => from_binary(&bin),
        None => Err(StdError::parse_err("ReceiveMsg", "no data")),
    }?;

    let holder_addr = wrapper.sender;
    match msg {
        ReceiveMsg::BondStake {} => handle_bond(deps, env, holder_addr, wrapper.amount),
    }
}

pub fn handle_bond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    holder_addr: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    if !env.message.sent_funds.is_empty() {
        return Err(StdError::generic_err("Do not send funds with stake"));
    }
    if amount.is_zero() {
        return Err(StdError::generic_err("Amount required"));
    }

    let address_raw = deps.api.canonical_address(&holder_addr)?;

    let mut state: State = read_state(&deps.storage)?;
    let mut holder: Holder = read_holder(&deps.storage, &address_raw)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance += amount;
    state.total_balance += amount;

    store_holder(&mut deps.storage, &address_raw, &holder)?;
    store_state(&mut deps.storage, &state)?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "bond_stake"),
            log("holder_address", holder_addr),
            log("amount", amount),
        ],
        data: None,
    };

    Ok(res)
}

pub fn handle_unbound<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = read_config(&deps.storage)?;
    let address_raw = deps.api.canonical_address(&env.message.sender)?;

    if !env.message.sent_funds.is_empty() {
        return Err(StdError::generic_err("Do not send funds with stake"));
    }
    if amount.is_zero() {
        return Err(StdError::generic_err("Amount required"));
    }

    let mut state: State = read_state(&deps.storage)?;
    let mut holder: Holder = read_holder(&deps.storage, &address_raw)?;
    if holder.balance < amount {
        return Err(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        )));
    }

    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance = (holder.balance - amount).unwrap();
    state.total_balance = (state.total_balance - amount).unwrap();

    store_holder(&mut deps.storage, &address_raw, &holder)?;
    store_state(&mut deps.storage, &state)?;

    // create claim
    let release_height = Expiration::AtHeight(env.block.height + config.unbonding_period);
    create_claim(&mut deps.storage, address_raw, amount, release_height)?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "unbond_stake"),
            log("holder_address", env.message.sender),
            log("amount", amount),
        ],
        data: None,
    };

    Ok(res)
}

pub fn handle_withdraw_stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cap: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let config = read_config(&deps.storage)?;
    let addr = deps.api.canonical_address(&env.message.sender)?;

    let amount = claim_tokens(&mut deps.storage, addr, &env.block, cap)?;
    if amount.is_zero() {
        return Err(StdError::GenericErr {
            msg: "Wait for the unbonding period".into(),
            backtrace: None,
        });
    }

    let cw20_human_addr = deps.api.human_address(&config.cw20_token_addr)?;
    let cw20_transfer_msg = Cw20HandleMsg::Transfer {
        recipient: env.message.sender.clone(),
        amount,
    };
    let msg = WasmMsg::Execute {
        contract_addr: cw20_human_addr,
        msg: to_binary(&cw20_transfer_msg)?,
        send: vec![],
    };

    Ok(HandleResponse {
        messages: vec![msg.into()],
        log: vec![
            log("action", "withdraw_stake"),
            log("holder_address", &env.message.sender),
            log("amount", amount),
        ],
        data: None,
    })
}

pub fn query_accrued_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<AccruedRewardsResponse> {
    let global_index = read_state(&deps.storage)?.global_index;

    let holder: Holder = read_holder(&deps.storage, &deps.api.canonical_address(&address)?)?;
    let reward_with_decimals =
        calculate_decimal_rewards(global_index, holder.index, holder.balance)?;
    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);

    let rewards = all_reward_with_decimals * Uint128(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<HolderResponse> {
    let holder: Holder = read_holder(&deps.storage, &deps.api.canonical_address(&address)?)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.canonical_address(&start_after)?)
    } else {
        None
    };

    let holders: Vec<HolderResponse> = read_holders(&deps, start_after, limit)?;

    Ok(HoldersResponse { holders })
}

// calculate the reward based on the sender's index and the global index.
fn calculate_decimal_rewards(
    global_index: Decimal,
    user_index: Decimal,
    user_balance: Uint128,
) -> StdResult<Decimal> {
    let decimal_balance = Decimal::from_ratio(user_balance, Uint128(1));
    Ok(decimal_multiplication_in_256(
        decimal_subtraction_in_256(global_index, user_index),
        decimal_balance,
    ))
}

// calculate the reward with decimal
fn get_decimals(value: Decimal) -> StdResult<Decimal> {
    let stringed: &str = &*value.to_string();
    let parts: &[&str] = &*stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal::zero()),
        2 => {
            let decimals = Decimal::from_str(&*("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(StdError::generic_err("Unexpected number of dots")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn proper_calculate_rewards() {
        let global_index = Decimal::from_ratio(Uint128(9), Uint128(100));
        let user_index = Decimal::zero();
        let user_balance = Uint128(1000);
        let reward = calculate_decimal_rewards(global_index, user_index, user_balance).unwrap();
        assert_eq!(reward.to_string(), "90");
    }

    #[test]
    pub fn proper_get_decimals() {
        let global_index = Decimal::from_ratio(Uint128(9999999), Uint128(100000000));
        let user_index = Decimal::zero();
        let user_balance = Uint128(10);
        let reward = get_decimals(
            calculate_decimal_rewards(global_index, user_index, user_balance).unwrap(),
        )
        .unwrap();
        assert_eq!(reward.to_string(), "0.9999999");
    }
}
