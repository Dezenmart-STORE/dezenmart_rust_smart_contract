use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

// Re-export commonly used types from main contract
pub use dezenmart_rust_smart_contract::*;
pub use dezenmart_rust_smart_contract::dezenmart_logistics::*;

/// Helper function to create test public keys with predictable seeds
pub fn create_test_pubkey(seed: u8) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    Pubkey::new_from_array(bytes)
}

/// Generate a random keypair for testing
pub fn generate_keypair() -> Keypair {
    Keypair::new()
}

/// Test data generator for creating mock trade data
pub struct MockDataGenerator {
    pub admin: Keypair,
    pub sellers: Vec<Keypair>,
    pub buyers: Vec<Keypair>,
    pub logistics_providers: Vec<Keypair>,
}

impl MockDataGenerator {
    pub fn new() -> Self {
        Self {
            admin: generate_keypair(),
            sellers: (0..5).map(|_| generate_keypair()).collect(),
            buyers: (0..10).map(|_| generate_keypair()).collect(),
            logistics_providers: (0..3).map(|_| generate_keypair()).collect(),
        }
    }

    pub fn get_seller(&self, index: usize) -> &Keypair {
        &self.sellers[index % self.sellers.len()]
    }

    pub fn get_buyer(&self, index: usize) -> &Keypair {
        &self.buyers[index % self.buyers.len()]
    }

    pub fn get_logistics_provider(&self, index: usize) -> &Keypair {
        &self.logistics_providers[index % self.logistics_providers.len()]
    }
}

/// Mock trade parameters for testing
#[derive(Clone, Debug)]
pub struct MockTradeParams {
    pub product_cost: u64,
    pub logistics_providers: Vec<Pubkey>,
    pub logistics_costs: Vec<u64>,
    pub total_quantity: u64,
    pub token_mint: Pubkey,
}

impl Default for MockTradeParams {
    fn default() -> Self {
        Self {
            product_cost: 1000,
            logistics_providers: vec![create_test_pubkey(1), create_test_pubkey(2)],
            logistics_costs: vec![100, 150],
            total_quantity: 10,
            token_mint: create_test_pubkey(99),
        }
    }
}

impl MockTradeParams {
    pub fn with_product_cost(mut self, cost: u64) -> Self {
        self.product_cost = cost;
        self
    }

    pub fn with_quantity(mut self, quantity: u64) -> Self {
        self.total_quantity = quantity;
        self
    }

    pub fn with_single_logistics_provider(provider: Pubkey, cost: u64) -> Self {
        Self {
            logistics_providers: vec![provider],
            logistics_costs: vec![cost],
            ..Default::default()
        }
    }
}

/// Mock purchase parameters for testing
#[derive(Clone, Debug)]
pub struct MockPurchaseParams {
    pub trade_id: u64,
    pub quantity: u64,
    pub logistics_provider: Pubkey,
    pub buyer: Pubkey,
}

impl Default for MockPurchaseParams {
    fn default() -> Self {
        Self {
            trade_id: 1,
            quantity: 2,
            logistics_provider: create_test_pubkey(1),
            buyer: create_test_pubkey(50),
        }
    }
}

/// Expected calculation results for validation
pub struct ExpectedCalculations {
    pub total_product_cost: u64,
    pub total_logistics_cost: u64,
    pub total_amount: u64,
    pub product_escrow_fee: u64,
    pub logistics_escrow_fee: u64,
    pub seller_payout: u64,
    pub logistics_payout: u64,
}

