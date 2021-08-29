use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, HumanAddr, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub cw20_token_addr: HumanAddr,
    pub cw20_token_reward_addr: HumanAddr,
    pub unbonding_period: u64,
    pub daily_rewards: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ////////////////////
    /// Owner's operations
    ///////////////////

    /// Update the global index
    UpdateGlobalIndex {},

    ////////////////////
    /// Staking operations
    ///////////////////

    /// Unbound user staking balance
    /// Withdraw rewards to pending rewards
    /// Set current reward index to global index
    UnbondStake { amount: Uint128 },

    /// Unbound user staking balance
    /// Withdraws released stake
    WithdrawStake { cap: Option<Uint128> },

    ////////////////////
    /// User's operations
    ///////////////////

    /// return the accrued reward in usdt to the user.
    ClaimRewards { recipient: Option<HumanAddr> },

    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    /// Bond stake user staking balance
    /// Withdraw rewards to pending rewards
    /// Set current reward index to global index
    BondStake {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    AccruedRewards {
        address: HumanAddr,
    },
    Holder {
        address: HumanAddr,
    },
    Holders {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
    Claims {
        address: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub cw20_token_addr: HumanAddr,
    pub cw20_token_reward_addr: HumanAddr,
    pub unbonding_period: u64,
    pub daily_rewards: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
    pub open_block_time: u64,
    pub open_every_block_time: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccruedRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HolderResponse {
    pub address: HumanAddr,
    pub balance: Uint128,
    pub index: Decimal,
    pub pending_rewards: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldersResponse {
    pub holders: Vec<HolderResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
