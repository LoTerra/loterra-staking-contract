use crate::state::{read_holder, read_holders, store_holder, Config, Holder, State, CONFIG, STATE};

use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::claim::{claim_tokens, create_claim};
use crate::math::{
    decimal_multiplication_in_256, decimal_subtraction_in_256, decimal_summation_in_256,
};
use crate::msg::{AccruedRewardsResponse, HolderResponse, HoldersResponse, ReceiveMsg};
use crate::taxation::deduct_tax;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};
use std::str::FromStr;

pub fn handle_claim_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Option<Addr>,
) -> StdResult<Response> {
    let holder_addr = info.sender.clone();
    let holder_addr_raw = deps.api.addr_canonicalize(&holder_addr.as_str())?;
    let recipient = match recipient {
        Some(value) => value,
        None => info.sender,
    };

    let mut holder: Holder = read_holder(&deps.as_ref(), &holder_addr_raw)?;
    let mut state: State = STATE.load(deps.storage)?;
    let config: Config = CONFIG.load(deps.storage)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);
    let decimals = get_decimals(all_reward_with_decimals).unwrap();

    let rewards = all_reward_with_decimals * Uint128(1);

    if rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet"));
    }
    //let f = state.prev_reward_balance.wrapping_sub(rewards);
    let new_balance = Uint128(state.prev_reward_balance.u128() - rewards.u128());
    state.prev_reward_balance = new_balance;
    STATE.save(deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    store_holder(deps.storage, &holder_addr_raw, &holder)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![BankMsg::Send {
            to_address: Addr::to_string(&recipient),
            amount: vec![deduct_tax(
                deps,
                Coin {
                    denom: config.reward_denom,
                    amount: rewards,
                },
            )?],
        }
        .into()],
        data: None,
        attributes: vec![
            attr("action", "claim_reward"),
            attr("holder_address", holder_addr),
            attr("rewards", rewards),
        ],
    })
}

pub fn handle_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // only loterra cw20 contract can send receieve msg
    if info.sender != deps.api.addr_humanize(&config.cw20_token_addr)? {
        return Err(StdError::generic_err(
            "only loterra contract can send receive messages",
        ));
    }

    let holder_addr = deps.api.addr_validate(&wrapper.sender)?;
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    match msg {
        ReceiveMsg::BondStake {} => handle_bond(deps, env, info, holder_addr, wrapper.amount),
    }
}

pub fn handle_bond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    holder_addr: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    if !info.funds.is_empty() {
        return Err(StdError::generic_err("Do not send funds with stake"));
    }
    if amount.is_zero() {
        return Err(StdError::generic_err("Amount required"));
    }

    let address_raw = deps.api.addr_canonicalize(&holder_addr.as_str())?;

    let mut state: State = STATE.load(deps.storage)?;
    let mut holder: Holder = read_holder(&deps.as_ref(), &address_raw)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance += amount;
    state.total_balance += amount;

    store_holder(deps.storage, &address_raw, &holder)?;
    STATE.save(deps.storage, &state)?;

    let res = Response {
        submessages: vec![],
        messages: vec![],
        data: None,
        attributes: vec![
            attr("action", "bond_stake"),
            attr("holder_address", holder_addr),
            attr("amount", amount),
        ],
    };

    Ok(res)
}

pub fn handle_unbound(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;

    if !info.funds.is_empty() {
        return Err(StdError::generic_err("Do not send funds with stake"));
    }
    if amount.is_zero() {
        return Err(StdError::generic_err("Amount required"));
    }

    let mut state: State = STATE.load(deps.storage)?;
    let mut holder: Holder = read_holder(&deps.as_ref(), &address_raw)?;
    if holder.balance < amount {
        return Err(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        )));
    }

    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = decimal_summation_in_256(rewards, holder.pending_rewards);
    holder.balance = holder.balance.checked_sub(amount)?;
    state.total_balance = state.total_balance.checked_sub(amount)?;

    store_holder(deps.storage, &address_raw, &holder)?;
    STATE.save(deps.storage, &state)?;

    // create claim
    let release_height = Expiration::AtHeight(env.block.height + config.unbonding_period);
    create_claim(deps.storage, address_raw, amount, release_height)?;

    let res = Response {
        submessages: vec![],
        messages: vec![],
        data: None,
        attributes: vec![
            attr("action", "unbond_stake"),
            attr("holder_address", info.sender),
            attr("amount", amount),
        ],
    };

    Ok(res)
}

pub fn handle_withdraw_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cap: Option<Uint128>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let address_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;

    let amount = claim_tokens(deps.storage, address_raw, &env.block, cap)?;
    if amount.is_zero() {
        return Err(StdError::generic_err("Wait for the unbonding period"));
    }

    let cw20_human_addr = deps.api.addr_humanize(&config.cw20_token_addr)?;

    let cw20_transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string().clone(),
        amount,
    };
    let msg = WasmMsg::Execute {
        contract_addr: cw20_human_addr.to_string(),
        msg: to_binary(&cw20_transfer_msg)?,
        send: vec![],
    };

    Ok(Response {
        submessages: vec![],
        messages: vec![msg.into()],
        data: None,
        attributes: vec![
            attr("action", "withdraw_stake"),
            attr("holder_address", &info.sender),
            attr("amount", amount),
        ],
    })
}

pub fn query_accrued_rewards(deps: Deps, address: Addr) -> StdResult<AccruedRewardsResponse> {
    let global_index = STATE.load(deps.storage)?.global_index;

    let holder: Holder = read_holder(&deps, &deps.api.addr_canonicalize(&address.as_str())?)?;
    let reward_with_decimals =
        calculate_decimal_rewards(global_index, holder.index, holder.balance)?;
    let all_reward_with_decimals =
        decimal_summation_in_256(reward_with_decimals, holder.pending_rewards);

    let rewards = all_reward_with_decimals * Uint128(1);

    Ok(AccruedRewardsResponse { rewards })
}

pub fn query_holder(deps: Deps, address: Addr) -> StdResult<HolderResponse> {
    let holder: Holder = read_holder(&deps, &deps.api.addr_canonicalize(&address.as_str())?)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_canonicalize(&start_after.as_str())?)
    } else {
        None
    };

    let holders: Vec<HolderResponse> = read_holders(deps, start_after, limit)?;

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
