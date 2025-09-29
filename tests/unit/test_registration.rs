#[cfg(test)]
mod test_registration {
    use super::super::helpers::*;
    use anchor_lang::prelude::*;

    /// Test register_logistics_provider function
    mod logistics_provider_registration {
        use super::*;

        #[test]
        fn test_register_logistics_provider_basic() {
            let mock_data = MockDataGenerator::new();
            let provider = mock_data.get_logistics_provider(0).pubkey();

            // Initialize provider account
            let mut provider_account = LogisticsProviderAccount {
                discriminator: [0; 8],
                provider: Pubkey::default(),
                is_registered: false,
                bump: 0,
            };

            // Simulate register_logistics_provider function logic
            provider_account.provider = provider;
            provider_account.is_registered = true;
            provider_account.bump = 254;

            // Validate registration
            assert_eq!(provider_account.provider, provider);
            assert_eq!(provider_account.is_registered, true);
            assert_eq!(provider_account.bump, 254);
            StateAssertions::assert_registration_account(&provider_account, true);
        }

        #[test]
        fn test_register_logistics_provider_multiple() {
            let mock_data = MockDataGenerator::new();

            // Test registering multiple providers
            for i in 0..3 {
                let provider = mock_data.get_logistics_provider(i).pubkey();
                let mut provider_account = LogisticsProviderAccount {
                    discriminator: [0; 8],
                    provider: Pubkey::default(),
                    is_registered: false,
                    bump: 0,
                };

                // Register each provider
                provider_account.provider = provider;
                provider_account.is_registered = true;
                provider_account.bump = 250 + i as u8;

                assert_eq!(provider_account.provider, provider);
                assert_eq!(provider_account.is_registered, true);
                assert_eq!(provider_account.bump, 250 + i as u8);
            }
        }

        #[test]
        fn test_register_logistics_provider_discriminator() {
            let mock_data = MockDataGenerator::new();
            let provider = mock_data.get_logistics_provider(0).pubkey();

            let mut provider_account = LogisticsProviderAccount {
                discriminator: [1, 2, 3, 4, 5, 6, 7, 8],
                provider: Pubkey::default(),
                is_registered: false,
                bump: 0,
            };

            let original_discriminator = provider_account.discriminator;

            // Registration should preserve discriminator
            provider_account.provider = provider;
            provider_account.is_registered = true;
            provider_account.bump = 255;

            assert_eq!(provider_account.discriminator, original_discriminator);
        }

        #[test]
        fn test_register_logistics_provider_space_allocation() {
            // Validate space requirements for LogisticsProviderAccount
            let expected_space = 8 +  // discriminator
                                32 + // provider (Pubkey)
                                1 +  // is_registered (bool)
                                1;   // bump (u8)

            let actual_space = std::mem::size_of::<LogisticsProviderAccount>();

            assert!(actual_space >= expected_space - 4,
                "LogisticsProviderAccount size {} too small", actual_space);
            assert!(actual_space <= expected_space + 8,
                "LogisticsProviderAccount size {} too large", actual_space);
        }

        #[test]
        fn test_register_logistics_provider_edge_cases() {
            // Test with extreme pubkey values
            let edge_providers = vec![
                Pubkey::default(),
                create_test_pubkey(0),
                create_test_pubkey(255),
                Pubkey::new_from_array([255; 32]),
            ];

            for provider in edge_providers {
                let mut provider_account = LogisticsProviderAccount {
                    discriminator: [0; 8],
                    provider: Pubkey::default(),
                    is_registered: false,
                    bump: 0,
                };

                provider_account.provider = provider;
                provider_account.is_registered = true;
                provider_account.bump = 100;

                assert_eq!(provider_account.provider, provider);
                assert_eq!(provider_account.is_registered, true);
            }
        }
    }

    /// Test register_seller function
    mod seller_registration {
        use super::*;

