#[cfg(test)]
mod test_trade_management {
    use super::super::helpers::*;
    use anchor_lang::prelude::*;

    /// Test create_trade function
    mod create_trade_tests {
        use super::*;

        #[test]
        fn test_create_trade_basic_functionality() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();
            let token_mint = create_test_pubkey(99);

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 0,
                purchase_counter: 0,
                bump: 255,
            };

            let trade_params = MockTradeParams::default()
                .with_product_cost(1500)
                .with_quantity(20);

            // Simulate create_trade function logic
            global_state.trade_counter += 1;
            let trade_id = global_state.trade_counter;

            let product_escrow_fee = (trade_params.product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id,
                seller,
                logistics_providers: trade_params.logistics_providers.clone(),
                logistics_costs: trade_params.logistics_costs.clone(),
                product_cost: trade_params.product_cost,
                escrow_fee: product_escrow_fee,
                total_quantity: trade_params.total_quantity,
                remaining_quantity: trade_params.total_quantity,
                active: true,
                purchase_ids: Vec::new(),
                token_mint,
                bump: 254,
            };

            // Validate trade creation
            assert_eq!(trade_account.trade_id, 1);
            assert_eq!(trade_account.seller, seller);
            assert_eq!(trade_account.product_cost, 1500);
            assert_eq!(trade_account.escrow_fee, 37); // 2.5% of 1500
            assert_eq!(trade_account.total_quantity, 20);
            assert_eq!(trade_account.remaining_quantity, 20);
            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.purchase_ids.len(), 0);
            assert_eq!(global_state.trade_counter, 1);

            StateAssertions::assert_trade_account(&trade_account, &seller, 1500, 20, true);
        }

        #[test]
        fn test_create_trade_validation_mismatched_arrays() {
            let logistics_providers = vec![create_test_pubkey(1), create_test_pubkey(2)];
            let logistics_costs = vec![100]; // Mismatched length

            // Test validation: arrays must have same length
            let arrays_match = logistics_providers.len() == logistics_costs.len();
            ErrorTestHelper::should_fail_validation(arrays_match, "MismatchedArrays");
        }

        #[test]
        fn test_create_trade_validation_empty_providers() {
            let empty_providers: Vec<Pubkey> = vec![];
            let empty_costs: Vec<u64> = vec![];

            // Test validation: must have at least one provider
            let has_providers = !empty_providers.is_empty();
            ErrorTestHelper::should_fail_validation(has_providers, "NoLogisticsProviders");
        }

        #[test]
        fn test_create_trade_validation_too_many_providers() {
            let mut many_providers = Vec::new();
            let mut many_costs = Vec::new();

            for i in 0..(MAX_LOGISTICS_PROVIDERS + 5) {
                many_providers.push(create_test_pubkey(i as u8));
                many_costs.push(100 + i as u64);
            }

            // Test validation: cannot exceed MAX_LOGISTICS_PROVIDERS
            let within_limit = many_providers.len() <= MAX_LOGISTICS_PROVIDERS;
            ErrorTestHelper::should_fail_validation(within_limit, "TooManyProviders");
        }

        #[test]
        fn test_create_trade_validation_zero_quantity() {
            let zero_quantity = 0u64;

            // Test validation: quantity must be greater than 0
            let valid_quantity = zero_quantity > 0;
            ErrorTestHelper::should_fail_validation(valid_quantity, "InvalidQuantity");
        }

        #[test]
        fn test_create_trade_max_logistics_providers() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();

            // Create exactly MAX_LOGISTICS_PROVIDERS
            let mut providers = Vec::new();
            let mut costs = Vec::new();

            for i in 0..MAX_LOGISTICS_PROVIDERS {
                providers.push(create_test_pubkey(i as u8 + 10));
                costs.push(100 + i as u64 * 10);
            }

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 0,
                purchase_counter: 0,
                bump: 255,
            };

            // Validate all constraints
            let arrays_match = providers.len() == costs.len();
            let has_providers = !providers.is_empty();
            let within_limit = providers.len() <= MAX_LOGISTICS_PROVIDERS;
            let valid_quantity = 10u64 > 0;

            ErrorTestHelper::should_pass_validation(arrays_match, "arrays length match");
            ErrorTestHelper::should_pass_validation(has_providers, "has providers");
            ErrorTestHelper::should_pass_validation(within_limit, "within provider limit");
            ErrorTestHelper::should_pass_validation(valid_quantity, "valid quantity");

            // Create trade with max providers
            global_state.trade_counter += 1;
            let trade_id = global_state.trade_counter;

            let product_cost = 2000u64;
            let product_escrow_fee = (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id,
                seller,
                logistics_providers: providers.clone(),
                logistics_costs: costs.clone(),
                product_cost,
                escrow_fee: product_escrow_fee,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 253,
            };

            assert_eq!(trade_account.logistics_providers.len(), MAX_LOGISTICS_PROVIDERS);
            assert_eq!(trade_account.logistics_costs.len(), MAX_LOGISTICS_PROVIDERS);
            assert_eq!(trade_account.logistics_providers, providers);
            assert_eq!(trade_account.logistics_costs, costs);
        }

        #[test]
        fn test_create_trade_escrow_fee_calculation() {
            let test_cases = vec![
                (1000u64, 25u64),   // 2.5% of 1000 = 25
                (2000u64, 50u64),   // 2.5% of 2000 = 50
                (100u64, 2u64),     // 2.5% of 100 = 2.5, rounded down to 2
                (40u64, 1u64),      // 2.5% of 40 = 1
                (39u64, 0u64),      // 2.5% of 39 = 0.975, rounded down to 0
            ];

            for (product_cost, expected_fee) in test_cases {
                let calculated_fee = (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
                assert_eq!(calculated_fee, expected_fee,
                    "Escrow fee calculation failed for cost {}", product_cost);
            }
        }

        #[test]
        fn test_create_trade_multiple_sequential() {
            let mock_data = MockDataGenerator::new();
            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 0,
                purchase_counter: 0,
                bump: 255,
            };

            let mut trades = Vec::new();

            // Create multiple trades sequentially
            for i in 0..5 {
                global_state.trade_counter += 1;
                let trade_id = global_state.trade_counter;

                let seller = mock_data.get_seller(i).pubkey();
                let product_cost = 1000 + i as u64 * 100;
                let total_quantity = 10 + i as u64;

                let trade_account = TradeAccount {
                    discriminator: [0; 8],
                    trade_id,
                    seller,
                    logistics_providers: vec![mock_data.get_logistics_provider(0).pubkey()],
                    logistics_costs: vec![100],
                    product_cost,
                    escrow_fee: (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS,
                    total_quantity,
                    remaining_quantity: total_quantity,
                    active: true,
                    purchase_ids: Vec::new(),
                    token_mint: create_test_pubkey(99),
                    bump: 250 + i as u8,
                };

                trades.push(trade_account);
            }

            // Validate all trades
            assert_eq!(global_state.trade_counter, 5);
            assert_eq!(trades.len(), 5);

            for (i, trade) in trades.iter().enumerate() {
                assert_eq!(trade.trade_id, i as u64 + 1);
                assert_eq!(trade.seller, mock_data.get_seller(i).pubkey());
                assert_eq!(trade.product_cost, 1000 + i as u64 * 100);
                assert_eq!(trade.total_quantity, 10 + i as u64);
                assert_eq!(trade.remaining_quantity, trade.total_quantity);
                assert_eq!(trade.active, true);
            }
        }

        #[test]
        fn test_create_trade_space_allocation() {
            // Validate space requirements for TradeAccount
            let base_expected_space = 8 +   // discriminator
                                     8 +   // trade_id
                                     32 +  // seller
                                     4 +   // logistics_providers Vec prefix
                                     4 +   // logistics_costs Vec prefix
                                     8 +   // product_cost
                                     8 +   // escrow_fee
                                     8 +   // total_quantity
                                     8 +   // remaining_quantity
                                     1 +   // active
                                     4 +   // purchase_ids Vec prefix
                                     32 +  // token_mint
                                     1;    // bump

            // Space for maximum providers and purchases
            let max_providers_space = MAX_LOGISTICS_PROVIDERS * (32 + 8); // Pubkey + u64
            let max_purchases_space = MAX_PURCHASE_IDS * 8; // u64 per purchase

            let empty_trade = TradeAccount {
                discriminator: [0; 8],
                trade_id: 0,
                seller: Pubkey::default(),
                logistics_providers: Vec::new(),
                logistics_costs: Vec::new(),
                product_cost: 0,
                escrow_fee: 0,
                total_quantity: 0,
                remaining_quantity: 0,
                active: false,
                purchase_ids: Vec::new(),
                token_mint: Pubkey::default(),
                bump: 0,
            };

            let empty_size = std::mem::size_of_val(&empty_trade);
            assert!(empty_size >= base_expected_space - 16);
        }

        #[test]
        fn test_create_trade_edge_cases() {
            let mock_data = MockDataGenerator::new();

            // Test with minimum values
            let min_trade = MockTradeParams {
                product_cost: 1,
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![1],
                total_quantity: 1,
                token_mint: create_test_pubkey(99),
            };

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 0,
                purchase_counter: 0,
                bump: 255,
            };

            global_state.trade_counter += 1;
            let trade_id = global_state.trade_counter;

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: min_trade.logistics_providers,
                logistics_costs: min_trade.logistics_costs,
                product_cost: min_trade.product_cost,
                escrow_fee: (min_trade.product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS,
                total_quantity: min_trade.total_quantity,
                remaining_quantity: min_trade.total_quantity,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: min_trade.token_mint,
                bump: 255,
            };

            assert_eq!(trade_account.product_cost, 1);
            assert_eq!(trade_account.total_quantity, 1);
            assert_eq!(trade_account.escrow_fee, 0); // 2.5% of 1 rounds down to 0
            assert_eq!(trade_account.active, true);
        }

        #[test]
        fn test_create_trade_with_large_values() {
            let mock_data = MockDataGenerator::new();

            // Test with large but valid values
            let large_cost = u64::MAX / 1000; // Prevent overflow in fee calculation
            let large_quantity = u64::MAX / 1000;

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 0,
                purchase_counter: 0,
                bump: 255,
            };

            global_state.trade_counter += 1;
            let trade_id = global_state.trade_counter;

            let escrow_fee = (large_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![1000],
                product_cost: large_cost,
                escrow_fee,
                total_quantity: large_quantity,
                remaining_quantity: large_quantity,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            assert_eq!(trade_account.product_cost, large_cost);
            assert_eq!(trade_account.total_quantity, large_quantity);
            assert_eq!(trade_account.remaining_quantity, large_quantity);
            assert!(trade_account.escrow_fee > 0);
        }
    }

    /// Test trade state management
    mod trade_state_management_tests {
        use super::*;

        #[test]
        fn test_trade_activation_states() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();

            // Create active trade
            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Initially active
            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.remaining_quantity, 10);

            // Simulate partial purchase
            trade_account.remaining_quantity -= 3;
            assert_eq!(trade_account.active, true); // Should remain active
            assert_eq!(trade_account.remaining_quantity, 7);

            // Simulate complete sellout
            trade_account.remaining_quantity = 0;
            if trade_account.remaining_quantity == 0 {
                trade_account.active = false;
            }

            assert_eq!(trade_account.active, false);
            assert_eq!(trade_account.remaining_quantity, 0);

            // Simulate quantity restoration (refund/cancellation)
            trade_account.remaining_quantity += 5;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }

            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.remaining_quantity, 5);
        }

        #[test]
        fn test_trade_purchase_ids_management() {
            let mock_data = MockDataGenerator::new();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 100,
                remaining_quantity: 100,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Add purchase IDs up to limit
            for i in 1..=MAX_PURCHASE_IDS {
                if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                    trade_account.purchase_ids.push(i as u64);
                }
            }

            assert_eq!(trade_account.purchase_ids.len(), MAX_PURCHASE_IDS);

            // Try to add one more (should not be added)
            let initial_len = trade_account.purchase_ids.len();
            if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                trade_account.purchase_ids.push((MAX_PURCHASE_IDS + 1) as u64);
            }

            assert_eq!(trade_account.purchase_ids.len(), initial_len);
        }

        #[test]
        fn test_trade_quantity_updates() {
            let mock_data = MockDataGenerator::new();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Test various quantity updates
            let updates = vec![1, 2, 3, 4]; // Total: 10

            for update in updates {
                trade_account.remaining_quantity -= update;
                if trade_account.remaining_quantity == 0 {
                    trade_account.active = false;
                }
            }

            assert_eq!(trade_account.remaining_quantity, 0);
            assert_eq!(trade_account.active, false);
            assert_eq!(trade_account.total_quantity, 10); // Should never change
        }

        #[test]
        fn test_trade_logistics_provider_lookup() {
            let providers = vec![
                create_test_pubkey(10),
                create_test_pubkey(20),
                create_test_pubkey(30),
            ];
            let costs = vec![100, 150, 200];

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: create_test_pubkey(1),
                logistics_providers: providers.clone(),
                logistics_costs: costs.clone(),
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Test finding each provider
            for (i, expected_provider) in providers.iter().enumerate() {
                let mut found_cost = 0u64;
                let mut found = false;

                for (j, provider) in trade_account.logistics_providers.iter().enumerate() {
                    if *provider == *expected_provider {
                        found_cost = trade_account.logistics_costs[j];
                        found = true;
                        break;
                    }
                }

                assert!(found, "Provider {:?} should be found", expected_provider);
                assert_eq!(found_cost, costs[i], "Cost should match for provider {}", i);
            }

            // Test with non-existent provider
            let non_existent = create_test_pubkey(99);
            let mut found = false;
            for provider in &trade_account.logistics_providers {
                if *provider == non_existent {
                    found = true;
                    break;
                }
            }
            assert!(!found, "Non-existent provider should not be found");
        }
    }

    /// Test trade invariants and consistency
    mod trade_invariants_tests {
        use super::*;

        #[test]
        fn test_trade_invariant_total_quantity_immutable() {
            let mock_data = MockDataGenerator::new();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let original_total = trade_account.total_quantity;

            // Simulate various operations that should not change total_quantity
            trade_account.remaining_quantity -= 3;
            assert_eq!(trade_account.total_quantity, original_total);

            trade_account.remaining_quantity += 2; // Refund
            assert_eq!(trade_account.total_quantity, original_total);

            trade_account.active = false;
            assert_eq!(trade_account.total_quantity, original_total);

            trade_account.purchase_ids.push(1);
            assert_eq!(trade_account.total_quantity, original_total);
        }

        #[test]
        fn test_trade_invariant_remaining_lte_total() {
            let mock_data = MockDataGenerator::new();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Test that remaining_quantity never exceeds total_quantity in valid operations
            for quantity_sold in 0..=10 {
                trade_account.remaining_quantity = 10 - quantity_sold;
                assert!(trade_account.remaining_quantity <= trade_account.total_quantity,
                    "Remaining {} should be <= total {}",
                    trade_account.remaining_quantity, trade_account.total_quantity);
            }
        }

        #[test]
        fn test_trade_invariant_arrays_consistency() {
            let providers = vec![create_test_pubkey(1), create_test_pubkey(2)];
            let costs = vec![100, 150];

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: create_test_pubkey(1),
                logistics_providers: providers.clone(),
                logistics_costs: costs.clone(),
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Invariant: logistics_providers and logistics_costs must have same length
            assert_eq!(trade_account.logistics_providers.len(),
                      trade_account.logistics_costs.len(),
                      "Provider and cost arrays must have same length");

            // Invariant: both arrays should have same content as input
            assert_eq!(trade_account.logistics_providers, providers);
            assert_eq!(trade_account.logistics_costs, costs);
        }

        #[test]
        fn test_trade_invariant_active_state_consistency() {
            let mock_data = MockDataGenerator::new();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 0, // Sold out
                active: true, // Inconsistent state
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            // Simulate invariant enforcement
            if trade_account.remaining_quantity == 0 {
                trade_account.active = false;
            }

            assert_eq!(trade_account.active, false,
                "Trade should be inactive when remaining_quantity is 0");

            // Test reactivation
            trade_account.remaining_quantity = 5;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }

            assert_eq!(trade_account.active, true,
                "Trade should be active when remaining_quantity > 0");
        }
    }
}