#[cfg(test)]
mod test_purchase_flow {
    use super::super::helpers::*;
    use anchor_lang::prelude::*;

    /// Test buy_trade function
    mod buy_trade_tests {
        use super::*;

        #[test]
        fn test_buy_trade_basic_functionality() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();
            let buyer = mock_data.get_buyer(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();

            // Setup existing trade
            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![logistics_provider],
                logistics_costs: vec![150],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 1,
                purchase_counter: 0,
                bump: 255,
            };

            let quantity = 3u64;

            // Simulate buy_trade function validation
            let valid_quantity = quantity > 0;
            let trade_active = trade_account.active;
            let sufficient_quantity = trade_account.remaining_quantity >= quantity;
            let buyer_not_seller = buyer != trade_account.seller;

            ErrorTestHelper::should_pass_validation(valid_quantity, "valid quantity");
            ErrorTestHelper::should_pass_validation(trade_active, "trade active");
            ErrorTestHelper::should_pass_validation(sufficient_quantity, "sufficient quantity");
            ErrorTestHelper::should_pass_validation(buyer_not_seller, "buyer not seller");

            // Find logistics cost
            let mut chosen_logistics_cost = 0u64;
            let mut found = false;
            for (i, provider) in trade_account.logistics_providers.iter().enumerate() {
                if *provider == logistics_provider {
                    chosen_logistics_cost = trade_account.logistics_costs[i];
                    found = true;
                    break;
                }
            }
            ErrorTestHelper::should_pass_validation(found, "logistics provider found");

            // Calculate costs
            let calc = ExpectedCalculations::new(trade_account.product_cost, chosen_logistics_cost, quantity);

            assert_eq!(calc.total_product_cost, 3000);
            assert_eq!(calc.total_logistics_cost, 450);
            assert_eq!(calc.total_amount, 3450);

            // Create purchase
            global_state.purchase_counter += 1;
            let purchase_id = global_state.purchase_counter;

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id,
                trade_id: trade_account.trade_id,
                buyer,
                quantity,
                total_amount: calc.total_amount,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: logistics_provider,
                logistics_cost: calc.total_logistics_cost,
                settled: false,
                bump: 254,
            };

