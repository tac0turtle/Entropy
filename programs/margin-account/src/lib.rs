#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use solana_program::program::invoke;

#[program]
pub mod margin_account {
    use super::*;

    /// Initialize new margin account under a specific trader's address.
    pub fn initialize(ctx: Context<Initialize>, trader: Pubkey) -> ProgramResult {
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.trader = trader;
        Ok(())
    }
    /// Initialize a collateral account to be used to open a position.
    pub fn init_obligation(ctx: Context<InitObligation>) -> ProgramResult {
        // Initialize the obligation through the token lending program.
        invoke(
            &spl_token_lending::instruction::init_obligation(
                *ctx.accounts.lending_program.key,
                *ctx.accounts.deposit_reserve.key,
                *ctx.accounts.borrow_reserve.key,
                *ctx.accounts.lending_market.key,
                *ctx.accounts.obligation.key,
                *ctx.accounts.obligation_token_mint.key,
                *ctx.accounts.obligation_token_output.key,
                *ctx.accounts.obligation_token_owner.key,
            ),
            &ctx.accounts.to_account_infos(),
        )?;
        Ok(())
    }
    /// Open a leveraged position on serum.
    pub fn open_position_amm(
        ctx: Context<OpenPositionAMM>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ProgramResult {
        let swap = spl_token_swap::instruction::Swap {
            amount_in,
            minimum_amount_out,
        };

        let instruction = &spl_token_swap::instruction::swap(
            &ctx.accounts.user_authority.key,
            &ctx.accounts.token_program.key,
            &ctx.accounts.swap_info.key,
            &ctx.accounts.swap_authority.key,
            &ctx.accounts.user_authority.key,
            &ctx.accounts.swap_source.key,
            &ctx.accounts.swap_source.key,
            &ctx.accounts.swap_dest.key,
            &ctx.accounts.dest.key,
            &ctx.accounts.pool_mint.key,
            &ctx.accounts.pool_fee.key,
            &ctx.accounts.host_fee.key,
            swap,
        )?;

        invoke(instruction, &ctx.accounts.to_account_infos())?;
        Ok(())
    }
    /// Close an open leveraged position.
    pub fn close_position(_ctx: Context<ClosePosition>) -> ProgramResult {
        // TODO
        Ok(())
    }
    /// Withdraw funds from an obligation account.
    pub fn withdraw(_ctx: Context<Withdraw>) -> ProgramResult {
        // TODO
        Ok(())
    }
    /// Liquidate a position if below liquidation price.
    //? This potentially isn't needed, if the logic can happen on the lending pool (obligation
    //? account) but for now it's assumed the transaction will have to go through here.
    pub fn liquidate(_ctx: Context<Liquidate>) -> ProgramResult {
        // TODO
        Ok(())
    }
}

/// Initializes new margin account.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    rent: Sysvar<'info, Rent>,
}

/// Initialize new margin collateral obligation.
#[derive(Accounts)]
pub struct InitObligation<'info> {
    lending_program: AccountInfo<'info>,
    deposit_reserve: AccountInfo<'info>,
    borrow_reserve: AccountInfo<'info>,
    // ? This probably needs to be initialized
    obligation: AccountInfo<'info>,
    // ?
    #[account(mut)]
    obligation_token_mint: AccountInfo<'info>,
    // ?
    #[account(mut)]
    obligation_token_output: AccountInfo<'info>,
    obligation_token_owner: AccountInfo<'info>,
    lending_market: AccountInfo<'info>,
    lending_market_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct OpenPositionAMM<'info> {
    /// accounts needed to call
    #[account(signer)]
    user_authority: AccountInfo<'info>,
    swap_info: AccountInfo<'info>,
    swap_authority: AccountInfo<'info>,
    #[account(mut)]
    source: AccountInfo<'info>,
    #[account(mut)]
    swap_source: AccountInfo<'info>,
    #[account(mut)]
    swap_dest: AccountInfo<'info>,
    #[account(mut)]
    dest: AccountInfo<'info>,
    #[account(mut)]
    pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pool_fee: AccountInfo<'info>,
    #[account(mut)]
    host_fee: AccountInfo<'info>,
    /// accounts needed to access funds from token vault
    #[account(mut)]
    margin_account: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    // TODO
    #[account(signer)]
    authority: AccountInfo<'info>,
}

//? Possibly add cancel position, if it cannot be combined with close.

#[derive(Accounts)]
pub struct Liquidate<'info> {
    // TODO
    authority: AccountInfo<'info>,
}

/// Margin account state which keeps track of positions opened for a given trader.
#[account]
pub struct MarginAccount {
    /// The owner of this margin account.
    pub trader: Pubkey,
    /// Open positions held by the margin account.
    pub positions: Vec<Position>,
    /// Address of token vault
    pub vault: Pubkey,
    /// nonce for program derived address
    pub nonce: u8,
}

/// Open margin trade position.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Position {
    /// Program address for obligation account used as collateral.
    pub obligation_account: Pubkey,
    /// Indicates whether an obligation account has been used to open a leveraged position.
    pub open: bool,
}
