use cosmwasm_std::{WasmMsg, HumanAddr, StdResult, CosmosMsg, to_binary};
use crate::msg::QueryMsg;
/*
    Encode and prepare message to perfom a TransferFrom cw-20 LOTA contract in order to transfer
    staker LOTA to staking contract funds are locked as a custodian
*/
pub fn encode_msg_execute(msg: QueryMsg, address: HumanAddr) -> StdResult<CosmosMsg> {
    Ok(WasmMsg::Execute {
        contract_addr: address,
        msg: to_binary(&msg)?,
        send: vec![],
    }
        .into())
}