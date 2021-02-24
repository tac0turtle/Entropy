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
        _ctx: Context<OpenPositionAMM>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ProgramResult {
        invoke(&spl_token_swap::instruction::swap())?;
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
    // TODO
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
    pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pool_fee: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

///   3. `[writable]` token_(A|B) SOURCE Account, amount is transferable by user transfer authority,
///   4. `[writable]` token_(A|B) Base Account to swap INTO.  Must be the SOURCE token.
///   5. `[writable]` token_(A|B) Base Account to swap FROM.  Must be the DESTINATION token.
///   6. `[writable]` token_(A|B) DESTINATION Account assigned to USER as the owner.
///   7. `[writable]` Pool token mint, to generate trading fees
///   8. `[writable]` Fee account, to receive trading fees
///   9. '[]` Token program id
///   10 `[optional, writable]` Host fee account to receive additional trading fees

//         let source_info = next_account_info(account_info_iter)?;
//         let swap_source_info = next_account_info(account_info_iter)?;
//         let swap_destination_info = next_account_info(account_info_iter)?;
//         let destination_info = next_account_info(account_info_iter)?;
//         let pool_mint_info = next_account_info(account_info_iter)?;
//         let pool_fee_account_info = next_account_info(account_info_iter)?;
//         let token_program_info = next_account_info(account_info_iter)?;

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
}

/// Open margin trade position.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Position {
    /// Program address for obligation account used as collateral.
    pub obligation_account: Pubkey,
    /// Indicates whether an obligation account has been used to open a leveraged position.
    pub open: bool,
}
