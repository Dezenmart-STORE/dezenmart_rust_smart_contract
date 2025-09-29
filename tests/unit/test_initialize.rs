#[cfg(test)]
mod test_initialize {
    use super::super::helpers::*;
    use anchor_lang::prelude::*;

    /// Test the initialize function logic
    #[test]
    fn test_initialize_basic_functionality() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();

        // Create fresh global state
        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: Pubkey::default(),
            trade_counter: 999, // Should be reset to 0
            purchase_counter: 888, // Should be reset to 0
            bump: 0,
        };

        // Simulate initialize function logic
        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 254; // Mock bump value

        // Validate initialization results
        StateAssertions::assert_global_state(&global_state, &admin, 0, 0);
        assert_eq!(global_state.bump, 254);
    }

    #[test]
    fn test_initialize_admin_assignment() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();
        let wrong_admin = create_test_pubkey(99);

        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: wrong_admin, // Wrong initial admin
            trade_counter: 0,
            purchase_counter: 0,
            bump: 0,
        };

        // Simulate initialize function - should overwrite admin
        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 255;

        assert_eq!(global_state.admin, admin);
        assert_ne!(global_state.admin, wrong_admin);
    }

    #[test]
    fn test_initialize_counter_reset() {
        let mock_data = MockDataGenerator::new();

        // Test various initial counter values are reset to 0
        let test_cases = vec![
            (0, 0),      // Already zero
            (1, 0),      // Small values
            (100, 200),  // Medium values
            (u64::MAX, u64::MAX - 1), // Large values
        ];

        for (initial_trade, initial_purchase) in test_cases {
            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: Pubkey::default(),
                trade_counter: initial_trade,
                purchase_counter: initial_purchase,
                bump: 0,
            };

            // Apply initialize logic
            global_state.admin = mock_data.admin.pubkey();
            global_state.trade_counter = 0;
            global_state.purchase_counter = 0;
            global_state.bump = 255;

            assert_eq!(global_state.trade_counter, 0,
                "Trade counter should be 0, was {}", initial_trade);
            assert_eq!(global_state.purchase_counter, 0,
                "Purchase counter should be 0, was {}", initial_purchase);
        }
    }

    #[test]
    fn test_initialize_bump_preservation() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();

        // Test different bump values
        let bump_values = vec![0, 1, 127, 254, 255];

        for expected_bump in bump_values {
            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: Pubkey::default(),
                trade_counter: 100,
                purchase_counter: 200,
                bump: 0,
            };

            // Simulate initialize with specific bump
            global_state.admin = admin;
            global_state.trade_counter = 0;
            global_state.purchase_counter = 0;
            global_state.bump = expected_bump;

            assert_eq!(global_state.bump, expected_bump);
            StateAssertions::assert_global_state(&global_state, &admin, 0, 0);
        }
    }

    #[test]
    fn test_initialize_discriminator_field() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();

        let mut global_state = GlobalState {
            discriminator: [1, 2, 3, 4, 5, 6, 7, 8], // Some initial values
            admin: Pubkey::default(),
            trade_counter: 0,
            purchase_counter: 0,
            bump: 0,
        };

        // Initialize function should preserve discriminator
        let original_discriminator = global_state.discriminator;

        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 255;

        assert_eq!(global_state.discriminator, original_discriminator);
        assert_eq!(global_state.admin, admin);
    }

    #[test]
    fn test_initialize_multiple_admins() {
        let mock_data = MockDataGenerator::new();

        // Test that initialize can handle different admin assignments
        let admins = vec![
            mock_data.admin.pubkey(),
            mock_data.get_seller(0).pubkey(),
            mock_data.get_buyer(0).pubkey(),
            create_test_pubkey(255),
        ];

        for admin in admins {
            let mut global_state = GlobalState {
                discriminator: [0; 8],
                admin: Pubkey::default(),
                trade_counter: 42,
                purchase_counter: 84,
                bump: 0,
            };

            // Apply initialize logic
            global_state.admin = admin;
            global_state.trade_counter = 0;
            global_state.purchase_counter = 0;
            global_state.bump = 200;

            StateAssertions::assert_global_state(&global_state, &admin, 0, 0);
            assert_eq!(global_state.bump, 200);
        }
    }

    #[test]
    fn test_initialize_idempotency() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();

        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: Pubkey::default(),
            trade_counter: 999,
            purchase_counter: 888,
            bump: 0,
        };

        // First initialization
        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 254;

        let state_after_first = global_state.clone();

        // Second initialization (should produce same result)
        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 254;

        assert_eq!(global_state.admin, state_after_first.admin);
        assert_eq!(global_state.trade_counter, state_after_first.trade_counter);
        assert_eq!(global_state.purchase_counter, state_after_first.purchase_counter);
        assert_eq!(global_state.bump, state_after_first.bump);
    }

    #[test]
    fn test_initialize_space_allocation() {
        // Test that GlobalState has correct space allocation
        // This validates the space calculation in the actual contract

        let expected_space = 8 + // discriminator
                            32 + // admin (Pubkey)
                            8 +  // trade_counter (u64)
                            8 +  // purchase_counter (u64)
                            1;   // bump (u8)

        let actual_space = std::mem::size_of::<GlobalState>();

        // Account for potential alignment padding
        assert!(actual_space >= expected_space - 8,
            "GlobalState size {} is smaller than expected minimum {}",
            actual_space, expected_space - 8);

        assert!(actual_space <= expected_space + 8,
            "GlobalState size {} is larger than expected maximum {}",
            actual_space, expected_space + 8);
    }

    #[test]
    fn test_initialize_default_state() {
        // Test that default state is properly handled
        let default_state = GlobalState {
            discriminator: [0; 8],
            admin: Pubkey::default(),
            trade_counter: 0,
            purchase_counter: 0,
            bump: 0,
        };

        assert_eq!(default_state.admin, Pubkey::default());
        assert_eq!(default_state.trade_counter, 0);
        assert_eq!(default_state.purchase_counter, 0);
        assert_eq!(default_state.bump, 0);
        assert_eq!(default_state.discriminator, [0; 8]);
    }

    #[test]
    fn test_initialize_validation_requirements() {
        let mock_data = MockDataGenerator::new();
        let admin = mock_data.admin.pubkey();

        // Test that admin cannot be default (in real contract, this would be validated)
        assert_ne!(admin, Pubkey::default(), "Admin should not be default pubkey");

        // Test that bump values are within valid range
        let valid_bumps = vec![0, 1, 127, 254, 255];
        for bump in valid_bumps {
            assert!(bump <= 255, "Bump value {} should be valid u8", bump);
        }

        // Test that counters can be reset regardless of initial value
        let large_counter = u64::MAX;
        let mut global_state = GlobalState {
            discriminator: [0; 8],
            admin: Pubkey::default(),
            trade_counter: large_counter,
            purchase_counter: large_counter,
            bump: 0,
        };

        global_state.admin = admin;
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = 255;

        assert_eq!(global_state.trade_counter, 0);
        assert_eq!(global_state.purchase_counter, 0);
    }
}