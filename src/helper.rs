use cosmwasm_std::{WasmMsg, HumanAddr, StdResult, CosmosMsg, to_binary};
use crate::msg::{QueryMsg, HandleMsg};
/*
    Encode and prepare message to perfom a TransferFrom cw-20 LOTA contract in order to transfer
    staker LOTA to staking contract funds are locked as a custodian
*/