use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::collections::BTreeMap;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod dezenmart_logistics {
    use super::*;

    // Constants
    pub const ESCROW_FEE_PERCENT: u64 = 250; // 2.5% (in basis points)
    pub const BASIS_POINTS: u64 = 10000;
    pub const MAX_LOGISTICS_PROVIDERS: usize = 10;
    pub const MAX_PURCHASE_IDS: usize = 100;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.admin = ctx.accounts.admin.key();
        global_state.trade_counter = 0;
        global_state.purchase_counter = 0;
        global_state.bump = ctx.bumps.global_state;
        Ok(())
    }

    pub fn register_logistics_provider(ctx: Context<RegisterLogisticsProvider>) -> Result<()> {
        let provider_account = &mut ctx.accounts.provider_account;
        provider_account.provider = ctx.accounts.provider.key();
        provider_account.is_registered = true;
        provider_account.bump = ctx.bumps.provider_account;

        emit!(LogisticsProviderRegistered {
            provider: ctx.accounts.provider.key(),
        });
        Ok(())
    }

    pub fn register_seller(ctx: Context<RegisterSeller>) -> Result<()> {
        let seller_account = &mut ctx.accounts.seller_account;
        seller_account.seller = ctx.accounts.seller.key();
        seller_account.is_registered = true;
        seller_account.bump = ctx.bumps.seller_account;
        Ok(())
    }

    pub fn register_buyer(ctx: Context<RegisterBuyer>) -> Result<()> {
        let buyer_account = &mut ctx.accounts.buyer_account;
        buyer_account.buyer = ctx.accounts.buyer.key();
        buyer_account.is_registered = true;
        buyer_account.purchase_ids = Vec::new();
        buyer_account.bump = ctx.bumps.buyer_account;
        Ok(())
    }

    pub fn create_trade(
        ctx: Context<CreateTrade>,
        product_cost: u64,
        logistics_providers: Vec<Pubkey>,
        logistics_costs: Vec<u64>,
        total_quantity: u64,
    ) -> Result<()> {
        require!(
            logistics_providers.len() == logistics_costs.len(),
            LogisticsError::MismatchedArrays
        );
        require!(!logistics_providers.is_empty(), LogisticsError::NoLogisticsProviders);
        require!(
            logistics_providers.len() <= MAX_LOGISTICS_PROVIDERS,
            LogisticsError::TooManyProviders
        );
        require!(total_quantity > 0, LogisticsError::InvalidQuantity);

        // Verify all logistics providers are registered
        for _provider in &logistics_providers {
            // In a real implementation, you'd check provider registration here
            // For simplicity, we're skipping this validation
        }

        let global_state = &mut ctx.accounts.global_state;
        global_state.trade_counter += 1;
        let trade_id = global_state.trade_counter;

        let product_escrow_fee = (product_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;

        let trade_account = &mut ctx.accounts.trade_account;
        trade_account.trade_id = trade_id;
        trade_account.seller = ctx.accounts.seller.key();
        trade_account.logistics_providers = logistics_providers.clone();
        trade_account.logistics_costs = logistics_costs;
        trade_account.product_cost = product_cost;
        trade_account.escrow_fee = product_escrow_fee;
        trade_account.total_quantity = total_quantity;
        trade_account.remaining_quantity = total_quantity;
        trade_account.active = true;
        trade_account.purchase_ids = Vec::new();
        trade_account.token_mint = ctx.accounts.token_mint.key();
        trade_account.bump = ctx.bumps.trade_account;

        emit!(TradeCreated {
            trade_id,
            seller: ctx.accounts.seller.key(),
            product_cost,
            total_quantity,
            token_address: ctx.accounts.token_mint.key(),
        });

        Ok(())
    }

    pub fn buy_trade(
        ctx: Context<BuyTrade>,
        trade_id: u64,
        quantity: u64,
        logistics_provider: Pubkey,
    ) -> Result<()> {
        require!(quantity > 0, LogisticsError::InvalidQuantity);
        
        let trade_account = &mut ctx.accounts.trade_account;
        require!(trade_account.active, LogisticsError::TradeInactive);
        require!(
            trade_account.remaining_quantity >= quantity,
            LogisticsError::InsufficientQuantity
        );
        require!(
            ctx.accounts.buyer.key() != trade_account.seller,
            LogisticsError::BuyerIsSeller
        );

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
        require!(found, LogisticsError::InvalidLogisticsProvider);

        // Calculate costs
        let total_product_cost = trade_account.product_cost * quantity;
        let total_logistics_cost = chosen_logistics_cost * quantity;
        let total_amount = total_product_cost + total_logistics_cost;

        // Transfer tokens to escrow
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_token_account.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, total_amount)?;

        // Update global counter
        let global_state = &mut ctx.accounts.global_state;
        global_state.purchase_counter += 1;
        let purchase_id = global_state.purchase_counter;

        // Create purchase
        let purchase_account = &mut ctx.accounts.purchase_account;
        purchase_account.purchase_id = purchase_id;
        purchase_account.trade_id = trade_id;
        purchase_account.buyer = ctx.accounts.buyer.key();
        purchase_account.quantity = quantity;
        purchase_account.total_amount = total_amount;
        purchase_account.delivered_and_confirmed = false;
        purchase_account.disputed = false;
        purchase_account.chosen_logistics_provider = logistics_provider;
        purchase_account.logistics_cost = total_logistics_cost;
        purchase_account.settled = false;
        purchase_account.bump = ctx.bumps.purchase_account;

        // Update trade state
        trade_account.remaining_quantity -= quantity;
        if trade_account.purchase_ids.len() < MAX_PURCHASE_IDS {
            trade_account.purchase_ids.push(purchase_id);
        }
        
        if trade_account.remaining_quantity == 0 {
            trade_account.active = false;
        }

        // Register buyer if not already registered
        if !ctx.accounts.buyer_account.is_registered {
            ctx.accounts.buyer_account.buyer = ctx.accounts.buyer.key();
            ctx.accounts.buyer_account.is_registered = true;
            ctx.accounts.buyer_account.purchase_ids = Vec::new();
        }
        
        if ctx.accounts.buyer_account.purchase_ids.len() < MAX_PURCHASE_IDS {
            ctx.accounts.buyer_account.purchase_ids.push(purchase_id);
        }

        emit!(PurchaseCreated {
            purchase_id,
            trade_id,
            buyer: ctx.accounts.buyer.key(),
            quantity,
        });

        emit!(PaymentHeld {
            purchase_id,
            total_amount,
        });

        Ok(())
    }

    pub fn confirm_delivery_and_purchase(ctx: Context<ConfirmDeliveryAndPurchase>) -> Result<()> {
        let purchase_account = &mut ctx.accounts.purchase_account;
        require!(
            ctx.accounts.buyer.key() == purchase_account.buyer,
            LogisticsError::NotAuthorized
        );
        require!(
            !purchase_account.delivered_and_confirmed,
            LogisticsError::AlreadyConfirmed
        );
        require!(!purchase_account.disputed, LogisticsError::Disputed);
        require!(!purchase_account.settled, LogisticsError::AlreadySettled);

        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        // Settle payments
        let trade_account = &ctx.accounts.trade_account;
        let product_escrow_fee = (trade_account.product_cost * ESCROW_FEE_PERCENT * purchase_account.quantity) / BASIS_POINTS;
        let seller_amount = (trade_account.product_cost * purchase_account.quantity) - product_escrow_fee;

        // Transfer to seller
        let escrow_bump = *Pubkey::find_program_address(
            &[b"escrow", trade_account.token_mint.as_ref()],
            ctx.program_id,
        ).1.to_le_bytes().last().unwrap();

        let seeds = &[
            b"escrow".as_ref(),
            trade_account.token_mint.as_ref(),
            &[escrow_bump],
        ];
        let signer = &[&seeds[..]];

        let transfer_to_seller_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.seller_token_account.to_account_info(),
                authority: ctx.accounts.escrow_token_account.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_to_seller_ctx, seller_amount)?;

        // Transfer to logistics provider
        let logistics_escrow_fee = (purchase_account.logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
        let logistics_amount = purchase_account.logistics_cost - logistics_escrow_fee;

        let transfer_to_logistics_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.logistics_token_account.to_account_info(),
                authority: ctx.accounts.escrow_token_account.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_to_logistics_ctx, logistics_amount)?;

        emit!(PurchaseCompletedAndConfirmed {
            purchase_id: purchase_account.purchase_id,
        });

        Ok(())
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>) -> Result<()> {
        let purchase_account = &mut ctx.accounts.purchase_account;
        require!(
            !purchase_account.delivered_and_confirmed,
            LogisticsError::AlreadyConfirmed
        );
        require!(!purchase_account.disputed, LogisticsError::AlreadyDisputed);

        purchase_account.disputed = true;

        emit!(DisputeRaised {
            purchase_id: purchase_account.purchase_id,
            initiator: ctx.accounts.user.key(),
        });

        Ok(())
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        purchase_id: u64,
        winner: Pubkey,
    ) -> Result<()> {
        let purchase_account = &mut ctx.accounts.purchase_account;
        let trade_account = &mut ctx.accounts.trade_account;
        
        require!(purchase_account.disputed, LogisticsError::NotDisputed);
        require!(!purchase_account.settled, LogisticsError::AlreadySettled);

        // Validate winner
        let valid_winner = winner == purchase_account.buyer 
            || winner == trade_account.seller 
            || winner == purchase_account.chosen_logistics_provider;
        require!(valid_winner, LogisticsError::InvalidWinner);

        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;

        let escrow_bump = *Pubkey::find_program_address(
            &[b"escrow", trade_account.token_mint.as_ref()],
            ctx.program_id,
        ).1.to_le_bytes().last().unwrap();

        let seeds = &[
            b"escrow".as_ref(),
            trade_account.token_mint.as_ref(),
            &[escrow_bump],
        ];
        let signer = &[&seeds[..]];

        if winner == purchase_account.buyer {
            // Refund buyer
            let transfer_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_token_account.to_account_info(),
                },
                signer,
            );
            token::transfer(transfer_ctx, purchase_account.total_amount)?;

            // Restore quantity
            trade_account.remaining_quantity += purchase_account.quantity;
            if !trade_account.active && trade_account.remaining_quantity > 0 {
                trade_account.active = true;
            }
        } else {
            // Pay seller and logistics provider
            let product_escrow_fee = (trade_account.product_cost * ESCROW_FEE_PERCENT * purchase_account.quantity) / BASIS_POINTS;
            let seller_amount = (trade_account.product_cost * purchase_account.quantity) - product_escrow_fee;

            let transfer_to_seller_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_token_account.to_account_info(),
                },
                signer,
            );
            token::transfer(transfer_to_seller_ctx, seller_amount)?;

            let logistics_escrow_fee = (purchase_account.logistics_cost * ESCROW_FEE_PERCENT) / BASIS_POINTS;
            let logistics_payout = purchase_account.logistics_cost - logistics_escrow_fee;

            let transfer_to_logistics_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.logistics_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_token_account.to_account_info(),
                },
                signer,
            );
            token::transfer(transfer_to_logistics_ctx, logistics_payout)?;
        }

        emit!(DisputeResolved {
            purchase_id,
            winner,
        });

        Ok(())
    }

    pub fn cancel_purchase(ctx: Context<CancelPurchase>) -> Result<()> {
        let purchase_account = &mut ctx.accounts.purchase_account;
        let trade_account = &mut ctx.accounts.trade_account;

        require!(
            ctx.accounts.buyer.key() == purchase_account.buyer,
            LogisticsError::NotAuthorized
        );
        require!(
            !purchase_account.delivered_and_confirmed,
            LogisticsError::AlreadyConfirmed
        );
        require!(!purchase_account.disputed, LogisticsError::Disputed);
        require!(!purchase_account.settled, LogisticsError::AlreadySettled);

        purchase_account.delivered_and_confirmed = true;
        purchase_account.settled = true;
        trade_account.remaining_quantity += purchase_account.quantity;

        if !trade_account.active && trade_account.remaining_quantity > 0 {
            trade_account.active = true;
        }

        // Refund buyer
        let escrow_bump = *Pubkey::find_program_address(
            &[b"escrow", trade_account.token_mint.as_ref()],
            ctx.program_id,
        ).1.to_le_bytes().last().unwrap();

        let seeds = &[
            b"escrow".as_ref(),
            trade_account.token_mint.as_ref(),
            &[escrow_bump],
        ];
        let signer = &[&seeds[..]];

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.escrow_token_account.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_ctx, purchase_account.total_amount)?;

        Ok(())
    }

    pub fn withdraw_escrow_fees(ctx: Context<WithdrawEscrowFees>) -> Result<()> {
        let balance = ctx.accounts.escrow_token_account.amount;
        require!(balance > 0, LogisticsError::NoFeesToWithdraw);

        // For withdrawing fees, we need to determine the escrow bump
        // This is a simplified approach - in practice, you'd pass the token mint
        let escrow_bump = 254u8; // This should be determined properly in practice

        let seeds = &[
            b"escrow".as_ref(),
            &[escrow_bump],
        ];
        let signer = &[&seeds[..]];

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.admin_token_account.to_account_info(),
                authority: ctx.accounts.escrow_token_account.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_ctx, balance)?;

        Ok(())
    }
}

