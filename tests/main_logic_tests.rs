#[cfg(test)]
mod main_logic_tests {
    use anchor_lang::prelude::*;
    use std::collections::BTreeMap;

    // Import all types and constants from main.rs
    use dezenmart_rust_smart_contract::*;
    use dezenmart_rust_smart_contract::dezenmart_logistics::*;

    // Helper function to create test pubkeys
    fn create_test_pubkey(seed: u8) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        Pubkey::new_from_array(bytes)
    }

    #[test]
    fn test_advanced_buy_trade_scenarios() {
        let seller = create_test_pubkey(1);
        let buyer1 = create_test_pubkey(2);
        let buyer2 = create_test_pubkey(3);
        let logistics_provider1 = create_test_pubkey(4);
        let logistics_provider2 = create_test_pubkey(5);

        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: create_test_pubkey(0),
            trade_counter: 0,
            purchase_counter: 0,
            bump: 255,
        };

        global_state.trade_counter += 1;

        let mut trade_account = TradeAccount {
            discriminator: [0; 8],
            trade_id: 1,
            seller,
            logistics_providers: vec![logistics_provider1, logistics_provider2],
            logistics_costs: vec![100, 150],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 10,
            active: true,
            purchase_ids: Vec::new(),
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // First buyer purchases 4 units with provider 1
        global_state.purchase_counter += 1;
        let purchase1_id = global_state.purchase_counter;

        let purchase1 = PurchaseAccount {
            discriminator: [0; 8],
            purchase_id: purchase1_id,
            trade_id: 1,
            buyer: buyer1,
            quantity: 4,
            total_amount: (1000 + 100) * 4, // 4400
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: logistics_provider1,
            logistics_cost: 100 * 4, // 400
            settled: false,
            bump: 255,
        };

        // Update trade
        trade_account.remaining_quantity -= 4;
        trade_account.purchase_ids.push(purchase1_id);

        // Second buyer purchases 6 units with provider 2 (more expensive)
        global_state.purchase_counter += 1;
        let purchase2_id = global_state.purchase_counter;

        let purchase2 = PurchaseAccount {
            discriminator: [0; 8],
            purchase_id: purchase2_id,
            trade_id: 1,
            buyer: buyer2,
            quantity: 6,
            total_amount: (1000 + 150) * 6, // 6900
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: logistics_provider2,
            logistics_cost: 150 * 6, // 900
            settled: false,
            bump: 255,
        };

        // Update trade - should become inactive
        trade_account.remaining_quantity -= 6;
        trade_account.purchase_ids.push(purchase2_id);

        if trade_account.remaining_quantity == 0 {
            trade_account.active = false;
        }

        // Verify state
        assert_eq!(trade_account.remaining_quantity, 0);
        assert_eq!(trade_account.active, false);
        assert_eq!(trade_account.purchase_ids.len(), 2);
        assert_eq!(purchase1.total_amount, 4400);
        assert_eq!(purchase2.total_amount, 6900);
        assert_eq!(global_state.purchase_counter, 2);
    }

    #[test]
    fn test_dispute_resolution_scenarios() {
        let buyer = create_test_pubkey(1);
        let seller = create_test_pubkey(2);
        let logistics_provider = create_test_pubkey(3);

        let mut purchase_account = PurchaseAccount {
            discriminator: [0; 8],
            purchase_id: 1,
            trade_id: 1,
            buyer,
            quantity: 5,
            total_amount: 5500,
            delivered_and_confirmed: false,
            disputed: true,
            chosen_logistics_provider: logistics_provider,
            logistics_cost: 500,
            settled: false,
            bump: 255,
        };

        let mut trade_account = TradeAccount {
            discriminator: [0; 8],
            trade_id: 1,
            seller,
            logistics_providers: vec![logistics_provider],
            logistics_costs: vec![100],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 5,
            active: true,
            purchase_ids: vec![1],
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Test scenario 1: Buyer wins dispute (refund)
        let winner = buyer;
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        if winner == purchase_account.buyer {
            // Restore quantity to trade
            trade_account.remaining_quantity += purchase_account.quantity;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }
        }

        assert_eq!(trade_account.remaining_quantity, 10); // 5 + 5 restored
        assert_eq!(trade_account.active, true);
        assert_eq!(purchase_account.settled, true);

        // Reset for scenario 2: Seller wins dispute
        trade_account.remaining_quantity = 5;
        purchase_account.settled = false;
        purchase_account.delivered_and_confirmed = false;

        let winner = seller;
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        // Calculate payments for seller and logistics
        let product_escrow_fee = (trade_account.product_cost * ESCROW_FEE_PERCENT * purchase_account.quantity) / BASIS_POINTS;
        let seller_amount = (trade_account.product_cost * purchase_account.quantity) - product_escrow_fee;
        let logistics_escrow_fee = (purchase_account.logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let logistics_payout = purchase_account.logistics_cost - logistics_escrow_fee;

        assert_eq!(product_escrow_fee, 125); // 2.5% of 5000
        assert_eq!(seller_amount, 4875); // 5000 - 125
        assert_eq!(logistics_escrow_fee, 12); // 2.5% of 500
        assert_eq!(logistics_payout, 488); // 500 - 12
        assert_eq!(purchase_account.settled, true);

        // Reset for scenario 3: Logistics provider wins dispute
        purchase_account.settled = false;
        purchase_account.delivered_and_confirmed = false;

        let winner = logistics_provider;
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        // Same calculation as seller wins
        assert_eq!(purchase_account.settled, true);
    }

    #[test]
    fn test_multiple_purchases_same_buyer() {
        let buyer = create_test_pubkey(1);
        let seller = create_test_pubkey(2);
        let logistics_provider = create_test_pubkey(3);

        let mut buyer_account = BuyerAccount {
            discriminator: [0; 8],
            buyer,
            is_registered: true,
            purchase_ids: Vec::new(),
            bump: 255,
        };

        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: create_test_pubkey(0),
            trade_counter: 1,
            purchase_counter: 0,
            bump: 255,
        };

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
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Make multiple purchases
        for i in 1..=3 {
            global_state.purchase_counter += 1;
            let purchase_id = global_state.purchase_counter;

            let quantity = 2u64;
            trade_account.remaining_quantity -= quantity;

            if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                trade_account.purchase_ids.push(purchase_id);
            }

            if buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                buyer_account.purchase_ids.push(purchase_id);
            }

            if trade_account.remaining_quantity == 0 {
                trade_account.active = false;
            }
        }

        assert_eq!(trade_account.remaining_quantity, 4); // 10 - (2*3)
        assert_eq!(trade_account.active, true);
        assert_eq!(trade_account.purchase_ids.len(), 3);
        assert_eq!(buyer_account.purchase_ids.len(), 3);
        assert_eq!(global_state.purchase_counter, 3);
    }

    #[test]
    fn test_cancel_purchase_restore_trade() {
        let buyer = create_test_pubkey(1);
        let seller = create_test_pubkey(2);
        let logistics_provider = create_test_pubkey(3);

        let mut purchase_account = PurchaseAccount {
            discriminator: [0; 8],
            purchase_id: 1,
            trade_id: 1,
            buyer,
            quantity: 8,
            total_amount: 8800,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: logistics_provider,
            logistics_cost: 800,
            settled: false,
            bump: 255,
        };

        let mut trade_account = TradeAccount {
            discriminator: [0; 8],
            trade_id: 1,
            seller,
            logistics_providers: vec![logistics_provider],
            logistics_costs: vec![100],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 2, // Only 2 left after purchase
            active: true,
            purchase_ids: vec![1],
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Cancel the purchase
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;
        trade_account.remaining_quantity += purchase_account.quantity;

        if !trade_account.active && trade_account.remaining_quantity > 0 {
            trade_account.active = true;
        }

        assert_eq!(purchase_account.settled, true);
        assert_eq!(trade_account.remaining_quantity, 10); // 2 + 8 restored
        assert_eq!(trade_account.active, true);
    }

    #[test]
    fn test_inactive_trade_reactivation() {
        let seller = create_test_pubkey(1);
        let logistics_provider = create_test_pubkey(2);

        let mut trade_account = TradeAccount {
            discriminator: [0; 8],
            trade_id: 1,
            seller,
            logistics_providers: vec![logistics_provider],
            logistics_costs: vec![100],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 0, // Sold out
            active: false, // Inactive
            purchase_ids: vec![1, 2],
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Simulate refund/cancellation that restores quantity
        let restored_quantity = 5u64;
        trade_account.remaining_quantity += restored_quantity;

        if !trade_account.active && trade_account.remaining_quantity > 0 {
            trade_account.active = true;
        }

        assert_eq!(trade_account.remaining_quantity, 5);
        assert_eq!(trade_account.active, true);
    }

    #[test]
    fn test_escrow_fee_withdrawal_calculations() {
        // Simulate multiple completed transactions generating fees
        let product_costs = vec![1000u64, 2000u64, 1500u64];
        let logistics_costs = vec![200u64, 300u64, 250u64];
        let quantities = vec![2u64, 1u64, 3u64];

        let mut total_product_fees = 0u64;
        let mut total_logistics_fees = 0u64;

        for i in 0..3 {
            let product_fee = (product_costs[i] * ESCROW_FEE_PERCENT * quantities[i]) / BASIS_POINTS;
            let logistics_fee = (logistics_costs[i] * quantities[i] * ESCROW_FEE_PERCENT) / BASIS_POINTS;

            total_product_fees += product_fee;
            total_logistics_fees += logistics_fee;
        }

        let total_fees = total_product_fees + total_logistics_fees;

        // Expected calculations:
        // Product fees: (1000*250*2 + 2000*250*1 + 1500*250*3) / 10000 = (500000 + 500000 + 1125000) / 10000 = 212.5 = 212
        // Logistics fees: (200*2*250)/10000=10 + (300*1*250)/10000=7 + (250*3*250)/10000=18 = 35 (integer division)
        assert_eq!(total_product_fees, 212);
        assert_eq!(total_logistics_fees, 35);
        assert_eq!(total_fees, 247);

        // Test withdrawal validation
        let escrow_balance = total_fees;
        assert!(escrow_balance > 0); // Should be valid for withdrawal
    }

    #[test]
    fn test_maximum_limits_behavior() {
        // Test MAX_LOGISTICS_PROVIDERS limit
        let mut providers = Vec::new();
        let mut costs = Vec::new();

        for i in 0..MAX_LOGISTICS_PROVIDERS {
            providers.push(create_test_pubkey(i as u8));
            costs.push(100u64 + i as u64 * 10); // Varying costs
        }

        assert_eq!(providers.len(), MAX_LOGISTICS_PROVIDERS);
        assert_eq!(costs.len(), MAX_LOGISTICS_PROVIDERS);

        // Test that we don't exceed the limit
        let result = providers.len() <= MAX_LOGISTICS_PROVIDERS;
        assert!(result);

        // Test MAX_PURCHASE_IDS limit
        let mut purchase_ids: Vec<u64> = Vec::new();
        let mut trade_account = TradeAccount {
            discriminator: [0; 8],
            trade_id: 1,
            seller: create_test_pubkey(1),
            logistics_providers: providers.clone(),
            logistics_costs: costs.clone(),
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 1000,
            remaining_quantity: 1000,
            active: true,
            purchase_ids: Vec::new(),
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Simulate adding purchases up to the limit
        for i in 1..=MAX_PURCHASE_IDS {
            if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                trade_account.purchase_ids.push(i as u64);
            }
        }

        assert_eq!(trade_account.purchase_ids.len(), MAX_PURCHASE_IDS);

        // Try to add one more (should not be added)
        if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
            trade_account.purchase_ids.push((MAX_PURCHASE_IDS + 1) as u64);
        }

        assert_eq!(trade_account.purchase_ids.len(), MAX_PURCHASE_IDS); // Still at limit
    }

    #[test]
    fn test_complex_payment_calculations() {
        let product_cost = 2500u64;
        let logistics_cost = 350u64;
        let quantity = 7u64;

        // Calculate total costs
        let total_product_cost = product_cost * quantity; // 17500
        let total_logistics_cost = logistics_cost * quantity; // 2450
        let total_amount = total_product_cost + total_logistics_cost; // 19950

        // Calculate escrow fees
        let product_escrow_fee = (total_product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS; // 437.5 = 437
        let logistics_escrow_fee = (total_logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS; // 61.25 = 61

        // Calculate final payouts
        let seller_amount = total_product_cost - product_escrow_fee; // 17063
        let logistics_amount = total_logistics_cost - logistics_escrow_fee; // 2389

        assert_eq!(total_amount, 19950);
        assert_eq!(product_escrow_fee, 437);
        assert_eq!(logistics_escrow_fee, 61);
        assert_eq!(seller_amount, 17063);
        assert_eq!(logistics_amount, 2389);

        // Verify total escrow fees collected
        let total_fees = product_escrow_fee + logistics_escrow_fee;
        assert_eq!(total_fees, 498);

        // Verify total payouts + fees = total amount
        let total_payouts = seller_amount + logistics_amount + total_fees;
        assert_eq!(total_payouts, total_amount);
    }

    #[test]
    fn test_edge_case_quantity_calculations() {
        // Test with quantity = 1
        let product_cost = 100u64;
        let logistics_cost = 10u64;
        let quantity = 1u64;

        let total_cost = (product_cost + logistics_cost) * quantity;
        let product_fee = (product_cost * quantity * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let logistics_fee = (logistics_cost * quantity * ESCROW_FEE_PERCENT) / BASIS_POINTS;

        assert_eq!(total_cost, 110);
        assert_eq!(product_fee, 2); // 2.5% of 100 = 2.5, rounded down to 2
        assert_eq!(logistics_fee, 0); // 2.5% of 10 = 0.25, rounded down to 0

        // Test with large quantities
        let large_quantity = 1000u64;
        let large_total = (product_cost + logistics_cost) * large_quantity;
        let large_product_fee = (product_cost * large_quantity * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let large_logistics_fee = (logistics_cost * large_quantity * ESCROW_FEE_PERCENT) / BASIS_POINTS;

        assert_eq!(large_total, 110000);
        assert_eq!(large_product_fee, 2500); // 2.5% of 100000
        assert_eq!(large_logistics_fee, 250); // 2.5% of 10000
    }

    #[test]
    fn test_complete_marketplace_simulation() {
        // Simulate a complete marketplace with multiple trades and users
        let admin = create_test_pubkey(0);
        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin,
            trade_counter: 0,
            purchase_counter: 0,
            bump: 255,
        };

        // Create multiple logistics providers
        let providers = vec![
            create_test_pubkey(10),
            create_test_pubkey(11),
            create_test_pubkey(12),
        ];

        // Create multiple sellers and trades
        let mut trades = Vec::new();
        for i in 1..=3 {
            global_state.trade_counter += 1;
            let trade = TradeAccount {
            discriminator: [0; 8],
                trade_id: global_state.trade_counter,
                seller: create_test_pubkey(i),
                logistics_providers: providers.clone(),
                logistics_costs: vec![100 + i as u64 * 10, 150 + i as u64 * 10, 200 + i as u64 * 10],
                product_cost: 1000 + i as u64 * 100,
                escrow_fee: ((1000 + i as u64 * 100) * ESCROW_FEE_PERCENT) / BASIS_POINTS,
                total_quantity: 20,
                remaining_quantity: 20,
                active: true,
                purchase_ids: Vec::new(),
                token_mint: create_test_pubkey(20 + i),
                bump: 255,
            };
            trades.push(trade);
        }

        // Create buyers and make purchases
        let buyers = vec![create_test_pubkey(30), create_test_pubkey(31), create_test_pubkey(32)];
        let mut purchases = Vec::new();

        for (buyer_idx, buyer) in buyers.iter().enumerate() {
            for (trade_idx, trade) in trades.iter_mut().enumerate() {
                global_state.purchase_counter += 1;
                let quantity = 2u64 + buyer_idx as u64;

                let chosen_provider_idx = buyer_idx % trade.logistics_providers.len();
                let chosen_provider = trade.logistics_providers[chosen_provider_idx];
                let logistics_cost = trade.logistics_costs[chosen_provider_idx];

                let total_amount = (trade.product_cost + logistics_cost) * quantity;

                let purchase = PurchaseAccount {
            discriminator: [0; 8],
                    purchase_id: global_state.purchase_counter,
                    trade_id: trade.trade_id,
                    buyer: *buyer,
                    quantity,
                    total_amount,
                    delivered_and_confirmed: false,
                    disputed: false,
                    chosen_logistics_provider: chosen_provider,
                    logistics_cost: logistics_cost * quantity,
                    settled: false,
                    bump: 255,
                };

                trade.remaining_quantity -= quantity;
                if trade.purchase_ids.len() < MAX_PURCHASE_IDS {
                    trade.purchase_ids.push(purchase.purchase_id);
                }

                if trade.remaining_quantity == 0 {
                    trade.active = false;
                }

                purchases.push(purchase);
            }
        }

        // Verify marketplace state
        assert_eq!(global_state.trade_counter, 3);
        assert_eq!(global_state.purchase_counter, 9); // 3 buyers * 3 trades
        assert_eq!(purchases.len(), 9);

        // Verify trade states
        for trade in &trades {
            let expected_remaining = 20 - (2 + 3 + 4); // 11
            assert_eq!(trade.remaining_quantity, expected_remaining);
            assert_eq!(trade.active, true); // Still active since not sold out
            assert_eq!(trade.purchase_ids.len(), 3);
        }

        // Simulate some confirmations and disputes
        let mut total_confirmed = 0;
        let mut total_disputed = 0;

        for (i, purchase) in purchases.iter().enumerate() {
            if i % 3 == 0 {
                // Confirm delivery (every 3rd purchase)
                total_confirmed += 1;
            } else if i % 5 == 0 {
                // Raise dispute (every 5th purchase)
                total_disputed += 1;
            }
        }

        assert!(total_confirmed > 0);
        assert!(total_disputed > 0);
    }
}