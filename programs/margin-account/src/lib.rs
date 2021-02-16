#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;

#[program]
pub mod margin_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> ProgramResult {
        let borrower_account = &mut ctx.accounts.borrower_account;
        borrower_account.data = data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    pub borrower_account: ProgramAccount<'info, BorrowerAccount>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct BorrowerAccount {
    // TODO remove: this data is irrelevant, just used for template
    pub data: u64,
}