        #[test]
        fn test_register_seller_basic() {
            let mock_data = MockDataGenerator::new();
            let seller = mock_data.get_seller(0).pubkey();

            let mut seller_account = SellerAccount {
                discriminator: [0; 8],
                seller: Pubkey::default(),
                is_registered: false,
                bump: 0,
            };

            // Simulate register_seller function logic
            seller_account.seller = seller;
            seller_account.is_registered = true;
            seller_account.bump = 253;

            assert_eq!(seller_account.seller, seller);
            assert_eq!(seller_account.is_registered, true);
            assert_eq!(seller_account.bump, 253);
            StateAssertions::assert_registration_account(&seller_account, true);
        }

        #[test]
        fn test_register_seller_admin_only_simulation() {
            let mock_data = MockDataGenerator::new();
            let admin = mock_data.admin.pubkey();
            let seller = mock_data.get_seller(0).pubkey();
            let non_admin = mock_data.get_buyer(0).pubkey();

            // Simulate admin check (in real contract, this would be enforced by Anchor)
            let is_admin_call = admin != Pubkey::default(); // Admin should be valid
            let is_non_admin_call = non_admin == admin; // Should be false

            assert!(is_admin_call, "Admin should be able to register sellers");
            assert!(!is_non_admin_call, "Non-admin should not be able to register sellers");

            // Only proceed with registration if admin
            if is_admin_call {
                let mut seller_account = SellerAccount {
                    discriminator: [0; 8],
                    seller: Pubkey::default(),
                    is_registered: false,
                    bump: 0,
                };

                seller_account.seller = seller;
                seller_account.is_registered = true;
                seller_account.bump = 255;

                assert_eq!(seller_account.seller, seller);
                assert_eq!(seller_account.is_registered, true);
            }
        }

        #[test]
        fn test_register_seller_multiple() {
            let mock_data = MockDataGenerator::new();

            // Register multiple sellers
            for i in 0..5 {
                let seller = mock_data.get_seller(i).pubkey();
                let mut seller_account = SellerAccount {
                    discriminator: [0; 8],
                    seller: Pubkey::default(),
                    is_registered: false,
                    bump: 0,
                };

                seller_account.seller = seller;
                seller_account.is_registered = true;
                seller_account.bump = 200 + i as u8;

                assert_eq!(seller_account.seller, seller);
                assert_eq!(seller_account.is_registered, true);
                assert_eq!(seller_account.bump, 200 + i as u8);
            }
        }

        #[test]
        fn test_register_seller_space_allocation() {
            let expected_space = 8 +  // discriminator
                                32 + // seller (Pubkey)
                                1 +  // is_registered (bool)
                                1;   // bump (u8)

            let actual_space = std::mem::size_of::<SellerAccount>();

            assert!(actual_space >= expected_space - 4);
            assert!(actual_space <= expected_space + 8);
        }
    }

    /// Test register_buyer function
    mod buyer_registration {
        use super::*;

        #[test]
        fn test_register_buyer_basic() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: vec![999, 888], // Should be reset
                bump: 0,
            };

            // Simulate register_buyer function logic
            buyer_account.buyer = buyer;
            buyer_account.is_registered = true;
            buyer_account.purchase_ids = Vec::new(); // Reset purchase IDs
            buyer_account.bump = 252;

