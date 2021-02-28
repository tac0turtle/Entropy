#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use solana_program::program::{invoke, invoke_signed};

#[program]
pub mod margin_account {
    use super::*;

    /// Initialize new margin account under a specific trader's address.
    // #[access_control(Initialize::accounts(&ctx, nonce))]
    pub fn initialize(ctx: Context<Initialize>, trader: Pubkey, nonce: u8) -> ProgramResult {
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.trader = trader;
        margin_account.nonce = nonce;

        Ok(())
    }

    /// Initialize a collateral account to be used to open a position.
    pub fn init_obligation(ctx: Context<InitObligation>) -> ProgramResult {
        let accounts = ctx.accounts.to_account_infos();
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
            // First account here is the lending program account, but indexes for obligation call
            // need to be indexed correctly
            &accounts[1..],
        )?;

        Ok(())
    }

    /// Trade on an amm with the loaned tokens.
    pub fn trade_amm(
        ctx: Context<TradeAMM>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ProgramResult {
        let accounts = ctx.accounts.to_account_infos();

        // create the desired swap amount and minimum amout of slippage the user is willing to sustain
        let swap = spl_token_swap::instruction::Swap {
            amount_in,
            minimum_amount_out,
        };

        let instruction = &spl_token_swap::instruction::swap(
            ctx.accounts.swap_program.key,
            ctx.accounts.token_program.key,
            ctx.accounts.swap_info.key,
            ctx.accounts.swap_authority.key,
            ctx.accounts.vault_signer.key,
            ctx.accounts.source_vault.to_account_info().key,
            ctx.accounts.swap_source.key,
            ctx.accounts.swap_dest.key,
            ctx.accounts.destination_vault.to_account_info().key,
            ctx.accounts.pool_mint.key,
            ctx.accounts.pool_fee.key,
            Some(ctx.accounts.host_fee.key),
            swap,
        )?;

        let seeds = &[
            ctx.accounts.margin_account.to_account_info().key.as_ref(),
            &[ctx.accounts.margin_account.nonce],
        ];
        let signer = &[&seeds[..]];

        invoke_signed(instruction, &accounts[1..], signer)?;

        // Mark account as having an open trade
        let margin_account = &mut ctx.accounts.margin_account;
        let position = margin_account
            .position
            .as_mut()
            .ok_or(ErrorCode::InvalidProgramAddress)?;
        if !position.collateral_vault.is_some() {
            position.collateral_vault = Some(*ctx.accounts.destination_vault.to_account_info().key);
        }

        Ok(())
    }

    pub fn repay(_ctx: Context<Repay>, _amount: u64) -> ProgramResult {
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
    #[account(mut)]
    obligation: AccountInfo<'info>,
    #[account(mut)]
    obligation_token_mint: AccountInfo<'info>,
    #[account(mut)]
    obligation_token_output: AccountInfo<'info>,
    obligation_token_owner: AccountInfo<'info>,
    lending_market: AccountInfo<'info>,
    lending_market_authority: AccountInfo<'info>,

    //? These may not be needed, but missing an account on CPI call
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
    #[account("token_program.key == &token::ID")]
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
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

// TradeAMM takes the tokens that are in the margin account and executes a trade with them.
#[derive(Accounts)]
pub struct TradeAMM<'info> {
    #[account(signer)]
    trader: AccountInfo<'info>,
    /// accounts needed to call
    swap_program: AccountInfo<'info>,
    swap_info: AccountInfo<'info>,
    swap_authority: AccountInfo<'info>,
    #[account(mut)]
    source: AccountInfo<'info>,
    #[account(mut)]
    swap_source: AccountInfo<'info>,
    #[account(mut)]
    swap_dest: AccountInfo<'info>,
    #[account(mut)]
    pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pool_fee: AccountInfo<'info>,
    host_fee: AccountInfo<'info>,
    /// accounts needed to access funds from token vault
    #[account(mut, has_one = trader)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    #[account(mut)]
    source_vault: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    destination_vault: CpiAccount<'info, TokenAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

//? Possibly add cancel position, if it cannot be combined with close.

#[derive(Accounts)]
pub struct Repay<'info> {
    // TODO
    authority: AccountInfo<'info>,
}
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
    pub position: Option<Position>,

    /// nonce for program derived address
    pub nonce: u8,
}

/// Track margin account position.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Position {
    /// Tracks the size of the loan to know if the amount being paid back is the total amount in order to unlock the account
    pub loan_amount: u64,
    // This account holds tokens from the loan before they are used in the trade and conversely to hold
    // tokens after closing the position and before repaying the loan.
    pub loaned_vault: Pubkey,
    // Tokens are stored here when a position is opened (status becomes locked). When the loan is repaid,
    // status is updated to available and the trader is able to withdraw the tokens.
    pub collateral_vault: Option<Pubkey>,
    // When a position is open, status is locked meaning funds can't be withdrawn. Once a position is closed out,
    // status is updated to available indicating that the trader can now withdraw the tokens.
    pub status: Status,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Status {
    Locked,
    Available,
}

impl Default for Status {
    fn default() -> Status {
        Status::Available
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Invalid program address. Did you provide the correct nonce?")]
    InvalidProgramAddress,
    #[msg("Invalid margin owner.")]
    InvalidVaultOwner,
}
