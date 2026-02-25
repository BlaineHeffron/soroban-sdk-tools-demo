//! Manual Token Contract (no soroban-sdk-tools)
//!
//! Same functionality as demo-token, written with raw SDK patterns.
//! Compare side-by-side to see what the macros replace.

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

// ---------------------------------------------------------------------------
// Storage keys — you write this enum by hand, one variant per field
// ---------------------------------------------------------------------------

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
    TotalSupply,
}

// ---------------------------------------------------------------------------
// Errors — you assign each code manually
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InsufficientBalance = 3,
    InsufficientAllowance = 4,
    InvalidAmount = 5,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ManualToken;

#[contractimpl]
impl ManualToken {
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), TokenError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TokenError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
        env.storage().instance().extend_ttl(50, 100);
        Ok(())
    }

    pub fn mint(env: &Env, to: &Address, amount: i128) -> Result<(), TokenError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(TokenError::NotInitialized)?;
        admin.require_auth();

        if amount <= 0 {
            return Err(TokenError::InvalidAmount);
        }

        let key = DataKey::Balance(to.clone());
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(balance + amount));
        env.storage().persistent().extend_ttl(&key, 50, 100);

        let supply_key = DataKey::TotalSupply;
        let supply: i128 = env.storage().instance().get(&supply_key).unwrap_or(0);
        env.storage().instance().set(&supply_key, &(supply + amount));

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

        let from_key = DataKey::Balance(from.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }
        env.storage().persistent().set(&from_key, &(from_bal - amount));

        let to_key = DataKey::Balance(to.clone());
        let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
        env.storage().persistent().set(&to_key, &(to_bal + amount));

        Ok(())
    }

    pub fn approve(
        env: &Env,
        owner: &Address,
        spender: &Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        owner.require_auth();
        let key = DataKey::Allowance(owner.clone(), spender.clone());
        env.storage().persistent().set(&key, &amount);
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

        let allowance_key = DataKey::Allowance(from.clone(), spender.clone());
        let allowance: i128 = env.storage().persistent().get(&allowance_key).unwrap_or(0);
        if allowance < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        env.storage()
            .persistent()
            .set(&allowance_key, &(allowance - amount));

        let from_key = DataKey::Balance(from.clone());
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance);
        }
        env.storage()
            .persistent()
            .set(&from_key, &(from_bal - amount));

        let to_key = DataKey::Balance(to.clone());
        let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
        env.storage().persistent().set(&to_key, &(to_bal + amount));

        Ok(())
    }

    pub fn balance(env: &Env, addr: &Address) -> i128 {
        let key = DataKey::Balance(addr.clone());
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
        let key = DataKey::Allowance(owner.clone(), spender.clone());
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn total_supply(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