impl ExpectedCalculations {
    pub fn new(product_cost: u64, logistics_cost: u64, quantity: u64) -> Self {
        let total_product_cost = product_cost * quantity;
        let total_logistics_cost = logistics_cost * quantity;
        let total_amount = total_product_cost + total_logistics_cost;

        let product_escrow_fee = (total_product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let logistics_escrow_fee = (total_logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

        let seller_payout = total_product_cost - product_escrow_fee;
        let logistics_payout = total_logistics_cost - logistics_escrow_fee;

        Self {
            total_product_cost,
            total_logistics_cost,
            total_amount,
            product_escrow_fee,
            logistics_escrow_fee,
            seller_payout,
            logistics_payout,
        }
    }
}

/// Assertion helpers for complex state validation
pub struct StateAssertions;

impl StateAssertions {
    pub fn assert_global_state(
        state: &GlobalState,
        expected_admin: &Pubkey,
        expected_trade_counter: u64,
        expected_purchase_counter: u64,
    ) {
        assert_eq!(state.admin, *expected_admin, "Global state admin mismatch");
        assert_eq!(state.trade_counter, expected_trade_counter, "Trade counter mismatch");
        assert_eq!(state.purchase_counter, expected_purchase_counter, "Purchase counter mismatch");
    }

    pub fn assert_trade_account(
        trade: &TradeAccount,
        expected_seller: &Pubkey,
        expected_cost: u64,
        expected_quantity: u64,
        expected_active: bool,
    ) {
        assert_eq!(trade.seller, *expected_seller, "Trade seller mismatch");
        assert_eq!(trade.product_cost, expected_cost, "Product cost mismatch");
        assert_eq!(trade.total_quantity, expected_quantity, "Total quantity mismatch");
        assert_eq!(trade.active, expected_active, "Trade active status mismatch");
    }

    pub fn assert_purchase_account(
        purchase: &PurchaseAccount,
        expected_buyer: &Pubkey,
        expected_quantity: u64,
        expected_amount: u64,
        expected_settled: bool,
    ) {
        assert_eq!(purchase.buyer, *expected_buyer, "Purchase buyer mismatch");
        assert_eq!(purchase.quantity, expected_quantity, "Purchase quantity mismatch");
        assert_eq!(purchase.total_amount, expected_amount, "Purchase amount mismatch");
        assert_eq!(purchase.settled, expected_settled, "Purchase settled status mismatch");
    }

    pub fn assert_registration_account<T>(
        account: &T,
        expected_registered: bool,
    )
    where
        T: RegistrationAccount,
    {
        assert_eq!(account.is_registered(), expected_registered, "Registration status mismatch");
    }
}

/// Trait for common registration account behavior
pub trait RegistrationAccount {
    fn is_registered(&self) -> bool;
    fn get_owner(&self) -> Pubkey;
}

impl RegistrationAccount for LogisticsProviderAccount {
    fn is_registered(&self) -> bool {
        self.is_registered
    }

    fn get_owner(&self) -> Pubkey {
        self.provider
    }
}

impl RegistrationAccount for SellerAccount {
    fn is_registered(&self) -> bool {
        self.is_registered
    }

    fn get_owner(&self) -> Pubkey {
        self.seller
    }
}

impl RegistrationAccount for BuyerAccount {
    fn is_registered(&self) -> bool {
        self.is_registered
    }

    fn get_owner(&self) -> Pubkey {
        self.buyer
    }
}

/// Error testing helpers
pub struct ErrorTestHelper;

impl ErrorTestHelper {
    pub fn expect_error<T>(result: Result<T, Box<dyn std::error::Error>>, expected_error: &str) {
        match result {
            Ok(_) => panic!("Expected error '{}', but operation succeeded", expected_error),
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains(expected_error),
                    "Expected error containing '{}', got '{}'",
                    expected_error,
                    error_msg
                );
            }
        }
    }

    pub fn should_fail_validation(condition: bool, error_name: &str) {
        assert!(!condition, "Validation should fail for: {}", error_name);
    }

    pub fn should_pass_validation(condition: bool, validation_name: &str) {
        assert!(condition, "Validation should pass for: {}", validation_name);
    }
}

/// Test case generator for boundary conditions
pub struct BoundaryTestCases;

impl BoundaryTestCases {
    pub fn zero_quantity_cases() -> Vec<u64> {
        vec![0]
    }

    pub fn edge_quantity_cases() -> Vec<u64> {
        vec![1, u64::MAX - 1, u64::MAX]
    }

    pub fn max_providers_cases() -> Vec<usize> {
        vec![MAX_LOGISTICS_PROVIDERS - 1, MAX_LOGISTICS_PROVIDERS, MAX_LOGISTICS_PROVIDERS + 1]
    }

    pub fn escrow_fee_edge_cases() -> Vec<(u64, u64)> {
        // (cost, quantity) pairs that test fee calculation edge cases
        vec![
            (1, 1),           // Minimal values
            (100, 1),         // Fee rounds to 0
            (1000, 1),        // Fee = 2.5
            (10000, 1),       // Fee = 25
            (u64::MAX / 1000, 1), // Large cost
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_data_generator() {
        let generator = MockDataGenerator::new();

        // Test that we get consistent keypairs
        let seller1 = generator.get_seller(0);
        let seller2 = generator.get_seller(0);
        assert_eq!(seller1.pubkey(), seller2.pubkey());

        // Test wraparound
        let seller_wrapped = generator.get_seller(generator.sellers.len());
        assert_eq!(seller1.pubkey(), seller_wrapped.pubkey());
    }

    #[test]
    fn test_expected_calculations() {
        let calc = ExpectedCalculations::new(1000, 100, 3);

        assert_eq!(calc.total_product_cost, 3000);
        assert_eq!(calc.total_logistics_cost, 300);
        assert_eq!(calc.total_amount, 3300);
        assert_eq!(calc.product_escrow_fee, 75); // 2.5% of 3000
        assert_eq!(calc.logistics_escrow_fee, 7);  // 2.5% of 300 (rounded down)
        assert_eq!(calc.seller_payout, 2925);     // 3000 - 75
        assert_eq!(calc.logistics_payout, 293);   // 300 - 7
    }

    #[test]
    fn test_mock_trade_params() {
        let params = MockTradeParams::default()
            .with_product_cost(2000)
            .with_quantity(5);

        assert_eq!(params.product_cost, 2000);
        assert_eq!(params.total_quantity, 5);
        assert_eq!(params.logistics_providers.len(), 2);
    }

    #[test]
    fn test_boundary_cases() {
        let zero_cases = BoundaryTestCases::zero_quantity_cases();
        assert_eq!(zero_cases, vec![0]);

        let max_provider_cases = BoundaryTestCases::max_providers_cases();
        assert!(max_provider_cases.contains(&MAX_LOGISTICS_PROVIDERS));
        assert!(max_provider_cases.contains(&(MAX_LOGISTICS_PROVIDERS + 1)));
    }
}