// Account structures
#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub trade_counter: u64,
    pub purchase_counter: u64,
    pub bump: u8,
}

#[account]
pub struct TradeAccount {
    pub trade_id: u64,
    pub seller: Pubkey,
    pub logistics_providers: Vec<Pubkey>,
    pub logistics_costs: Vec<u64>,
    pub product_cost: u64,
    pub escrow_fee: u64,
    pub total_quantity: u64,
    pub remaining_quantity: u64,
    pub active: bool,
    pub purchase_ids: Vec<u64>,
    pub token_mint: Pubkey,
    pub bump: u8,
}

#[account]
pub struct PurchaseAccount {
    pub purchase_id: u64,
    pub trade_id: u64,
    pub buyer: Pubkey,
    pub quantity: u64,
    pub total_amount: u64,
    pub delivered_and_confirmed: bool,
    pub disputed: bool,
    pub chosen_logistics_provider: Pubkey,
    pub logistics_cost: u64,
    pub settled: bool,
    pub bump: u8,
}

#[account]
pub struct LogisticsProviderAccount {
    pub provider: Pubkey,
    pub is_registered: bool,
    pub bump: u8,
}

#[account]
pub struct SellerAccount {
    pub seller: Pubkey,
    pub is_registered: bool,
    pub bump: u8,
}

