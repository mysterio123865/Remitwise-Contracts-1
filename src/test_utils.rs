#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

use crate::{RemitFlowContract, RemitFlowContractClient};

/// Assert that a generated client's `try_*` call returned a contract error.
///
/// Soroban wraps contract errors in an outer invocation result, which makes a
/// plain `assert_eq!` concise but difficult to diagnose when the call succeeds
/// or fails at the host layer. This helper reports the operation being checked
/// and distinguishes all of those outcomes.
pub(crate) fn assert_contract_error<T, E, H>(
    result: Result<Result<T, E>, Result<E, H>>,
    expected: E,
    operation: &str,
) where
    T: core::fmt::Debug,
    E: core::fmt::Debug + PartialEq,
    H: core::fmt::Debug,
{
    match result {
        Err(Ok(actual)) => assert_eq!(
            actual, expected,
            "{operation}: contract returned an unexpected error"
        ),
        Ok(value) => panic!(
            "{operation}: expected contract error {expected:?}, but the call succeeded with {value:?}"
        ),
        Err(Err(host_error)) => panic!(
            "{operation}: expected contract error {expected:?}, but invocation failed with host error {host_error:?}"
        ),
    }
}

pub(crate) const DEFAULT_SENDER_BALANCE: i128 = 1_000;
pub(crate) const DEFAULT_TRANSFER_AMOUNT: i128 = 400;
pub(crate) const DEFAULT_EXPIRY_OFFSET: u64 = 1_000;

/// Common fixture for contract tests.
///
/// The fixture deploys and initializes RemitFlow, deploys its Stellar Asset
/// Contract, creates the standard test actors, and funds the sender.
pub(crate) struct TestFixture<'a> {
    pub(crate) env: Env,
    pub(crate) client: RemitFlowContractClient<'a>,
    pub(crate) token: Address,
    pub(crate) admin: Address,
    pub(crate) from: Address,
    pub(crate) recipient: Address,
}

impl<'a> TestFixture<'a> {
    pub(crate) fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let from = Address::generate(&env);
        let recipient = Address::generate(&env);

        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token_contract.address();
        StellarAssetClient::new(&env, &token).mint(&from, &DEFAULT_SENDER_BALANCE);

        let contract_id = env.register_contract(None, RemitFlowContract);
        let client = RemitFlowContractClient::new(&env, &contract_id);
        client.initialize(&admin, &token);
        client.add_caller(&from);

        Self {
            env,
            client,
            token,
            admin,
            from,
            recipient,
        }
    }

    pub(crate) fn token_client(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.token)
    }

    pub(crate) fn future_expiry(&self) -> u64 {
        self.env.ledger().timestamp() + DEFAULT_EXPIRY_OFFSET
    }

    pub(crate) fn create_default_transfer(&self) -> u64 {
        self.client.create_transfer(
            &self.from,
            &self.recipient,
            &DEFAULT_TRANSFER_AMOUNT,
            &self.future_expiry(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::assert_contract_error;

    #[test]
    fn assert_contract_error_accepts_the_expected_error() {
        let result: Result<Result<(), u32>, Result<u32, &str>> = Err(Ok(7));

        assert_contract_error(result, 7, "create transfer");
    }

    #[test]
    #[should_panic(expected = "create transfer: contract returned an unexpected error")]
    fn assert_contract_error_reports_context_for_a_mismatch() {
        let result: Result<Result<(), u32>, Result<u32, &str>> = Err(Ok(8));

        assert_contract_error(result, 7, "create transfer");
    }

    #[test]
    #[should_panic(expected = "claim transfer: expected contract error 7, but the call succeeded")]
    fn assert_contract_error_reports_an_unexpected_success() {
        let result: Result<Result<u64, u32>, Result<u32, &str>> = Ok(Ok(1));

        assert_contract_error(result, 7, "claim transfer");
    }

    #[test]
    #[should_panic(
        expected = "cancel transfer: expected contract error 7, but invocation failed with host error \"budget exhausted\""
    )]
    fn assert_contract_error_reports_a_host_failure() {
        let result: Result<Result<(), u32>, Result<u32, &str>> = Err(Err("budget exhausted"));

        assert_contract_error(result, 7, "cancel transfer");
    }
}