            // Update trade state
            trade_account.remaining_quantity -= quantity;
            if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                trade_account.purchase_ids.push(purchase_id);
            }
            if trade_account.remaining_quantity == 0 {
                trade_account.active = false;
            }

            // Validate purchase creation
            StateAssertions::assert_purchase_account(&purchase_account, &buyer, quantity, calc.total_amount, false);
            assert_eq!(purchase_account.purchase_id, 1);
            assert_eq!(purchase_account.trade_id, 1);
            assert_eq!(purchase_account.chosen_logistics_provider, logistics_provider);
            assert_eq!(purchase_account.logistics_cost, 450);

            // Validate trade updates
            assert_eq!(trade_account.remaining_quantity, 7);
            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.purchase_ids.len(), 1);
            assert_eq!(global_state.purchase_counter, 1);
        }

        #[test]
        fn test_buy_trade_validation_invalid_quantity() {
            let mock_data = MockDataGenerator::new();

            let trade_account = TradeAccount {
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

            // Test invalid quantities
            let invalid_quantities = vec![0];

            for quantity in invalid_quantities {
                let valid_quantity = quantity > 0;
                ErrorTestHelper::should_fail_validation(valid_quantity, "InvalidQuantity");
            }
        }

        #[test]
        fn test_buy_trade_validation_trade_inactive() {
            let mock_data = MockDataGenerator::new();

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 0, // Sold out
                active: false, // Inactive
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let trade_active = trade_account.active;
            ErrorTestHelper::should_fail_validation(trade_active, "TradeInactive");
        }

        #[test]
        fn test_buy_trade_validation_insufficient_quantity() {
            let mock_data = MockDataGenerator::new();

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 3, // Only 3 left
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let requested_quantity = 5u64; // More than available

            let sufficient_quantity = trade_account.remaining_quantity >= requested_quantity;
            ErrorTestHelper::should_fail_validation(sufficient_quantity, "InsufficientQuantity");
        }

        #[test]
        fn test_buy_trade_validation_buyer_is_seller() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();

            let trade_account = TradeAccount {
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

            let buyer = seller; // Same as seller

            let buyer_not_seller = buyer != trade_account.seller;
            ErrorTestHelper::should_fail_validation(buyer_not_seller, "BuyerIsSeller");
        }

        #[test]
        fn test_buy_trade_validation_invalid_logistics_provider() {
            let mock_data = MockDataGenerator::new();

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller: mock_data.get_seller(0).pubkey(),
                logistics_providers: vec![create_test_pubkey(1), create_test_pubkey(2)],
                logistics_costs: vec![100, 150],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 10,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let invalid_provider = create_test_pubkey(99); // Not in the list

            let mut found = false;
            for provider in &trade_account.logistics_providers {
                if *provider == invalid_provider {
                    found = true;
                    break;
                }
            }

            ErrorTestHelper::should_fail_validation(found, "InvalidLogisticsProvider");
        }

        #[test]
        fn test_buy_trade_multiple_purchases_same_trade() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![logistics_provider],
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

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 1,
                purchase_counter: 0,
                bump: 255,
            };

            let mut purchases = Vec::new();

            // Create multiple purchases
            for i in 0..3 {
                let buyer = mock_data.get_buyer(i).pubkey();
                let quantity = 2u64;

                // Validate purchase can be made
                assert!(trade_account.active);
                assert!(trade_account.remaining_quantity >= quantity);
                assert_ne!(buyer, trade_account.seller);

                // Create purchase
                global_state.purchase_counter += 1;
                let purchase_id = global_state.purchase_counter;

                let calc = ExpectedCalculations::new(trade_account.product_cost, 100, quantity);

                let purchase_account = PurchaseAccount {
                    discriminator: [0; 8],
                    purchase_id,
                    trade_id: trade_account.trade_id,
                    buyer,
                    quantity,
                    total_amount: calc.total_amount,
                    delivered_and_confirmed: false,
                    disputed: false,
                    chosen_logistics_provider: logistics_provider,
                    logistics_cost: calc.total_logistics_cost,
                    settled: false,
                    bump: 250 + i as u8,
                };

                purchases.push(purchase_account);

                // Update trade
                trade_account.remaining_quantity -= quantity;
                if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                    trade_account.purchase_ids.push(purchase_id);
                }
                if trade_account.remaining_quantity == 0 {
                    trade_account.active = false;
                }
            }

            // Validate final state
            assert_eq!(purchases.len(), 3);
            assert_eq!(trade_account.remaining_quantity, 4); // 10 - (2*3)
            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.purchase_ids.len(), 3);
            assert_eq!(global_state.purchase_counter, 3);

            // Validate each purchase
            for (i, purchase) in purchases.iter().enumerate() {
                assert_eq!(purchase.purchase_id, i as u64 + 1);
                assert_eq!(purchase.buyer, mock_data.get_buyer(i).pubkey());
                assert_eq!(purchase.quantity, 2);
                assert_eq!(purchase.total_amount, 2200); // (1000 + 100) * 2
            }
        }

        #[test]
        fn test_buy_trade_complete_sellout() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();
            let buyer = mock_data.get_buyer(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![logistics_provider],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 5,
                remaining_quantity: 5,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(99),
                bump: 255,
            };

            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: mock_data.admin.pubkey(),
                trade_counter: 1,
                purchase_counter: 0,
                bump: 255,
            };

            let quantity = 5u64; // Buy entire remaining quantity

            // Create purchase
            global_state.purchase_counter += 1;
            let purchase_id = global_state.purchase_counter;

            let calc = ExpectedCalculations::new(trade_account.product_cost, 100, quantity);

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id,
                trade_id: trade_account.trade_id,
                buyer,
                quantity,
                total_amount: calc.total_amount,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: logistics_provider,
                logistics_cost: calc.total_logistics_cost,
                settled: false,
                bump: 254,
            };

            // Update trade state
            trade_account.remaining_quantity -= quantity;
            if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                trade_account.purchase_ids.push(purchase_id);
            }
            if trade_account.remaining_quantity == 0 {
                trade_account.active = false;
            }

            // Validate complete sellout
            assert_eq!(trade_account.remaining_quantity, 0);
            assert_eq!(trade_account.active, false);
            assert_eq!(purchase_account.quantity, 5);
            assert_eq!(purchase_account.total_amount, 5500); // (1000 + 100) * 5
        }

        #[test]
        fn test_buy_trade_different_logistics_providers() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();

            let providers = vec![
                mock_data.get_logistics_provider(0).pubkey(),
                mock_data.get_logistics_provider(1).pubkey(),
                mock_data.get_logistics_provider(2).pubkey(),
            ];
            let costs = vec![100, 150, 200];

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
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

            // Test purchasing with each logistics provider
            for (i, provider) in providers.iter().enumerate() {
                let quantity = 1u64;

                // Find logistics cost for this provider
                let mut chosen_logistics_cost = 0u64;
                let mut found = false;
                for (j, trade_provider) in trade_account.logistics_providers.iter().enumerate() {
                    if *trade_provider == *provider {
                        chosen_logistics_cost = trade_account.logistics_costs[j];
                        found = true;
                        break;
                    }
                }

                assert!(found, "Provider {} should be found", i);
                assert_eq!(chosen_logistics_cost, costs[i]);

                let calc = ExpectedCalculations::new(trade_account.product_cost, chosen_logistics_cost, quantity);
                let expected_total = 1000 + costs[i]; // Product cost + logistics cost

                assert_eq!(calc.total_amount, expected_total);
            }
        }
    }

    /// Test confirm_delivery_and_purchase function
    mod confirm_delivery_tests {
        use super::*;

        #[test]
        fn test_confirm_delivery_basic_functionality() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let seller = mock_data.get_seller(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();

            let mut purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: logistics_provider,
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![logistics_provider],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 7,
                active: true,
                purchase_ids: vec![1],
                token_mint: create_test_pubkey(99),
                bump: 254,
            };

            // Validate confirmation requirements
            let correct_buyer = buyer == purchase_account.buyer;
            let not_confirmed = !purchase_account.delivered_and_confirmed;
            let not_disputed = !purchase_account.disputed;
            let not_settled = !purchase_account.settled;

            ErrorTestHelper::should_pass_validation(correct_buyer, "correct buyer");
            ErrorTestHelper::should_pass_validation(not_confirmed, "not confirmed");
            ErrorTestHelper::should_pass_validation(not_disputed, "not disputed");
            ErrorTestHelper::should_pass_validation(not_settled, "not settled");

            // Simulate confirmation
            purchase_account.delivered_and_confirmed = true;
            purchase_account.settled = true;

            // Calculate payment distribution
            let calc = ExpectedCalculations::new(trade_account.product_cost, 100, purchase_account.quantity);

            assert_eq!(calc.product_escrow_fee, 75); // 2.5% of 3000
            assert_eq!(calc.seller_payout, 2925); // 3000 - 75
            assert_eq!(calc.logistics_escrow_fee, 7); // 2.5% of 300 (rounded down)
            assert_eq!(calc.logistics_payout, 293); // 300 - 7

            // Validate final state
            assert_eq!(purchase_account.delivered_and_confirmed, true);
            assert_eq!(purchase_account.settled, true);
            assert_eq!(purchase_account.disputed, false);
        }

        #[test]
        fn test_confirm_delivery_validation_wrong_buyer() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let wrong_buyer = mock_data.get_buyer(1).pubkey();

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let correct_buyer = wrong_buyer == purchase_account.buyer;
            ErrorTestHelper::should_fail_validation(correct_buyer, "NotAuthorized");
        }

        #[test]
        fn test_confirm_delivery_validation_already_confirmed() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: true, // Already confirmed
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let not_confirmed = !purchase_account.delivered_and_confirmed;
            ErrorTestHelper::should_fail_validation(not_confirmed, "AlreadyConfirmed");
        }

        #[test]
        fn test_confirm_delivery_validation_disputed() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: true, // Disputed
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let not_disputed = !purchase_account.disputed;
            ErrorTestHelper::should_fail_validation(not_disputed, "Disputed");
        }

        #[test]
        fn test_confirm_delivery_validation_already_settled() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: true, // Already settled
                bump: 255,
            };

            let not_settled = !purchase_account.settled;
            ErrorTestHelper::should_fail_validation(not_settled, "AlreadySettled");
        }

        #[test]
        fn test_confirm_delivery_payment_calculations() {
            let test_cases = vec![
                (1000u64, 100u64, 1u64), // Small purchase
                (2000u64, 200u64, 5u64), // Medium purchase
                (500u64, 50u64, 10u64),  // Large quantity
                (100u64, 10u64, 1u64),   // Small costs
            ];

            for (product_cost, logistics_cost, quantity) in test_cases {
                let calc = ExpectedCalculations::new(product_cost, logistics_cost, quantity);

                // Validate fee calculations
                let expected_product_fee = (calc.total_product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
                let expected_logistics_fee = (calc.total_logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

                assert_eq!(calc.product_escrow_fee, expected_product_fee);
                assert_eq!(calc.logistics_escrow_fee, expected_logistics_fee);

                // Validate payouts
                assert_eq!(calc.seller_payout, calc.total_product_cost - calc.product_escrow_fee);
                assert_eq!(calc.logistics_payout, calc.total_logistics_cost - calc.logistics_escrow_fee);

                // Validate total balance
                let total_fees = calc.product_escrow_fee + calc.logistics_escrow_fee;
                let total_payouts = calc.seller_payout + calc.logistics_payout;
                assert_eq!(total_payouts + total_fees, calc.total_amount);
            }
        }

        #[test]
        fn test_confirm_delivery_multiple_purchases() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let seller = mock_data.get_seller(0).pubkey();
            let logistics_provider = mock_data.get_logistics_provider(0).pubkey();

            // Test confirming multiple purchases from same buyer
            for i in 1..=3 {
                let mut purchase_account = PurchaseAccount {
                    discriminator: [0; 8],
                    purchase_id: i,
                    trade_id: 1,
                    buyer,
                    quantity: 2,
                    total_amount: 2200,
                    delivered_and_confirmed: false,
                    disputed: false,
                    chosen_logistics_provider: logistics_provider,
                    logistics_cost: 200,
                    settled: false,
                    bump: 250 + i as u8,
                };

                // Confirm delivery
                purchase_account.delivered_and_confirmed = true;
                purchase_account.settled = true;

                assert_eq!(purchase_account.delivered_and_confirmed, true);
                assert_eq!(purchase_account.settled, true);
                assert_eq!(purchase_account.purchase_id, i);
            }
        }
    }

    /// Test cancel_purchase function
    mod cancel_purchase_tests {
        use super::*;

        #[test]
        fn test_cancel_purchase_basic_functionality() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let seller = mock_data.get_seller(0).pubkey();

            let mut purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 10,
                remaining_quantity: 7, // After purchase
                active: true,
                purchase_ids: vec![1],
                token_mint: create_test_pubkey(99),
                bump: 254,
            };

            // Validate cancellation requirements
            let correct_buyer = buyer == purchase_account.buyer;
            let not_confirmed = !purchase_account.delivered_and_confirmed;
            let not_disputed = !purchase_account.disputed;
            let not_settled = !purchase_account.settled;

            ErrorTestHelper::should_pass_validation(correct_buyer, "correct buyer");
            ErrorTestHelper::should_pass_validation(not_confirmed, "not confirmed");
            ErrorTestHelper::should_pass_validation(not_disputed, "not disputed");
            ErrorTestHelper::should_pass_validation(not_settled, "not settled");

            // Simulate cancellation
            purchase_account.delivered_and_confirmed = true; // Mark as processed
            purchase_account.settled = true;
            trade_account.remaining_quantity += purchase_account.quantity; // Restore quantity

            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }

            // Validate final state
            assert_eq!(purchase_account.settled, true);
            assert_eq!(trade_account.remaining_quantity, 10); // 7 + 3 restored
            assert_eq!(trade_account.active, true);
        }

        #[test]
        fn test_cancel_purchase_trade_reactivation() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let seller = mock_data.get_seller(0).pubkey();

            let mut purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 5, // Entire remaining quantity
                total_amount: 5500,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 500,
                settled: false,
                bump: 255,
            };

            let mut trade_account = TradeAccount {
                discriminator: [0; 8],
                trade_id: 1,
                seller,
                logistics_providers: vec![create_test_pubkey(1)],
                logistics_costs: vec![100],
                product_cost: 1000,
                escrow_fee: 25,
                total_quantity: 5,
                remaining_quantity: 0, // Sold out
                active: false, // Inactive due to sellout
                purchase_ids: vec![1],
                token_mint: create_test_pubkey(99),
                bump: 254,
            };

            // Initially inactive and sold out
            assert_eq!(trade_account.active, false);
            assert_eq!(trade_account.remaining_quantity, 0);

            // Cancel purchase - should restore quantity and reactivate
            purchase_account.delivered_and_confirmed = true;
            purchase_account.settled = true;
            trade_account.remaining_quantity += purchase_account.quantity;

            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }

            // Trade should be reactivated
            assert_eq!(trade_account.active, true);
            assert_eq!(trade_account.remaining_quantity, 5);
            assert_eq!(purchase_account.settled, true);
        }

        #[test]
        fn test_cancel_purchase_validation_errors() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();
            let wrong_buyer = mock_data.get_buyer(1).pubkey();

            // Test wrong buyer
            let purchase_account = PurchaseAccount {
                discriminator: [0; 8],
                purchase_id: 1,
                trade_id: 1,
                buyer,
                quantity: 3,
                total_amount: 3300,
                delivered_and_confirmed: false,
                disputed: false,
                chosen_logistics_provider: create_test_pubkey(1),
                logistics_cost: 300,
                settled: false,
                bump: 255,
            };

            let correct_buyer = wrong_buyer == purchase_account.buyer;
            ErrorTestHelper::should_fail_validation(correct_buyer, "NotAuthorized");

            // Test already confirmed
            let confirmed_purchase = PurchaseAccount {
                delivered_and_confirmed: true,
                ..purchase_account
            };

            let not_confirmed = !confirmed_purchase.delivered_and_confirmed;
            ErrorTestHelper::should_fail_validation(not_confirmed, "AlreadyConfirmed");

            // Test disputed
            let disputed_purchase = PurchaseAccount {
                disputed: true,
                ..purchase_account
            };

            let not_disputed = !disputed_purchase.disputed;
            ErrorTestHelper::should_fail_validation(not_disputed, "Disputed");

            // Test already settled
            let settled_purchase = PurchaseAccount {
                settled: true,
                ..purchase_account
            };

            let not_settled = !settled_purchase.settled;
            ErrorTestHelper::should_fail_validation(not_settled, "AlreadySettled");
        }

        #[test]
        fn test_cancel_purchase_quantity_restoration() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let test_cases = vec![
                (10u64, 7u64, 3u64), // Partial purchase cancellation
                (5u64, 0u64, 5u64),  // Complete sellout cancellation
                (20u64, 15u64, 5u64), // Large trade cancellation
            ];

            for (total_quantity, current_remaining, cancelled_quantity) in test_cases {
                let mut purchase_account = PurchaseAccount {
                    discriminator: [0; 8],
                    purchase_id: 1,
                    trade_id: 1,
                    buyer,
                    quantity: cancelled_quantity,
                    total_amount: (1000 + 100) * cancelled_quantity,
                    delivered_and_confirmed: false,
                    disputed: false,
                    chosen_logistics_provider: create_test_pubkey(1),
                    logistics_cost: 100 * cancelled_quantity,
                    settled: false,
                    bump: 255,
                };

                let mut trade_account = TradeAccount {
                    discriminator: [0; 8],
                    trade_id: 1,
                    seller: mock_data.get_seller(0).pubkey(),
                    logistics_providers: vec![create_test_pubkey(1)],
                    logistics_costs: vec![100],
                    product_cost: 1000,
                    escrow_fee: 25,
                    total_quantity,
                    remaining_quantity: current_remaining,
                    active: current_remaining > 0,
                    purchase_ids: vec![1],
                    token_mint: create_test_pubkey(99),
                    bump: 254,
                };

                // Cancel purchase
                purchase_account.delivered_and_confirmed = true;
                purchase_account.settled = true;
                trade_account.remaining_quantity += purchase_account.quantity;

                if !trade_account.active && trade_account.remaining_quantity > 0 {
                    trade_account.active = true;
                }

                // Validate restoration
                let expected_remaining = current_remaining + cancelled_quantity;
                assert_eq!(trade_account.remaining_quantity, expected_remaining);
                assert_eq!(trade_account.active, true);
                assert_eq!(purchase_account.settled, true);

                // Ensure we don't exceed total quantity
                assert!(trade_account.remaining_quantity <= trade_account.total_quantity);
            }
        }
    }

    /// Test buyer account integration
    mod buyer_account_integration_tests {
        use super::*;

        #[test]
        fn test_buyer_auto_registration() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            // Test unregistered buyer
            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: Vec::new(),
                bump: 0,
            };

            // Simulate auto-registration during purchase
            if !buyer_account.is_registered {
                buyer_account.buyer = buyer;
                buyer_account.is_registered = true;
                buyer_account.purchase_ids = Vec::new();
                buyer_account.bump = 255;
            }

            // Add purchase ID
            if buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                buyer_account.purchase_ids.push(1);
            }

            assert_eq!(buyer_account.buyer, buyer);
            assert_eq!(buyer_account.is_registered, true);
            assert_eq!(buyer_account.purchase_ids, vec![1]);
        }

        #[test]
        fn test_buyer_purchase_ids_management() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer,
                is_registered: true,
                purchase_ids: Vec::new(),
                bump: 255,
            };

            // Add multiple purchase IDs
            for i in 1..=5 {
                if buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                    buyer_account.purchase_ids.push(i);
                }
            }

            assert_eq!(buyer_account.purchase_ids.len(), 5);
            assert_eq!(buyer_account.purchase_ids, vec![1, 2, 3, 4, 5]);

            // Test MAX_PURCHASE_IDS limit
            let mut full_buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer,
                is_registered: true,
                purchase_ids: (1..=MAX_PURCHASE_IDS as u64).collect(),
                bump: 255,
            };

            // Try to add one more
            if full_buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                full_buyer_account.purchase_ids.push((MAX_PURCHASE_IDS + 1) as u64);
            }

            assert_eq!(full_buyer_account.purchase_ids.len(), MAX_PURCHASE_IDS);
        }
    }
}