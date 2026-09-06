use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        self,
        CloseAccount,
        Mint,
        Token,
        TokenAccount,
        Transfer,
    },
};

declare_id!("BVq1BrnEdBuhHfauPXynkdpg3dK1eYTjKMbhnY1Mws74");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        deposit_amount: u64,
        receive_amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        escrow.maker = ctx.accounts.maker.key();
        escrow.mint_a = ctx.accounts.mint_a.key();
        escrow.mint_b = ctx.accounts.mint_b.key();
        escrow.deposit_amount = deposit_amount;
        escrow.receive_amount = receive_amount;
        escrow.seed = seed;
        escrow.bump = ctx.bumps.escrow;

        // Transfer token A from maker into the escrow vault.
        let transfer_accounts = Transfer {
            from: ctx.accounts.maker_ata_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.key(),
            transfer_accounts,
        );

        token::transfer(cpi_ctx, deposit_amount)?;

        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;

        // --------------------------------------------------
        // 1. Taker pays maker in token B.
        // --------------------------------------------------

        let payment_accounts = Transfer {
            from: ctx.accounts.taker_ata_b.to_account_info(),
            to: ctx.accounts.maker_ata_b.to_account_info(),
            authority: ctx.accounts.taker.to_account_info(),
        };

        let payment_ctx = CpiContext::new(
            ctx.accounts.token_program.key(),
            payment_accounts,
        );

        token::transfer(
            payment_ctx,
            escrow.receive_amount,
        )?;

        // --------------------------------------------------
        // 2. Escrow PDA signs to release token A.
        // --------------------------------------------------

        let seed_bytes = escrow.seed.to_le_bytes();
        let bump = [escrow.bump];

        let signer_seeds: &[&[u8]] = &[
            b"escrow",
            escrow.maker.as_ref(),
            seed_bytes.as_ref(),
            bump.as_ref(),
        ];

        let signer = &[signer_seeds];

        let release_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.taker_ata_a.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let release_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            release_accounts,
            signer,
        );

        token::transfer(
            release_ctx,
            escrow.deposit_amount,
        )?;

        // --------------------------------------------------
        // 3. Close the empty vault.
        // --------------------------------------------------

        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            close_accounts,
            signer,
        );

        token::close_account(close_ctx)?;

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;

        let seed_bytes = escrow.seed.to_le_bytes();
        let bump = [escrow.bump];

        let signer_seeds: &[&[u8]] = &[
            b"escrow",
            escrow.maker.as_ref(),
            seed_bytes.as_ref(),
            bump.as_ref(),
        ];

        let signer = &[signer_seeds];

        // --------------------------------------------------
        // 1. Return token A to maker.
        // --------------------------------------------------

        let transfer_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.maker_ata_a.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            transfer_accounts,
            signer,
        );

        token::transfer(
            transfer_ctx,
            escrow.deposit_amount,
        )?;

        // --------------------------------------------------
        // 2. Close vault.
        // --------------------------------------------------

        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            close_accounts,
            signer,
        );

        token::close_account(close_ctx)?;

        Ok(())
    }
}


// ==================================================
// Make
// ==================================================

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [
            b"escrow",
            maker.key().as_ref(),
            seed.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = maker,
        token::mint = mint_a,
        token::authority = escrow,
        seeds = [
            b"vault",
            escrow.key().as_ref(),
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


// ==================================================
// Take
// ==================================================

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(
        mut,
        address = escrow.maker,
    )]
    pub maker: SystemAccount<'info>,

    #[account(
        address = escrow.mint_a,
    )]
    pub mint_a: Account<'info, Mint>,

    #[account(
        address = escrow.mint_b,
    )]
    pub mint_b: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            b"escrow",
            escrow.maker.as_ref(),
            escrow.seed.to_le_bytes().as_ref(),
        ],
        bump = escrow.bump,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        close = maker,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [
            b"vault",
            escrow.key().as_ref(),
        ],
        bump,
        token::mint = mint_a,
        token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
    )]
    pub maker_ata_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}


// ==================================================
// Refund
// ==================================================

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        address = escrow.mint_a,
    )]
    pub mint_a: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            b"escrow",
            maker.key().as_ref(),
            escrow.seed.to_le_bytes().as_ref(),
        ],
        bump = escrow.bump,
        has_one = maker,
        has_one = mint_a,
        close = maker,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [
            b"vault",
            escrow.key().as_ref(),
        ],
        bump,
        token::mint = mint_a,
        token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}


// ==================================================
// State
// ==================================================

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Pubkey,

    pub mint_a: Pubkey,
    pub mint_b: Pubkey,

    pub deposit_amount: u64,
    pub receive_amount: u64,

    pub seed: u64,
    pub bump: u8,
}