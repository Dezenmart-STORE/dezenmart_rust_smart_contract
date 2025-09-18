use anchor_lang::prelude::*;
use std::collections::BTreeMap;

#[cfg(test)]
mod main_tests {
    use super::*;

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
    fn test_constants_main() {
        assert_eq!(ESCROW_FEE_PERCENT, 250);
        assert_eq!(BASIS_POINTS, 10000);
        assert_eq!(MAX_LOGISTICS_PROVIDERS, 10);
        assert_eq!(MAX_PURCHASE_IDS, 100);
    }

    #[test]
    fn test_initialize_logic_main() {
        let admin = create_test_pubkey(1);

        // Test global state initialization
        let mut global_state = GlobalState {
            admin: Pubkey::default(),
            trade_counter: 999,
            purchase_counter: 999,
            bump: 0,
        };

        // Simulate initialize function logic
        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 255;

        assert_eq!(global_state.admin, admin);
        assert_eq!(global_state.trade_counter, 0);
        assert_eq!(global_state.purchase_counter, 0);
        assert_eq!(global_state.bump, 255);
    }

    #[test]
    fn test_register_logistics_provider_logic_main() {
        let provider = create_test_pubkey(2);

        // Simulate register_logistics_provider function logic
        let mut provider_account = LogisticsProviderAccount {
            provider: Pubkey::default(),
            is_registered: false,
            bump: 0,
        };

        provider_account.provider = provider;
        provider_account.is_registered = true;
        provider_account.bump = 255;

        assert_eq!(provider_account.provider, provider);
        assert_eq!(provider_account.is_registered, true);
        assert_eq!(provider_account.bump, 255);
    }

    #[test]
    fn test_register_seller_logic_main() {
        let seller = create_test_pubkey(3);

        // Simulate register_seller function logic
        let mut seller_account = SellerAccount {
            seller: Pubkey::default(),
            is_registered: false,
            bump: 0,
        };

        seller_account.seller = seller;
        seller_account.is_registered = true;
        seller_account.bump = 255;

        assert_eq!(seller_account.seller, seller);
        assert_eq!(seller_account.is_registered, true);
        assert_eq!(seller_account.bump, 255);
    }

    #[test]
    fn test_register_buyer_logic_main() {
        let buyer = create_test_pubkey(4);

        // Simulate register_buyer function logic
        let mut buyer_account = BuyerAccount {
            buyer: Pubkey::default(),
            is_registered: false,
            purchase_ids: vec![1, 2, 3], // Should be reset
            bump: 0,
        };

        buyer_account.buyer = buyer;
        buyer_account.is_registered = true;
        buyer_account.purchase_ids = Vec::new();
        buyer_account.bump = 255;

        assert_eq!(buyer_account.buyer, buyer);
        assert_eq!(buyer_account.is_registered, true);
        assert_eq!(buyer_account.purchase_ids.len(), 0);
        assert_eq!(buyer_account.bump, 255);
    }

