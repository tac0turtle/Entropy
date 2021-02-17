#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};

#[program]
pub mod margin_account {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> ProgramResult {
        // TODO
        Ok(())
    }
    pub fn deposit(_ctx: Context<Deposit>) -> ProgramResult {
        // TODO
        Ok(())
    }
    pub fn withdraw(_ctx: Context<Withdraw>) -> ProgramResult {
        // TODO
        Ok(())
    }
    pub fn trade(_ctx: Context<Trade>) -> ProgramResult {
        // TODO
        Ok(())
    }
    pub fn liquidate(_ctx: Context<Trade>) -> ProgramResult {
        // TODO
        Ok(())
    }
    pub fn deleverage(_ctx: Context<Trade>) -> ProgramResult {
        // TODO
        Ok(())
    }
}

/// Initializes new margin account.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    /// Authority (trader)
    // TODO check this
    #[account(mut)]
    trader: AccountInfo<'info>,
    /// Coordinator address
    coordinator: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,

    // Misc
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
}

/// Deposit funds into program account to be used for trading.
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    /// Coordinator address
    coordinator: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    // Coordinator address
    coordinator: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Trade<'info> {
    // TODO
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    // TODO
    #[account(signer)]
    authority: AccountInfo<'info>,
}

/// Margin account which handles
#[account]
pub struct MarginAccount {
    /// The owner of this Vesting account.
    pub trader: Pubkey,
    /// The mint of the SPL token locked up.
    pub mint: Pubkey,
    /// Address of the account's token vault.
    pub vault: Pubkey,
    /// Coordinator has the write to call functions within this program
    pub coordinator: Pubkey,
    /// The starting balance of this vesting account, i.e., how much was
    /// originally deposited.
    pub initial_balance: u64,
    /// Signer nonce.
    pub nonce: u8,
    /// Check if there is an open trade.
    pub open_trade: bool,
}
