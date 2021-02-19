#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};

#[program]
pub mod margin_account {
    use super::*;

    /// Initialize new margin account under a specific trader's address.
    pub fn initialize(ctx: Context<Initialize>, trader: Pubkey) -> ProgramResult {
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.trader = trader;
        Ok(())
    }
    /// Deposit funds into a trader's margin account to use to trade.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> ProgramResult {
        // Transfer funds to the pool to be used for margin trades.
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Add position to the margin account state
        let mint = ctx.accounts.from.mint;
        if let Some(pos) = ctx
            .accounts
            .margin_account
            .tokens
            .iter_mut()
            .find(|p| p.mint == mint)
        {
            // Tokens are already deposited, add to this amount
            pos.amount += amount;
        } else {
            // No tokens for this denom are deposited, create new
            ctx.accounts
                .margin_account
                .tokens
                .push(TokenDeposit { mint, amount });
        }

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

/// Deposit funds into program account to be used for trading.
#[derive(Accounts)]
pub struct Deposit<'info> {
    // TODO correct access control
    #[account(mut)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    /// Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    /// Token account the check is made from.
    #[account(mut, "from.mint == vault.mint")]
    from: CpiAccount<'info, TokenAccount>,
    /// Token vault for the trader's margin fund.
    // TODO is the vault owned by the signer or liquidity pool?
    #[account(mut, "&vault.owner == authority.key")]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
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
    authority: AccountInfo<'info>,
}

/// Margin account which accepts token deposits and allows the trader to open
/// margin positions.
#[account]
pub struct MarginAccount {
    /// The owner of this margin account.
    pub trader: Pubkey,
    /// Address of the account's token vault.
    // ! This probably won't be needed?
    pub vault: Pubkey,
    /// Token balances available to the margin account.
    pub tokens: Vec<TokenDeposit>,
    /// Open positions held by the margin account.
    pub positions: Vec<Position>,
}

/// Deposits for the account to be used for opening trades.
#[derive(Default, Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TokenDeposit {
    /// Address of the token program for the position
    pub mint: Pubkey,
    /// Amount of minted tokens are added.
    pub amount: u64,
}

/// Open margin trade position.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Position {
    // TODO
}
