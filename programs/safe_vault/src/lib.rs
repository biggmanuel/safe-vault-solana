use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("HSYGkUaRWNa3ia3mYmCsRpQwU28rFvR5WgATwwvdSacA"); // <--- REMEMBER TO UPDATE THIS WITH YOUR ID

#[program]
pub mod safe_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault_account = &mut ctx.accounts.vault_account;
        vault_account.total_collateral = 0;
        vault_account.total_borrowed = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let user_account = &mut ctx.accounts.user_account;
        let vault = &mut ctx.accounts.vault_account;

        user_account.collateral_amount += amount;
        vault.total_collateral += amount;

        Ok(())
    }

    pub fn borrow(ctx: Context<Borrow>, amount_to_borrow: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let vault = &mut ctx.accounts.vault_account;

        // 1. Mock Oracle & Risk Check
        let price = 100; 
        let max_loan_value = (user_account.collateral_amount * price) / 2;
        
        require!(
            user_account.borrowed_amount + amount_to_borrow <= max_loan_value,
            ErrorCode::InsufficientCollateral
        );

        user_account.borrowed_amount += amount_to_borrow;
        vault.total_borrowed += amount_to_borrow;

        // 2. Transfer with PDA Signer
        // FIX: We use .as_ref() to make the types match (both become &[u8])
        let seeds = &[
            b"vault_tokens".as_ref(), 
            &[ctx.bumps.vault_token_account]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_token_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount_to_borrow)?;

        Ok(())
    }
}

// --- DATA STRUCTURES ---

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = user, 
        space = 8 + 8 + 8,
        seeds = [b"vault_state"],
        bump
    )]
    pub vault_account: Account<'info, VaultAccount>,
    
    // We init the vault's token wallet here so it exists
    #[account(
        init,
        payer = user,
        seeds = [b"vault_tokens"],
        bump,
        token::mint = mint,
        token::authority = vault_token_account, // The account controls itself via PDA
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    pub mint: Account<'info, token::Mint>, // The token we are lending
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault_state"],
        bump
    )]
    pub vault_account: Account<'info, VaultAccount>,
    
    #[account(
        init_if_needed, 
        payer = user, 
        space = 8 + 32 + 8 + 8,
        seeds = [b"user-stats", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserStats>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vault_tokens"],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>, 
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(
        mut,
        seeds = [b"vault_state"],
        bump
    )]
    pub vault_account: Account<'info, VaultAccount>,
    
    #[account(
        mut,
        seeds = [b"user-stats", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserStats>,
    
    // ADDED SEEDS HERE -> This fixes your "BorrowBumps" error
    #[account(
        mut,
        seeds = [b"vault_tokens"],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>, 
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct VaultAccount {
    pub total_collateral: u64,
    pub total_borrowed: u64,
}

#[account]
pub struct UserStats {
    pub authority: Pubkey,
    pub collateral_amount: u64,
    pub borrowed_amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient collateral to borrow this amount.")]
    InsufficientCollateral,
}