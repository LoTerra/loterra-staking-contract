use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub cw20_token_addr: Addr,
    pub reward_denom: String,
    pub unbonding_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
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
    ClaimRewards { recipient: Option<Addr> },

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
        address: Addr,
    },
    Holder {
        address: Addr,
    },
    Holders {
        start_after: Option<Addr>,
        limit: Option<u32>,
    },
    Claims {
        address: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub cw20_token_addr: Addr,
    pub reward_denom: String,
    pub unbonding_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub global_index: Decimal,
    pub total_balance: Uint128,
    pub prev_reward_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccruedRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HolderResponse {
    pub address: Addr,
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
