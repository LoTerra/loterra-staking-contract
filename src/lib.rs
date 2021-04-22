pub mod contract;
pub mod state;

mod global;
mod math;
mod taxation;
mod user;

mod msg;
#[cfg(test)]
mod testing;
mod claim;
mod helper;
#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
