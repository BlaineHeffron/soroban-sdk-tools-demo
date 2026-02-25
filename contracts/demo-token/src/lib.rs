//! Demo Token Contract
//!
//! Shows three soroban-sdk-tools features in one contract:
//!   1. #[contractstorage] for typed storage with auto_shorten
//!   2. #[scerr] for composable error enums
//!   3. Auth test helpers (in test.rs)

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk_tools::{contractstorage, scerr, InstanceItem, PersistentMap};

// ---------------------------------------------------------------------------
// Storage: one macro replaces all the manual DataKey + get/set boilerplate
// ---------------------------------------------------------------------------

#[contractstorage(auto_shorten = true)]
pub struct TokenStorage {
    admin: InstanceItem<Address>,
    balances: PersistentMap<Address, i128>,
    allowances: PersistentMap<(Address, Address), i128>,
    total_supply: InstanceItem<i128>,
}

// ---------------------------------------------------------------------------
// Errors: sequential codes, doc-comment descriptions, flat WASM spec
// ---------------------------------------------------------------------------

#[scerr]
pub enum TokenError {
    /// already initialized
    AlreadyInitialized,
    /// not initialized
    NotInitialized,
    /// insufficient balance
    InsufficientBalance,
    /// insufficient allowance
    InsufficientAllowance,
    /// amount must be positive
    InvalidAmount,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct DemoToken;

#[contractimpl]
impl DemoToken {
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), TokenError> {
        if TokenStorage::get_admin(env).is_some() {
            return Err(TokenError::AlreadyInitialized);
        }
        TokenStorage::set_admin(env, admin);
        TokenStorage::set_total_supply(env, &0);
        env.storage().instance().extend_ttl(50, 100);
        Ok(())
    }

    pub fn mint(env: &Env, to: &Address, amount: i128) -> Result<(), TokenError> {
        let admin = TokenStorage::get_admin(env).ok_or(TokenError::NotInitialized)?;
        admin.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        // update() does atomic read-modify-write in one call
        TokenStorage::update_balances(env, to, |bal| bal.unwrap_or(0) + amount);
        TokenStorage::update_total_supply(env, |s| s.unwrap_or(0) + amount);
        TokenStorage::extend_balances_ttl(env, to, 50, 100);
        Ok(())
    }

    pub fn transfer(
        env: &Env,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        from.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let from_bal = TokenStorage::get_balances(env, from).unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }

        TokenStorage::set_balances(env, from, &(from_bal - amount));
        TokenStorage::update_balances(env, to, |bal| bal.unwrap_or(0) + amount);
        Ok(())
    }

    pub fn approve(
        env: &Env,
        owner: &Address,
        spender: &Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        owner.require_auth();
        TokenStorage::set_allowances(env, &(owner.clone(), spender.clone()), &amount);
        Ok(())
    }

    pub fn transfer_from(
        env: &Env,
        spender: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        spender.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let pair = (from.clone(), spender.clone());
        let allowance = TokenStorage::get_allowances(env, &pair).unwrap_or(0);
        if allowance < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        TokenStorage::set_allowances(env, &pair, &(allowance - amount));

        let from_bal = TokenStorage::get_balances(env, from).unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }
        TokenStorage::set_balances(env, from, &(from_bal - amount));
        TokenStorage::update_balances(env, to, |bal| bal.unwrap_or(0) + amount);
        Ok(())
    }

    pub fn balance(env: &Env, addr: &Address) -> i128 {
        TokenStorage::get_balances(env, addr).unwrap_or(0)
    }

    pub fn allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
        TokenStorage::get_allowances(env, &(owner.clone(), spender.clone())).unwrap_or(0)
    }

    pub fn total_supply(env: &Env) -> i128 {
        TokenStorage::get_total_supply(env).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
