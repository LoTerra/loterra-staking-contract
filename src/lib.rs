pub mod contract;
pub mod state;

mod global;
mod math;
mod user;
mod taxation;

#[cfg(test)]
mod testing;
mod msg;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