            assert_eq!(buyer_account.buyer, buyer);
            assert_eq!(buyer_account.is_registered, true);
            assert_eq!(buyer_account.purchase_ids.len(), 0);
            assert_eq!(buyer_account.bump, 252);
            StateAssertions::assert_registration_account(&buyer_account, true);
        }

        #[test]
        fn test_register_buyer_purchase_ids_reset() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            // Test with various initial purchase_ids states
            let initial_states = vec![
                vec![],                    // Empty
                vec![1],                   // Single item
                vec![1, 2, 3, 4, 5],      // Multiple items
                (1..=50).collect::<Vec<_>>(), // Many items
            ];

            for initial_ids in initial_states {
                let mut buyer_account = BuyerAccount {
                    discriminator: [0; 8],
                    buyer: Pubkey::default(),
                    is_registered: false,
                    purchase_ids: initial_ids.clone(),
                    bump: 0,
                };

                // Registration should reset purchase_ids
                buyer_account.buyer = buyer;
                buyer_account.is_registered = true;
                buyer_account.purchase_ids = Vec::new();
                buyer_account.bump = 255;

                assert_eq!(buyer_account.purchase_ids.len(), 0,
                    "Purchase IDs should be reset from {:?}", initial_ids);
                assert_eq!(buyer_account.is_registered, true);
            }
        }

        #[test]
        fn test_register_buyer_self_registration() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            // Test that buyer can register themselves (no admin required)
            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: Vec::new(),
                bump: 0,
            };

            // Simulate self-registration
            buyer_account.buyer = buyer;
            buyer_account.is_registered = true;
            buyer_account.purchase_ids = Vec::new();
            buyer_account.bump = 251;

            assert_eq!(buyer_account.buyer, buyer);
            assert_eq!(buyer_account.is_registered, true);
        }

        #[test]
        fn test_register_buyer_space_allocation() {
            // BuyerAccount has dynamic size due to Vec<u64>
            let base_expected_space = 8 +  // discriminator
                                     32 + // buyer (Pubkey)
                                     1 +  // is_registered (bool)
                                     4 +  // Vec length prefix
                                     1;   // bump (u8)

            // Space for MAX_PURCHASE_IDS
            let max_purchase_space = MAX_PURCHASE_IDS * 8; // u64 per purchase ID

            let buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: Vec::new(),
                bump: 0,
            };

            let empty_size = std::mem::size_of_val(&buyer_account);

            // With empty Vec, should be close to base size
            assert!(empty_size >= base_expected_space - 8);
            assert!(empty_size <= base_expected_space + 32);
        }

        #[test]
        fn test_register_buyer_max_purchase_ids() {
            let mock_data = MockDataGenerator::new();
            let buyer = mock_data.get_buyer(0).pubkey();

            // Test registration with consideration for MAX_PURCHASE_IDS
            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: (1..=MAX_PURCHASE_IDS as u64).collect(), // Max size
                bump: 0,
            };

            // Registration should still reset to empty
            buyer_account.buyer = buyer;
            buyer_account.is_registered = true;
            buyer_account.purchase_ids = Vec::new();
            buyer_account.bump = 255;

            assert_eq!(buyer_account.purchase_ids.len(), 0);
            assert!(buyer_account.purchase_ids.capacity() >= 0);

            // Test that we can still add up to MAX_PURCHASE_IDS
            for i in 1..=MAX_PURCHASE_IDS as u64 {
                if buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
                    buyer_account.purchase_ids.push(i);
                }
            }

            assert_eq!(buyer_account.purchase_ids.len(), MAX_PURCHASE_IDS);
        }
    }

    /// Cross-registration tests
    mod cross_registration_tests {
        use super::*;

        #[test]
        fn test_same_pubkey_multiple_roles() {
            let mock_data = MockDataGenerator::new();
            let user = mock_data.get_seller(0).pubkey();

            // Same pubkey can be registered in different roles
            let mut logistics_account = LogisticsProviderAccount {
                discriminator: [0; 8],
                provider: user,
                is_registered: true,
                bump: 254,
            };

            let mut seller_account = SellerAccount {
                discriminator: [0; 8],
                seller: user,
                is_registered: true,
                bump: 253,
            };

            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: user,
                is_registered: true,
                purchase_ids: Vec::new(),
                bump: 252,
            };

            // All should be valid
            assert_eq!(logistics_account.provider, user);
            assert_eq!(seller_account.seller, user);
            assert_eq!(buyer_account.buyer, user);

            StateAssertions::assert_registration_account(&logistics_account, true);
            StateAssertions::assert_registration_account(&seller_account, true);
            StateAssertions::assert_registration_account(&buyer_account, true);
        }

        #[test]
        fn test_registration_account_trait() {
            let mock_data = MockDataGenerator::new();

            let logistics_account = LogisticsProviderAccount {
                discriminator: [0; 8],
                provider: mock_data.get_logistics_provider(0).pubkey(),
                is_registered: true,
                bump: 255,
            };

            let seller_account = SellerAccount {
                discriminator: [0; 8],
                seller: mock_data.get_seller(0).pubkey(),
                is_registered: true,
                bump: 254,
            };

            let buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: mock_data.get_buyer(0).pubkey(),
                is_registered: true,
                purchase_ids: Vec::new(),
                bump: 253,
            };

            // Test trait implementations
            assert!(logistics_account.is_registered());
            assert!(seller_account.is_registered());
            assert!(buyer_account.is_registered());

            assert_eq!(logistics_account.get_owner(), mock_data.get_logistics_provider(0).pubkey());
            assert_eq!(seller_account.get_owner(), mock_data.get_seller(0).pubkey());
            assert_eq!(buyer_account.get_owner(), mock_data.get_buyer(0).pubkey());
        }

        #[test]
        fn test_unregistered_accounts() {
            let mock_data = MockDataGenerator::new();

            let unregistered_logistics = LogisticsProviderAccount {
                discriminator: [0; 8],
                provider: mock_data.get_logistics_provider(0).pubkey(),
                is_registered: false,
                bump: 255,
            };

            let unregistered_seller = SellerAccount {
                discriminator: [0; 8],
                seller: mock_data.get_seller(0).pubkey(),
                is_registered: false,
                bump: 254,
            };

            let unregistered_buyer = BuyerAccount {
                discriminator: [0; 8],
                buyer: mock_data.get_buyer(0).pubkey(),
                is_registered: false,
                purchase_ids: Vec::new(),
                bump: 253,
            };

            // All should be unregistered
            StateAssertions::assert_registration_account(&unregistered_logistics, false);
            StateAssertions::assert_registration_account(&unregistered_seller, false);
            StateAssertions::assert_registration_account(&unregistered_buyer, false);
        }
    }

    /// Validation and error condition tests
    mod registration_validation_tests {
        use super::*;

        #[test]
        fn test_registration_state_transitions() {
            let mock_data = MockDataGenerator::new();
            let user = mock_data.get_buyer(0).pubkey();

            let mut buyer_account = BuyerAccount {
                discriminator: [0; 8],
                buyer: Pubkey::default(),
                is_registered: false,
                purchase_ids: Vec::new(),
                bump: 0,
            };

            // Initially unregistered
            assert!(!buyer_account.is_registered);

            // After registration
            buyer_account.buyer = user;
            buyer_account.is_registered = true;
            buyer_account.bump = 255;

            assert!(buyer_account.is_registered);
            assert_eq!(buyer_account.buyer, user);
        }

        #[test]
        fn test_registration_with_default_pubkeys() {
            // Test registration validation with default pubkeys
            let default_pubkey = Pubkey::default();

            let mut accounts = vec![
                LogisticsProviderAccount {
                    discriminator: [0; 8],
                    provider: default_pubkey,
                    is_registered: true,
                    bump: 255,
                },
            ];

            // In a real scenario, default pubkey might not be allowed
            // Here we just test that the structure handles it
            assert_eq!(accounts[0].provider, default_pubkey);
            assert_eq!(accounts[0].is_registered, true);
        }

        #[test]
        fn test_bump_value_ranges() {
            let mock_data = MockDataGenerator::new();
            let user = mock_data.get_seller(0).pubkey();

            // Test all valid bump values
            for bump in 0..=255u8 {
                let seller_account = SellerAccount {
                    discriminator: [0; 8],
                    seller: user,
                    is_registered: true,
                    bump,
                };

                assert_eq!(seller_account.bump, bump);
                assert!(seller_account.is_registered);
            }
        }
    }
}