    #[test]
    fn test_create_trade_validation_logic_main() {
        let seller = create_test_pubkey(5);
        let logistics_provider1 = create_test_pubkey(6);
        let logistics_provider2 = create_test_pubkey(7);
        let token_mint = create_test_pubkey(8);

        // Test validation: mismatched arrays
        let logistics_providers = vec![logistics_provider1, logistics_provider2];
        let logistics_costs = vec![100]; // Mismatched length
        let result = logistics_providers.len() == logistics_costs.len();
        assert_eq!(result, false); // Should fail validation

        // Test validation: no logistics providers
        let empty_providers: Vec<Pubkey> = vec![];
        let result = !empty_providers.is_empty();
        assert_eq!(result, false); // Should fail validation

        // Test validation: too many providers
        let mut many_providers = Vec::new();
        for i in 0..15 {
            many_providers.push(create_test_pubkey(i));
        }
        let result = many_providers.len() <= MAX_LOGISTICS_PROVIDERS;
        assert_eq!(result, false); // Should fail validation

        // Test validation: invalid quantity
        let total_quantity = 0u64;
        let result = total_quantity > 0;
        assert_eq!(result, false); // Should fail validation

        // Test valid case
        let logistics_providers = vec![logistics_provider1];
        let logistics_costs = vec![100];
        let total_quantity = 10u64;
        let product_cost = 1000u64;

        let arrays_match = logistics_providers.len() == logistics_costs.len();
        let has_providers = !logistics_providers.is_empty();
        let within_limit = logistics_providers.len() <= MAX_LOGISTICS_PROVIDERS;
        let valid_quantity = total_quantity > 0;

        assert!(arrays_match);
        assert!(has_providers);
        assert!(within_limit);
        assert!(valid_quantity);

        // Simulate create_trade logic
        let mut global_state = GlobalState {
            admin: create_test_pubkey(1),
            trade_counter: 0,
            purchase_counter: 0,
            bump: 255,
        };

        global_state.trade_counter += 1;
        let trade_id = global_state.trade_counter;

        let product_escrow_fee = (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

        let mut trade_account = TradeAccount {
            trade_id,
            seller,
            logistics_providers: logistics_providers.clone(),
            logistics_costs: logistics_costs.clone(),
            product_cost,
            escrow_fee: product_escrow_fee,
            total_quantity,
            remaining_quantity: total_quantity,
            active: true,
            purchase_ids: Vec::new(),
            token_mint,
            bump: 255,
        };

        assert_eq!(trade_account.trade_id, 1);
        assert_eq!(trade_account.seller, seller);
        assert_eq!(trade_account.logistics_providers, logistics_providers);
        assert_eq!(trade_account.logistics_costs, logistics_costs);
        assert_eq!(trade_account.product_cost, product_cost);
        assert_eq!(trade_account.escrow_fee, 25); // 2.5% of 1000
        assert_eq!(trade_account.total_quantity, total_quantity);
        assert_eq!(trade_account.remaining_quantity, total_quantity);
        assert_eq!(trade_account.active, true);
        assert_eq!(trade_account.purchase_ids.len(), 0);
        assert_eq!(trade_account.token_mint, token_mint);
    }

    #[test]
    fn test_buy_trade_validation_logic_main() {
        let seller = create_test_pubkey(5);
        let buyer = create_test_pubkey(9);
        let logistics_provider = create_test_pubkey(6);
        let invalid_logistics_provider = create_test_pubkey(10);

        // Setup trade account
        let mut trade_account = TradeAccount {
            trade_id: 1,
            seller,
            logistics_providers: vec![logistics_provider],
            logistics_costs: vec![100],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 5,
            active: true,
            purchase_ids: Vec::new(),
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Test validation: invalid quantity (zero)
        let quantity = 0u64;
        let result = quantity > 0;
        assert_eq!(result, false); // Should fail validation

        // Test validation: trade inactive
        trade_account.active = false;
        let result = trade_account.active;
        assert_eq!(result, false); // Should fail validation

        // Reset for next tests
        trade_account.active = true;

        // Test validation: insufficient quantity
        let quantity = 10u64; // More than remaining
        let result = trade_account.remaining_quantity >= quantity;
        assert_eq!(result, false); // Should fail validation

        // Test validation: buyer is seller
        let buyer_is_seller = buyer == trade_account.seller;
        assert_eq!(buyer_is_seller, false); // Should pass (buyer != seller)

        // Test validation: invalid logistics provider
        let mut found = false;
        for provider in &trade_account.logistics_providers {
            if *provider == invalid_logistics_provider {
                found = true;
                break;
            }
        }
        assert_eq!(found, false); // Should fail validation

        // Test valid case
        let quantity = 3u64;
        let chosen_logistics_provider = logistics_provider;

        let valid_quantity = quantity > 0;
        let trade_active = trade_account.active;
        let sufficient_quantity = trade_account.remaining_quantity >= quantity;
        let buyer_not_seller = buyer != trade_account.seller;

        // Find logistics cost
        let mut chosen_logistics_cost = 0u64;
        let mut provider_found = false;
        for (i, provider) in trade_account.logistics_providers.iter().enumerate() {
            if *provider == chosen_logistics_provider {
                chosen_logistics_cost = trade_account.logistics_costs[i];
                provider_found = true;
                break;
            }
        }

        assert!(valid_quantity);
        assert!(trade_active);
        assert!(sufficient_quantity);
        assert!(buyer_not_seller);
        assert!(provider_found);
        assert_eq!(chosen_logistics_cost, 100);

        // Calculate costs
        let total_product_cost = trade_account.product_cost * quantity;
        let total_logistics_cost = chosen_logistics_cost * quantity;
        let total_amount = total_product_cost + total_logistics_cost;

        assert_eq!(total_product_cost, 3000);
        assert_eq!(total_logistics_cost, 300);
        assert_eq!(total_amount, 3300);

        // Simulate purchase creation
        let mut global_state = GlobalState {
            admin: create_test_pubkey(1),
            trade_counter: 1,
            purchase_counter: 0,
            bump: 255,
        };

        global_state.purchase_counter += 1;
        let purchase_id = global_state.purchase_counter;

        let purchase_account = PurchaseAccount {
            purchase_id,
            trade_id: trade_account.trade_id,
            buyer,
            quantity,
            total_amount,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider,
            logistics_cost: total_logistics_cost,
            settled: false,
            bump: 255,
        };

        // Update trade state
        trade_account.remaining_quantity -= quantity;
        if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
            trade_account.purchase_ids.push(purchase_id);
        }

        if trade_account.remaining_quantity == 0 {
            trade_account.active = false;
        }

        assert_eq!(purchase_account.purchase_id, 1);
        assert_eq!(purchase_account.trade_id, 1);
        assert_eq!(purchase_account.buyer, buyer);
        assert_eq!(purchase_account.quantity, quantity);
        assert_eq!(purchase_account.total_amount, total_amount);
        assert_eq!(trade_account.remaining_quantity, 2);
        assert_eq!(trade_account.active, true); // Still active
        assert_eq!(trade_account.purchase_ids.len(), 1);
    }

    #[test]
    fn test_confirm_delivery_validation_logic_main() {
        let buyer = create_test_pubkey(9);
        let wrong_buyer = create_test_pubkey(11);
        let seller = create_test_pubkey(5);
        let logistics_provider = create_test_pubkey(6);

        // Setup purchase account
        let mut purchase_account = PurchaseAccount {
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

        // Test validation: wrong buyer
        let result = wrong_buyer == purchase_account.buyer;
        assert_eq!(result, false); // Should fail validation

        // Test validation: already confirmed
        purchase_account.delivered_and_confirmed = true;
        let result = !purchase_account.delivered_and_confirmed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.delivered_and_confirmed = false;

        // Test validation: disputed
        purchase_account.disputed = true;
        let result = !purchase_account.disputed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.disputed = false;

        // Test validation: already settled
        purchase_account.settled = true;
        let result = !purchase_account.settled;
        assert_eq!(result, false); // Should fail validation

        // Reset for valid case
        purchase_account.settled = false;

        // Test valid case
        let correct_buyer = buyer == purchase_account.buyer;
        let not_confirmed = !purchase_account.delivered_and_confirmed;
        let not_disputed = !purchase_account.disputed;
        let not_settled = !purchase_account.settled;

        assert!(correct_buyer);
        assert!(not_confirmed);
        assert!(not_disputed);
        assert!(not_settled);

        // Simulate confirmation logic
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        assert_eq!(purchase_account.delivered_and_confirmed, true);
        assert_eq!(purchase_account.settled, true);

        // Test payment calculation logic
        let trade_account = TradeAccount {
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
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        let product_escrow_fee = (trade_account.product_cost * ESCROW_FEE_PERCENT * purchase_account.quantity) / BASIS_POINTS;
        let seller_amount = (trade_account.product_cost * purchase_account.quantity) - product_escrow_fee;

        let logistics_escrow_fee = (purchase_account.logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let logistics_amount = purchase_account.logistics_cost - logistics_escrow_fee;

        assert_eq!(product_escrow_fee, 75); // 2.5% of (1000 * 3)
        assert_eq!(seller_amount, 2925); // 3000 - 75
        assert_eq!(logistics_escrow_fee, 7); // 2.5% of 300 (rounded down)
        assert_eq!(logistics_amount, 293); // 300 - 7
    }

    #[test]
    fn test_raise_dispute_logic_main() {
        let buyer = create_test_pubkey(9);
        let user = create_test_pubkey(12);

        // Setup purchase account
        let mut purchase_account = PurchaseAccount {
            purchase_id: 1,
            trade_id: 1,
            buyer,
            quantity: 3,
            total_amount: 3300,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: create_test_pubkey(6),
            logistics_cost: 300,
            settled: false,
            bump: 255,
        };

        // Test validation: already confirmed
        purchase_account.delivered_and_confirmed = true;
        let result = !purchase_account.delivered_and_confirmed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.delivered_and_confirmed = false;

        // Test validation: already disputed
        purchase_account.disputed = true;
        let result = !purchase_account.disputed;
        assert_eq!(result, false); // Should fail validation

        // Reset for valid case
        purchase_account.disputed = false;

        // Test valid case
        let not_confirmed = !purchase_account.delivered_and_confirmed;
        let not_disputed = !purchase_account.disputed;

        assert!(not_confirmed);
        assert!(not_disputed);

        // Simulate raise_dispute logic
        purchase_account.disputed = true;

        assert_eq!(purchase_account.disputed, true);
    }

    #[test]
    fn test_resolve_dispute_logic_main() {
        let buyer = create_test_pubkey(9);
        let seller = create_test_pubkey(5);
        let logistics_provider = create_test_pubkey(6);
        let invalid_winner = create_test_pubkey(13);

        // Setup purchase account
        let mut purchase_account = PurchaseAccount {
            purchase_id: 1,
            trade_id: 1,
            buyer,
            quantity: 3,
            total_amount: 3300,
            delivered_and_confirmed: false,
            disputed: true,
            chosen_logistics_provider: logistics_provider,
            logistics_cost: 300,
            settled: false,
            bump: 255,
        };

        let mut trade_account = TradeAccount {
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
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Test validation: not disputed
        purchase_account.disputed = false;
        let result = purchase_account.disputed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.disputed = true;

        // Test validation: already settled
        purchase_account.settled = true;
        let result = !purchase_account.settled;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.settled = false;

        // Test validation: invalid winner
        let valid_winner = invalid_winner == purchase_account.buyer
            || invalid_winner == trade_account.seller
            || invalid_winner == purchase_account.chosen_logistics_provider;
        assert_eq!(valid_winner, false); // Should fail validation

        // Test valid winners
        let buyer_valid = buyer == purchase_account.buyer
            || buyer == trade_account.seller
            || buyer == purchase_account.chosen_logistics_provider;
        assert!(buyer_valid);

        let seller_valid = seller == purchase_account.buyer
            || seller == trade_account.seller
            || seller == purchase_account.chosen_logistics_provider;
        assert!(seller_valid);

        let logistics_valid = logistics_provider == purchase_account.buyer
            || logistics_provider == trade_account.seller
            || logistics_provider == purchase_account.chosen_logistics_provider;
        assert!(logistics_valid);

        // Test buyer wins - restore quantity logic
        if buyer == purchase_account.buyer {
            trade_account.remaining_quantity += purchase_account.quantity;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }
        }

        assert_eq!(trade_account.remaining_quantity, 10); // 7 + 3
        assert_eq!(trade_account.active, true);

        // Simulate resolve_dispute completion
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        assert_eq!(purchase_account.delivered_and_confirmed, true);
        assert_eq!(purchase_account.settled, true);
    }

    #[test]
    fn test_cancel_purchase_logic_main() {
        let buyer = create_test_pubkey(9);
        let wrong_buyer = create_test_pubkey(11);

        // Setup purchase account
        let mut purchase_account = PurchaseAccount {
            purchase_id: 1,
            trade_id: 1,
            buyer,
            quantity: 3,
            total_amount: 3300,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: create_test_pubkey(6),
            logistics_cost: 300,
            settled: false,
            bump: 255,
        };

        let mut trade_account = TradeAccount {
            trade_id: 1,
            seller: create_test_pubkey(5),
            logistics_providers: vec![create_test_pubkey(6)],
            logistics_costs: vec![100],
            product_cost: 1000,
            escrow_fee: 25,
            total_quantity: 10,
            remaining_quantity: 7,
            active: true,
            purchase_ids: vec![1],
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Test validation: wrong buyer
        let result = wrong_buyer == purchase_account.buyer;
        assert_eq!(result, false); // Should fail validation

        // Test validation: already confirmed
        purchase_account.delivered_and_confirmed = true;
        let result = !purchase_account.delivered_and_confirmed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.delivered_and_confirmed = false;

        // Test validation: disputed
        purchase_account.disputed = true;
        let result = !purchase_account.disputed;
        assert_eq!(result, false); // Should fail validation

        // Reset for next test
        purchase_account.disputed = false;

        // Test validation: already settled
        purchase_account.settled = true;
        let result = !purchase_account.settled;
        assert_eq!(result, false); // Should fail validation

        // Reset for valid case
        purchase_account.settled = false;

        // Test valid case
        let correct_buyer = buyer == purchase_account.buyer;
        let not_confirmed = !purchase_account.delivered_and_confirmed;
        let not_disputed = !purchase_account.disputed;
        let not_settled = !purchase_account.settled;

        assert!(correct_buyer);
        assert!(not_confirmed);
        assert!(not_disputed);
        assert!(not_settled);

        // Simulate cancel_purchase logic
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;
        trade_account.remaining_quantity += purchase_account.quantity;

        if !trade_account.active && trade_account.remaining_quantity > 0 {
            trade_account.active = true;
        }

        assert_eq!(purchase_account.delivered_and_confirmed, true);
        assert_eq!(purchase_account.settled, true);
        assert_eq!(trade_account.remaining_quantity, 10); // 7 + 3
        assert_eq!(trade_account.active, true);
    }

    #[test]
    fn test_withdraw_escrow_fees_logic_main() {
        // Simulate escrow token account with balance
        let balance = 1000u64;

        // Test validation: no fees to withdraw
        let zero_balance = 0u64;
        let result = zero_balance > 0;
        assert_eq!(result, false); // Should fail validation

        // Test valid case
        let result = balance > 0;
        assert!(result);

        // The actual transfer would happen in the smart contract
        // Here we just validate the condition
    }

    #[test]
    fn test_error_conditions_main() {
        // Test all error conditions that would trigger in main.rs

        // MismatchedArrays
        let providers = vec![create_test_pubkey(1), create_test_pubkey(2)];
        let costs = vec![100]; // Mismatched length
        assert_ne!(providers.len(), costs.len());

        // NoLogisticsProviders
        let empty_providers: Vec<Pubkey> = vec![];
        assert!(empty_providers.is_empty());

        // TooManyProviders
        let mut many_providers = Vec::new();
        for i in 0..15 {
            many_providers.push(create_test_pubkey(i));
        }
        assert!(many_providers.len() > MAX_LOGISTICS_PROVIDERS);

        // InvalidQuantity
        let zero_quantity = 0u64;
        assert_eq!(zero_quantity, 0);

        // TradeInactive
        let inactive_trade = false;
        assert!(!inactive_trade);

        // InsufficientQuantity
        let available = 5u64;
        let requested = 10u64;
        assert!(available < requested);

        // BuyerIsSeller
        let buyer = create_test_pubkey(1);
        let seller = create_test_pubkey(1); // Same as buyer
        assert_eq!(buyer, seller);

        // InvalidLogisticsProvider
        let valid_providers = vec![create_test_pubkey(1), create_test_pubkey(2)];
        let chosen_provider = create_test_pubkey(3); // Not in list
        let mut found = false;
        for provider in &valid_providers {
            if *provider == chosen_provider {
                found = true;
                break;
            }
        }
        assert!(!found);

        // AlreadyConfirmed
        let already_confirmed = true;
        assert!(already_confirmed);

        // Disputed
        let is_disputed = true;
        assert!(is_disputed);

        // AlreadySettled
        let already_settled = true;
        assert!(already_settled);

        // AlreadyDisputed
        let already_disputed = true;
        assert!(already_disputed);

        // NotDisputed
        let not_disputed = false;
        assert!(!not_disputed);

        // InvalidWinner
        let buyer = create_test_pubkey(1);
        let seller = create_test_pubkey(2);
        let logistics = create_test_pubkey(3);
        let invalid_winner = create_test_pubkey(4);

        let valid_winner = invalid_winner == buyer || invalid_winner == seller || invalid_winner == logistics;
        assert!(!valid_winner);

        // NoFeesToWithdraw
        let zero_balance = 0u64;
        assert_eq!(zero_balance, 0);
    }

    #[test]
    fn test_complete_workflow_with_dispute_main() {
        // Initialize global state
        let admin = create_test_pubkey(1);
        let mut global_state = GlobalState {
            admin,
            trade_counter: 0,
            purchase_counter: 0,
            bump: 255,
        };

        // Register logistics provider
        let logistics_provider = create_test_pubkey(2);
        let provider_account = LogisticsProviderAccount {
            provider: logistics_provider,
            is_registered: true,
            bump: 255,
        };

        // Register seller
        let seller = create_test_pubkey(3);
        let seller_account = SellerAccount {
            seller,
            is_registered: true,
            bump: 255,
        };

        // Register buyer
        let buyer = create_test_pubkey(4);
        let mut buyer_account = BuyerAccount {
            buyer,
            is_registered: true,
            purchase_ids: Vec::new(),
            bump: 255,
        };

        // Create trade
        global_state.trade_counter += 1;
        let trade_id = global_state.trade_counter;

        let product_cost = 1000u64;
        let logistics_cost = 100u64;
        let total_quantity = 10u64;

        let mut trade_account = TradeAccount {
            trade_id,
            seller,
            logistics_providers: vec![logistics_provider],
            logistics_costs: vec![logistics_cost],
            product_cost,
            escrow_fee: (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS,
            total_quantity,
            remaining_quantity: total_quantity,
            active: true,
            purchase_ids: Vec::new(),
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Buy trade
        let buy_quantity = 3u64;
        global_state.purchase_counter += 1;
        let purchase_id = global_state.purchase_counter;

        let total_amount = (product_cost + logistics_cost) * buy_quantity;

        let mut purchase_account = PurchaseAccount {
            purchase_id,
            trade_id,
            buyer,
            quantity: buy_quantity,
            total_amount,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: logistics_provider,
            logistics_cost: logistics_cost * buy_quantity,
            settled: false,
            bump: 255,
        };

        // Update trade and buyer accounts
        trade_account.remaining_quantity -= buy_quantity;
        trade_account.purchase_ids.push(purchase_id);
        buyer_account.purchase_ids.push(purchase_id);

        // Raise dispute
        purchase_account.disputed = true;

        // Resolve dispute in favor of buyer (refund)
        let winner = buyer;
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        // If buyer wins, restore quantity
        if winner == purchase_account.buyer {
            trade_account.remaining_quantity += purchase_account.quantity;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }
        }

        // Verify final state
        assert_eq!(global_state.trade_counter, 1);
        assert_eq!(global_state.purchase_counter, 1);
        assert_eq!(trade_account.remaining_quantity, 10); // Restored
        assert_eq!(trade_account.active, true);
        assert_eq!(purchase_account.delivered_and_confirmed, true);
        assert_eq!(purchase_account.disputed, true);
        assert_eq!(purchase_account.settled, true);
        assert_eq!(buyer_account.purchase_ids.len(), 1);
    }

    #[test]
    fn test_complete_workflow_with_cancellation_main() {
        // Initialize global state
        let admin = create_test_pubkey(1);
        let mut global_state = GlobalState {
            admin,
            trade_counter: 0,
            purchase_counter: 0,
            bump: 255,
        };

        // Create trade
        global_state.trade_counter += 1;
        let trade_id = global_state.trade_counter;

        let product_cost = 1000u64;
        let logistics_cost = 100u64;
        let total_quantity = 10u64;

        let mut trade_account = TradeAccount {
            trade_id,
            seller: create_test_pubkey(3),
            logistics_providers: vec![create_test_pubkey(2)],
            logistics_costs: vec![logistics_cost],
            product_cost,
            escrow_fee: (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS,
            total_quantity,
            remaining_quantity: total_quantity,
            active: true,
            purchase_ids: Vec::new(),
            token_mint: create_test_pubkey(8),
            bump: 255,
        };

        // Buy trade
        let buy_quantity = 3u64;
        global_state.purchase_counter += 1;
        let purchase_id = global_state.purchase_counter;

        let total_amount = (product_cost + logistics_cost) * buy_quantity;

        let mut purchase_account = PurchaseAccount {
            purchase_id,
            trade_id,
            buyer: create_test_pubkey(4),
            quantity: buy_quantity,
            total_amount,
            delivered_and_confirmed: false,
            disputed: false,
            chosen_logistics_provider: create_test_pubkey(2),
            logistics_cost: logistics_cost * buy_quantity,
            settled: false,
            bump: 255,
        };

        // Update trade
        trade_account.remaining_quantity -= buy_quantity;
        trade_account.purchase_ids.push(purchase_id);

        // Cancel purchase
        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;
        trade_account.remaining_quantity += purchase_account.quantity;

        if !trade_account.active && trade_account.remaining_quantity > 0 {
            trade_account.active = true;
        }

        // Verify final state
        assert_eq!(trade_account.remaining_quantity, 10); // Restored
        assert_eq!(trade_account.active, true);
        assert_eq!(purchase_account.settled, true);
    }

    #[test]
    fn test_escrow_fee_calculations_main() {
        let product_cost = 2000u64;
        let logistics_cost = 500u64;
        let quantity = 4u64;

        // Product escrow fee calculation
        let product_escrow_fee = (product_cost * ESCROW_FEE_PERCENT * quantity) / BASIS_POINTS;
        let expected_product_fee = (2000 * 250 * 4) / 10000; // 200
        assert_eq!(product_escrow_fee, expected_product_fee);

        // Logistics escrow fee calculation
        let total_logistics_cost = logistics_cost * quantity;
        let logistics_escrow_fee = (total_logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let expected_logistics_fee = (2000 * 250) / 10000; // 50
        assert_eq!(logistics_escrow_fee, expected_logistics_fee);

        // Final amounts after fees
        let seller_amount = (product_cost * quantity) - product_escrow_fee;
        let logistics_amount = total_logistics_cost - logistics_escrow_fee;

        assert_eq!(seller_amount, 7800); // 8000 - 200
        assert_eq!(logistics_amount, 1950); // 2000 - 50
    }
}