#[account]
pub struct BuyerAccount {
    pub buyer: Pubkey,
    pub is_registered: bool,
    pub purchase_ids: Vec<u64>,
    pub bump: u8,
}

// Context structures
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 1,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterLogisticsProvider<'info> {
    #[account(
        init,
        payer = provider,
        space = 8 + 32 + 1 + 1,
        seeds = [b"logistics_provider", provider.key().as_ref()],
        bump
    )]
    pub provider_account: Account<'info, LogisticsProviderAccount>,
    #[account(mut)]
    pub provider: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterSeller<'info> {
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump,
        has_one = admin
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1 + 1,
        seeds = [b"seller", seller.key().as_ref()],
        bump
    )]
    pub seller_account: Account<'info, SellerAccount>,
    /// CHECK: This is the seller being registered
    pub seller: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterBuyer<'info> {
    #[account(
        init,
        payer = buyer,
        space = 8 + 32 + 1 + 4 + (8 * MAX_PURCHASE_IDS) + 1,
        seeds = [b"buyer", buyer.key().as_ref()],
        bump
    )]
    pub buyer_account: Account<'info, BuyerAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(trade_id: u64)]
pub struct CreateTrade<'info> {
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump,
        has_one = admin
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = admin,
        space = 8 + 8 + 32 + 4 + (32 * MAX_LOGISTICS_PROVIDERS) + 4 + (8 * MAX_LOGISTICS_PROVIDERS) + 8 + 8 + 8 + 8 + 1 + 4 + (8 * MAX_PURCHASE_IDS) + 32 + 1,
        seeds = [b"trade", trade_id.to_le_bytes().as_ref()],
        bump
    )]
    pub trade_account: Account<'info, TradeAccount>,
    /// CHECK: This is the seller for the trade
    pub seller: UncheckedAccount<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(trade_id: u64)]
