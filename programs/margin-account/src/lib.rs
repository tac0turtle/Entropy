#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use solana_program::program::{invoke, invoke_signed};

#[program]
pub mod margin_account {
    use super::*;

    /// Initialize new margin account under a specific trader's address.
    #[access_control(Initialize::accounts(&ctx, nonce))]
    pub fn initialize(ctx: Context<Initialize>, trader: Pubkey, nonce: u8) -> ProgramResult {
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.trader = trader;
        margin_account.nonce = nonce;

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

    /// Open a leveraged position on an AMM.
    pub fn open_position_amm(
        ctx: Context<OpenPositionAMM>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ProgramResult {
        if amount_in == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        };
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
            ctx.accounts.loaned_vault.to_account_info().key,
            ctx.accounts.swap_source.key,
            ctx.accounts.swap_dest.key,
            ctx.accounts.collateral_vault.to_account_info().key,
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

        invoke_signed(instruction, &ctx.accounts.to_account_infos(), signer)?;

        // Mark account as having an open trade
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.position.collateral_vault =
            *ctx.accounts.collateral_vault.to_account_info().key;

        Ok(())
    }

    /// Close an open leveraged position on an AMM.
    pub fn close_position_amm(
        ctx: Context<ClosePositionAMM>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ProgramResult {
        if amount_in == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        };
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
            ctx.accounts.collateral_vault.to_account_info().key,
            ctx.accounts.swap_source.key,
            ctx.accounts.swap_dest.key,
            ctx.accounts.loaned_vault.to_account_info().key,
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

        invoke_signed(instruction, &ctx.accounts.to_account_infos(), signer)?;

        Ok(())
    }

    /// repay repays the outstanding loan. If the user is not able to return what they took out it is taken from the collateral
    pub fn repay(ctx: Context<Repay>, amount: u64) -> ProgramResult {
        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        };

        let instruction = &spl_token_lending::instruction::repay_reserve_liquidity(
            *ctx.accounts.lending_program.key,
            amount,
            *ctx.accounts.loan_vault.to_account_info().key,
            *ctx.accounts.destination_coll_account.key,
            *ctx.accounts.repay_reserve_account.key,
            *ctx.accounts.repay_reserve_spl_acccount.key,
            *ctx.accounts.withdraw_reserve.key,
            *ctx.accounts.withdraw_reserve_collateral.key,
            *ctx.accounts.obligation.key,
            *ctx.accounts.obligation_mint.key,
            *ctx.accounts.obligation_input.key,
            *ctx.accounts.lending_market.key,
            *ctx.accounts.derived_lending_authority.key,
            *ctx.accounts.vault_signer.key,
        );

        let seeds = &[
            ctx.accounts.margin_account.to_account_info().key.as_ref(),
            &[ctx.accounts.margin_account.nonce],
        ];
        let signer = &[&seeds[..]];

        invoke_signed(instruction, &ctx.accounts.to_account_infos(), signer)?;

        // Mark account as having an open trade
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.position.loan_amount -= amount;
        if margin_account.position.loan_amount == 0 {
            margin_account.position.status = Status::Available;
        }

        Ok(())
    }

    pub fn borrow(ctx: Context<Borrow>, loan_amount: u64, collateral_amount: u64) -> ProgramResult {
        if loan_amount == 0 || collateral_amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        };

        let accounts = ctx.accounts.to_account_infos();

        let instruction = &spl_token_lending::instruction::margin_borrow_reserve_liquidity(
            *ctx.accounts.lending_program.key,
            collateral_amount,
            loan_amount,
            spl_token_lending::instruction::BorrowAmountType::MarginBorrowAmount,
            *ctx.accounts.source_collateral.key,
            *ctx.accounts.loaned_vault.to_account_info().key,
            *ctx.accounts.deposit_reserve.key,
            *ctx.accounts.deposit_reserve_collateral_supply.key,
            *ctx.accounts.deposit_reserve_collateral_fees_receiver.key,
            *ctx.accounts.borrow_reserve.key,
            *ctx.accounts.borrow_reserve_liquidity_supply.key,
            *ctx.accounts.lending_market.key,
            *ctx.accounts.lending_market_authority.key,
            *ctx.accounts.vault_signer.key,
            *ctx.accounts.obligation.key,
            *ctx.accounts.obligation_token_mint.key,
            *ctx.accounts.obligation_token_output.key,
            *ctx.accounts.dex_market.key,
            *ctx.accounts.dex_market_order_book_side.key,
            *ctx.accounts.memory.key,
            None,
        );

