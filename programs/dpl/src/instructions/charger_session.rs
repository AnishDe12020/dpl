use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{BONK_PRICE_PER_M, BONK_TOKEN_MINT, DEFAULT_TOKEN_MINT},
    errors::DplError,
    state::charger::Charger,
};

pub fn charger_session_ix(ctx: Context<ChargerSession>, amount: u64) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let user = &ctx.accounts.user;
    let user_ata = &ctx.accounts.user_ata;
    let operator_ata = &ctx.accounts.operator_ata;
    let nft_mint_owner_ata = &ctx.accounts.nft_mint_owner_ata;
    let token_program = ctx.accounts.token_program.to_account_info();
    let charger_pda = &mut ctx.accounts.charger_pda;

    let bonk_mint = &ctx.accounts.bonk_mint;
    let bonk_vault_authority = &ctx.accounts.bonk_vault_authority;
    let bonk_vault_ata = &ctx.accounts.bonk_vault_ata;
    let bonk_receiver_ata = &ctx.accounts.bonk_receiver_ata;

    require!(
        mint.key() == Pubkey::from_str(DEFAULT_TOKEN_MINT).unwrap(),
        DplError::InvalidMint
    );

    require!(
        bonk_mint.key() == Pubkey::from_str(BONK_TOKEN_MINT).unwrap(),
        DplError::InvalidMint
    );

    require!(amount > 0, DplError::InvalidAmount);
    require!(user_ata.amount >= amount as u64, DplError::InvalidAmount);

    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: user_ata.to_account_info(),
                mint: mint.to_account_info(),
                to: operator_ata.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        amount * 0.3 as u64 * 10_u64.pow(mint.decimals.into()),
        mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: user_ata.to_account_info(),
                mint: mint.to_account_info(),
                to: nft_mint_owner_ata.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        amount * 0.7 as u64 * 10_u64.pow(mint.decimals.into()),
        mint.decimals,
    )?;

    // This token transfer is for the BONK airdrop which is ~10% of the total amount in BONK tokens. We are currently using a hardcoded value of $BONK_PRICE_PER_M for 1M BONK.
    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: bonk_vault_ata.to_account_info(),
                mint: bonk_mint.to_account_info(),
                to: bonk_receiver_ata.to_account_info(),
                authority: bonk_vault_authority.to_account_info(),
            },
        ),
        // ((f64::from(amount) * 0.1) / BONK_PRICE) as u64 * 10_u64.pow(bonk_mint.decimals.into()), // TODO: update
        amount * 0.1 as u64 / BONK_PRICE_PER_M as u64
            * 10_u64.pow(6)
            * 10_u64.pow(bonk_mint.decimals.into()),
        bonk_mint.decimals,
    )?;

    charger_pda.all_time_revenue += amount;

    Ok(())
}

#[derive(Accounts)]
pub struct ChargerSession<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        token::mint = mint,
        token::authority = user
    )]
    pub user_ata: Account<'info, TokenAccount>,
    /// CHECK: charger unsafe acc
    pub charger: AccountInfo<'info>,
    #[account(
        seeds = [b"charger", charger.key().as_ref()],
        bump
    )]
    pub charger_pda: Account<'info, Charger>,
    pub mint: Account<'info, Mint>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: nft_mint_owner unsafe acc
    pub nft_mint_owner: AccountInfo<'info>,
    #[account(
        token::mint = mint,
        token::authority = nft_mint_owner
    )]
    pub nft_mint_owner_ata: Account<'info, TokenAccount>,
    /// CHECK:
    pub operator: AccountInfo<'info>,
    #[account(
        token::mint = mint,
        token::authority = operator
    )]
    pub operator_ata: Account<'info, TokenAccount>,
    pub bonk_mint: Account<'info, Mint>,
    #[account(mut)]
    pub bonk_vault_authority: Signer<'info>,
    #[account(
        token::mint = bonk_mint,
        token::authority = bonk_vault_authority
    )]
    pub bonk_vault_ata: Account<'info, TokenAccount>,
    #[account(
        token::mint = bonk_mint,
        token::authority = user
    )]
    pub bonk_receiver_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