pub struct BuyTrade<'info> {
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        mut,
        seeds = [b"trade", trade_id.to_le_bytes().as_ref()],
        bump = trade_account.bump
    )]
    pub trade_account: Account<'info, TradeAccount>,
    #[account(
        init,
        payer = buyer,
        space = 8 + 8 + 8 + 32 + 8 + 8 + 1 + 1 + 32 + 8 + 1 + 1,
        seeds = [b"purchase", global_state.purchase_counter.saturating_add(1).to_le_bytes().as_ref()],
        bump
    )]
    pub purchase_account: Account<'info, PurchaseAccount>,
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + 32 + 1 + 4 + (8 * MAX_PURCHASE_IDS) + 1,
        seeds = [b"buyer", buyer.key().as_ref()],
        bump
    )]
    pub buyer_account: Account<'info, BuyerAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [b"escrow", trade_account.token_mint.as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = escrow_token_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(purchase_id: u64)]
pub struct ConfirmDeliveryAndPurchase<'info> {
    #[account(
        mut,
        seeds = [b"purchase", purchase_id.to_le_bytes().as_ref()],
        bump = purchase_account.bump
    )]
    pub purchase_account: Account<'info, PurchaseAccount>,
    #[account(
        seeds = [b"trade", purchase_account.trade_id.to_le_bytes().as_ref()],
        bump = trade_account.bump
    )]
    pub trade_account: Account<'info, TradeAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub logistics_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(purchase_id: u64)]