        let seeds = &[
            ctx.accounts.margin_account.to_account_info().key.as_ref(),
            &[ctx.accounts.margin_account.nonce],
        ];
        let signer = &[&seeds[..]];

        invoke_signed(instruction, &accounts[1..], signer)?;

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
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    fn accounts(ctx: &Context<Initialize>, nonce: u8) -> Result<()> {
        let margin_authority = Pubkey::create_program_address(
            &[
                ctx.accounts.margin_account.to_account_info().key.as_ref(),
                &[nonce],
            ],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidProgramAddress)?;
        if ctx.accounts.vault.owner != margin_authority {
            return Err(ErrorCode::InvalidVaultOwner)?;
        }

        Ok(())
    }
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

// OpenPositionAMM takes the tokens that are in the margin account and executes a trade with them.
#[derive(Accounts)]
pub struct OpenPositionAMM<'info> {
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
    loaned_vault: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    collateral_vault: CpiAccount<'info, TokenAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

// ClosePositionAMM call an amm to close the entire position or only enough to repay the loan, if in profit
#[derive(Accounts)]
pub struct ClosePositionAMM<'info> {
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
    #[account(mut)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    #[account(mut)]
    loaned_vault: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    collateral_vault: CpiAccount<'info, TokenAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,

    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    lending_program: AccountInfo<'info>,
    /// Account which is repaying the loan.
    #[account(mut)]
    source_liquidity_acc: AccountInfo<'info>,
    /// This account specifies where to send the obligation account after repay
    #[account(mut)]
    destination_coll_account: AccountInfo<'info>,
    #[account(mut)]
    repay_reserve_account: AccountInfo<'info>,
    #[account(mut)]
    repay_reserve_spl_acccount: AccountInfo<'info>,
    withdraw_reserve: AccountInfo<'info>,
    /// User token account to withdraw obligation to
    #[account(mut)]
    withdraw_reserve_collateral: AccountInfo<'info>,
    #[account(mut)]
    obligation: AccountInfo<'info>,
    #[account(mut)]
    obligation_mint: AccountInfo<'info>,
    #[account(mut)]
    obligation_input: AccountInfo<'info>,
    lending_market: AccountInfo<'info>,
    derived_lending_authority: AccountInfo<'info>,

    /// Loan vault are the tokens that will be used to repay the loan.
    #[account(mut)]
    loan_vault: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    // specify the correct lending program
    lending_program: AccountInfo<'info>,
    #[account(mut)]
    source_collateral: AccountInfo<'info>,
    deposit_reserve: AccountInfo<'info>,
    #[account(mut)]
    deposit_reserve_collateral_supply: AccountInfo<'info>,
    #[account(mut)]
    deposit_reserve_collateral_fees_receiver: AccountInfo<'info>,
    #[account(mut)]
    borrow_reserve: AccountInfo<'info>,
    #[account(mut)]
    borrow_reserve_liquidity_supply: AccountInfo<'info>,
    lending_market: AccountInfo<'info>,
    lending_market_authority: AccountInfo<'info>,
    obligation: AccountInfo<'info>,
    obligation_token_mint: AccountInfo<'info>,
    obligation_token_output: AccountInfo<'info>,
    memory: AccountInfo<'info>,
    dex_market: AccountInfo<'info>,
    dex_market_order_book_side: AccountInfo<'info>,

    /// User transfer authority
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,
    #[account(mut)]
    loaned_vault: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    margin_account: ProgramAccount<'info, MarginAccount>,
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
    /// Position is a wrapper around an open position. This position
    pub position: Position,
    /// nonce for program derived address
    pub nonce: u8,
}

/// Open margin trade position.
#[account]
pub struct Position {
    /// Tracks the size of the loan to know if the amount being paid back is the total amount in order to unlock the account
    pub loan_amount: u64,
    /// Open positions held by the margin account.
    // This account holds tokens from the loan before they are used in the trade and conversely to hold
    // tokens after closing the position and before repaying the loan.
    pub loaned_vault: Pubkey,
    // Tokens are stored here when a position is opened (status becomes locked). When the loan is repaid,
    // status is updated to available and the trader is able to withdraw the tokens.
    pub collateral_vault: Pubkey,
    // When a position is open, status is locked meaning funds can't be withdrawn. Once a position is closed out,
    // status is updated to available indicating that the trader can now withdraw the tokens.
    pub status: Status,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Status {
    Locked,
    Available,
}

#[error]
pub enum ErrorCode {
    #[msg("Invalid program address. Did you provide the correct nonce?")]
    InvalidProgramAddress,
    #[msg("Invalid margin owner.")]
    InvalidVaultOwner,
    #[msg("Amount has to be greater than 0")]
    InvalidAmount,
}
