#![no_std]

//! RemitFlow: a cross-border remittance escrow contract for Soroban/Stellar.
//!
//! Senders lock token funds for a recipient with an expiry. The recipient can
//! claim the funds; the sender can cancel and reclaim them after expiry.

mod error;
mod events;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, token, Address, Env};

use crate::error::Error;
use crate::types::{Status, Transfer};

/// The RemitFlow remittance escrow contract.
#[contract]
pub struct RemitFlowContract;

#[contractimpl]
impl RemitFlowContract {
    /// Initialize the contract with an administrator and token address.
    ///
    /// Can only be called once; subsequent calls return
    /// [`Error::AlreadyInitialized`].
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_counter(&env, 0);
        storage::extend_instance(&env);
        events::init(&env, &admin, &token);
        Ok(())
    }

    /// Return the configured administrator address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Return the configured token contract address.
    pub fn get_token(env: Env) -> Result<Address, Error> {
        storage::get_token(&env).ok_or(Error::NotInitialized)
    }

    /// Return the number of transfers created so far.
    pub fn counter(env: Env) -> u64 {
        storage::get_counter(&env)
    }

    /// Create a new escrowed transfer from `from` to `recipient`.
    ///
    /// Transfers `amount` of the configured token from `from` into the
    /// contract and records a pending transfer that expires at `expiry`.
    /// Returns the new transfer's id.
    pub fn create_transfer(
        env: Env,
        from: Address,
        recipient: Address,
        amount: i128,
        expiry: u64,
    ) -> Result<u64, Error> {
        let token = storage::get_token(&env).ok_or(Error::NotInitialized)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if expiry <= env.ledger().timestamp() {
            return Err(Error::InvalidExpiry);
        }
        from.require_auth();

        let id = storage::get_counter(&env)
            .checked_add(1)
            .ok_or(Error::CounterOverflow)?;

        token::Client::new(&env, &token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );

        let transfer = Transfer {
            id,
            from: from.clone(),
            recipient: recipient.clone(),
            amount,
            expiry,
            status: Status::Pending,
        };
        storage::set_transfer(&env, &transfer);
        storage::set_counter(&env, id);
        storage::extend_instance(&env);
        events::created(&env, id, &from, &recipient, amount);
        Ok(id)
    }
}