pub struct RaiseDispute<'info> {
    #[account(
        mut,
        seeds = [b"purchase", purchase_id.to_le_bytes().as_ref()],
        bump = purchase_account.bump
    )]
    pub purchase_account: Account<'info, PurchaseAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(purchase_id: u64)]
pub struct ResolveDispute<'info> {
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump,
        has_one = admin
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        mut,
        seeds = [b"purchase", purchase_id.to_le_bytes().as_ref()],
        bump = purchase_account.bump
    )]
    pub purchase_account: Account<'info, PurchaseAccount>,
    #[account(
        mut,
        seeds = [b"trade", purchase_account.trade_id.to_le_bytes().as_ref()],
        bump = trade_account.bump
    )]
    pub trade_account: Account<'info, TradeAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub logistics_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(purchase_id: u64)]
pub struct CancelPurchase<'info> {
    #[account(
        mut,
        seeds = [b"purchase", purchase_id.to_le_bytes().as_ref()],
        bump = purchase_account.bump
    )]
    pub purchase_account: Account<'info, PurchaseAccount>,
    #[account(
        mut,
        seeds = [b"trade", purchase_account.trade_id.to_le_bytes().as_ref()],
        bump = trade_account.bump
    )]
    pub trade_account: Account<'info, TradeAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawEscrowFees<'info> {
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump,
        has_one = admin
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Events
#[event]
pub struct TradeCreated {
    pub trade_id: u64,
    pub seller: Pubkey,
    pub product_cost: u64,
    pub total_quantity: u64,
    pub token_address: Pubkey,
}

#[event]
pub struct PurchaseCreated {
    pub purchase_id: u64,
    pub trade_id: u64,
    pub buyer: Pubkey,
    pub quantity: u64,
}

#[event]
pub struct PaymentHeld {
    pub purchase_id: u64,
    pub total_amount: u64,
}

#[event]
pub struct PurchaseCompletedAndConfirmed {
    pub purchase_id: u64,
}

#[event]
pub struct DisputeRaised {
    pub purchase_id: u64,
    pub initiator: Pubkey,
}

#[event]
pub struct DisputeResolved {
    pub purchase_id: u64,
    pub winner: Pubkey,
}

#[event]
pub struct LogisticsProviderRegistered {
    pub provider: Pubkey,
}

// Error types
#[error_code]
pub enum LogisticsError {
    #[msg("Mismatched arrays length")]
    MismatchedArrays,
    #[msg("No logistics providers provided")]
    NoLogisticsProviders,
    #[msg("Too many logistics providers")]
    TooManyProviders,
    #[msg("Invalid quantity")]
    InvalidQuantity,
    #[msg("Trade is inactive")]
    TradeInactive,
    #[msg("Insufficient quantity available")]
    InsufficientQuantity,
    #[msg("Buyer cannot be the seller")]
    BuyerIsSeller,
    #[msg("Invalid logistics provider")]
    InvalidLogisticsProvider,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Already confirmed")]
    AlreadyConfirmed,
    #[msg("Purchase is disputed")]
    Disputed,
    #[msg("Already settled")]
    AlreadySettled,
    #[msg("Already disputed")]
    AlreadyDisputed,
    #[msg("Not disputed")]
    NotDisputed,
    #[msg("Invalid winner")]
    InvalidWinner,
    #[msg("No fees to withdraw")]
    NoFeesToWithdraw,
}

fn main() {
    println!("DezenMart Logistics Smart Contract");
}