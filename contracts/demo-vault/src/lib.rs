//! Demo Vault Contract
//!
//! Shows cross-contract error composition with #[scerr]:
//!   - Imports the demo-token WASM to get its error types
//!   - Composes VaultError from TokenError using #[from_contract_client]
//!   - The flattened WASM spec means TypeScript sees every variant

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk_tools::{contractstorage, scerr, InstanceItem, PersistentMap};

// Import the token contract — this generates the Client, error types, etc.
mod token {
    soroban_sdk_tools::contractimport!(
        file = "../../target/wasm32v1-none/release/demo_token.wasm"
    );
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[contractstorage(auto_shorten = true)]
pub struct VaultStorage {
    token_id: InstanceItem<Address>,
    admin: InstanceItem<Address>,
    deposits: PersistentMap<Address, i128>,
    total_deposited: InstanceItem<i128>,
}

// ---------------------------------------------------------------------------
// Errors — composed from local + token errors
// ---------------------------------------------------------------------------

#[scerr]
pub enum VaultError {
    /// vault already initialized
    AlreadyInitialized,
    /// vault not initialized
    NotInitialized,
    /// deposit amount must be positive
    InvalidAmount,
    /// withdrawal exceeds deposit
    InsufficientDeposit,

    // Propagate token errors through try_ calls with ??
    #[from_contract_client]
    Token(token::TokenError),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DemoVault;

#[contractimpl]
impl DemoVault {
    pub fn initialize(
        env: &Env,
        admin: &Address,
        token_id: &Address,
    ) -> Result<(), VaultError> {
        if VaultStorage::get_admin(env).is_some() {
            return Err(VaultError::AlreadyInitialized);
        }
        VaultStorage::set_admin(env, admin);
        VaultStorage::set_token_id(env, token_id);
        VaultStorage::set_total_deposited(env, &0);
        env.storage().instance().extend_ttl(50, 100);
        Ok(())
    }

    /// Deposit tokens into the vault.
    /// The vault calls token.transfer_from, so the user must approve first.
    pub fn deposit(env: &Env, user: &Address, amount: i128) -> Result<(), VaultError> {
        user.require_auth();

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_id = VaultStorage::get_token_id(env).ok_or(VaultError::NotInitialized)?;
        let vault_addr = env.current_contract_address();

        // try_transfer_from returns Result — ?? converts token errors via #[from_contract_client]
        token::Client::new(env, &token_id)
            .try_transfer_from(&vault_addr, user, &vault_addr, &amount)??;

        VaultStorage::update_deposits(env, user, |d| d.unwrap_or(0) + amount);
        VaultStorage::update_total_deposited(env, |t| t.unwrap_or(0) + amount);
        Ok(())
    }

    /// Withdraw tokens from the vault back to the user.
    pub fn withdraw(env: &Env, user: &Address, amount: i128) -> Result<(), VaultError> {
        user.require_auth();

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let deposit = VaultStorage::get_deposits(env, user).unwrap_or(0);
        if deposit < amount {
            return Err(VaultError::InsufficientDeposit);
        }

        let token_id = VaultStorage::get_token_id(env).ok_or(VaultError::NotInitialized)?;

        token::Client::new(env, &token_id).transfer(&env.current_contract_address(), user, &amount);

        VaultStorage::set_deposits(env, user, &(deposit - amount));
        VaultStorage::update_total_deposited(env, |t| t.unwrap_or(0) - amount);
        Ok(())
    }

    pub fn get_deposit(env: &Env, user: &Address) -> i128 {
        VaultStorage::get_deposits(env, user).unwrap_or(0)
    }

    pub fn total_deposited(env: &Env) -> i128 {
        VaultStorage::get_total_deposited(env).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
