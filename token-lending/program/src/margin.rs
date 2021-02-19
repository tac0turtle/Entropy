use crate::{
    dex_market::TradeSimulator,
    error::LendingError,
    helper::{
        assert_last_update_slot, spl_token_burn, spl_token_mint_to, spl_token_transfer,
        unpack_mint, TokenBurnParams, TokenMintToParams, TokenTransferParams,
    },
    math::Decimal,
    state::{LendingMarket, Obligation, RepayResult, Reserve},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

#[inline(never)] // avoid stack frame limit
pub fn process_margin_repay(
    program_id: &Pubkey,
    liquidity_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if liquidity_amount == 0 {
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter();
    let source_liquidity_info = next_account_info(account_info_iter)?;
    let destination_collateral_info = next_account_info(account_info_iter)?;
    let repay_reserve_info = next_account_info(account_info_iter)?;
    let repay_reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_info = next_account_info(account_info_iter)?;
    let withdraw_reserve_collateral_supply_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let obligation_token_mint_info = next_account_info(account_info_iter)?;
    let obligation_token_input_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    if obligation_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &obligation.borrow_reserve != repay_reserve_info.key {
        msg!("Invalid repay reserve account");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &obligation.collateral_reserve != withdraw_reserve_info.key {
        msg!("Invalid withdraw reserve account");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if obligation.deposited_collateral_tokens == 0 {
        return Err(LendingError::ObligationEmpty.into());
    }

    let obligation_mint = unpack_mint(&obligation_token_mint_info.data.borrow())?;
    if &obligation.token_mint != obligation_token_mint_info.key {
        msg!("Invalid obligation token mint account");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let mut repay_reserve = Reserve::unpack(&repay_reserve_info.data.borrow())?;
    if repay_reserve_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &repay_reserve.lending_market != lending_market_info.key {
        msg!("Invalid reserve lending market account");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let withdraw_reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
    if withdraw_reserve_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if withdraw_reserve.lending_market != repay_reserve.lending_market {
        return Err(LendingError::LendingMarketMismatch.into());
    }

    if repay_reserve_info.key == withdraw_reserve_info.key {
        return Err(LendingError::DuplicateReserve.into());
    }
    if repay_reserve.liquidity.mint_pubkey == withdraw_reserve.liquidity.mint_pubkey {
        return Err(LendingError::DuplicateReserveMint.into());
    }
    if &repay_reserve.liquidity.supply_pubkey != repay_reserve_liquidity_supply_info.key {
        msg!("Invalid repay reserve liquidity supply account");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey != withdraw_reserve_collateral_supply_info.key {
        msg!("Invalid withdraw reserve collateral supply account");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &repay_reserve.liquidity.supply_pubkey == source_liquidity_info.key {
        msg!("Cannot use repay reserve liquidity supply as source account input");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &withdraw_reserve.collateral.supply_pubkey == destination_collateral_info.key {
        msg!("Cannot use withdraw reserve collateral supply as destination account input");
        return Err(LendingError::InvalidAccountInput.into());
    }

    // accrue interest and update rates
    assert_last_update_slot(&repay_reserve, clock.slot)?;
    obligation.accrue_interest(repay_reserve.cumulative_borrow_rate_wads)?;

    let RepayResult {
        integer_repay_amount,
        decimal_repay_amount,
        collateral_withdraw_amount,
        obligation_token_amount,
    } = obligation.repay(liquidity_amount, obligation_mint.supply)?;
    repay_reserve
        .liquidity
        .repay(integer_repay_amount, decimal_repay_amount)?;

    Reserve::pack(repay_reserve, &mut repay_reserve_info.data.borrow_mut())?;
    Obligation::pack(obligation, &mut obligation_info.data.borrow_mut())?;

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if lending_market_authority_info.key != &lending_market_authority_pubkey {
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    // burn obligation tokens
    spl_token_burn(TokenBurnParams {
        mint: obligation_token_mint_info.clone(),
        source: obligation_token_input_info.clone(),
        amount: obligation_token_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    // deposit repaid liquidity
    spl_token_transfer(TokenTransferParams {
        source: source_liquidity_info.clone(),
        destination: repay_reserve_liquidity_supply_info.clone(),
        amount: integer_repay_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    // withdraw collateral
    spl_token_transfer(TokenTransferParams {
        source: withdraw_reserve_collateral_supply_info.clone(),
        destination: destination_collateral_info.clone(),
        amount: collateral_withdraw_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
pub fn process_margin_borrow(
    program_id: &Pubkey,
    token_amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if token_amount == 0 {
        return Err(LendingError::InvalidAmount.into());
    }

    let account_info_iter = &mut accounts.iter();
    let source_collateral_info = next_account_info(account_info_iter)?;
    let destination_liquidity_info = next_account_info(account_info_iter)?;
    let deposit_reserve_info = next_account_info(account_info_iter)?;
    let deposit_reserve_collateral_supply_info = next_account_info(account_info_iter)?;
    let deposit_reserve_collateral_fees_receiver_info = next_account_info(account_info_iter)?;
    let borrow_reserve_info = next_account_info(account_info_iter)?;
    let borrow_reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let dex_market_info = next_account_info(account_info_iter)?;
    let dex_market_orders_info = next_account_info(account_info_iter)?;
    let memory = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program_id = next_account_info(account_info_iter)?;

    // Ensure memory is owned by this program so that we don't have to zero it out
    if memory.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }

    let lending_market = LendingMarket::unpack(&lending_market_info.data.borrow())?;
    if lending_market_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &lending_market.token_program_id != token_program_id.key {
        return Err(LendingError::InvalidTokenProgram.into());
    }

    let deposit_reserve = Reserve::unpack(&deposit_reserve_info.data.borrow())?;
    if deposit_reserve_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if &deposit_reserve.lending_market != lending_market_info.key {
        msg!("Invalid reserve lending market account");
        return Err(LendingError::InvalidAccountInput.into());
    }

    let mut borrow_reserve = Reserve::unpack(&borrow_reserve_info.data.borrow())?;
    if borrow_reserve_info.owner != program_id {
        return Err(LendingError::InvalidAccountOwner.into());
    }
    if borrow_reserve.lending_market != deposit_reserve.lending_market {
        return Err(LendingError::LendingMarketMismatch.into());
    }

    if deposit_reserve.config.loan_to_value_ratio == 0 {
        return Err(LendingError::ReserveCollateralDisabled.into());
    }
    if deposit_reserve_info.key == borrow_reserve_info.key {
        return Err(LendingError::DuplicateReserve.into());
    }
    if deposit_reserve.liquidity.mint_pubkey == borrow_reserve.liquidity.mint_pubkey {
        return Err(LendingError::DuplicateReserveMint.into());
    }
    if &borrow_reserve.liquidity.supply_pubkey != borrow_reserve_liquidity_supply_info.key {
        msg!("Invalid borrow reserve liquidity supply account input");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &deposit_reserve.collateral.supply_pubkey != deposit_reserve_collateral_supply_info.key {
        msg!("Invalid deposit reserve collateral supply account input");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &deposit_reserve.collateral.supply_pubkey == source_collateral_info.key {
        msg!("Cannot use deposit reserve collateral supply as source account input");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &deposit_reserve.collateral.fees_receiver
        != deposit_reserve_collateral_fees_receiver_info.key
    {
        msg!("Invalid deposit reserve collateral fees receiver account");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if &borrow_reserve.liquidity.supply_pubkey == destination_liquidity_info.key {
        msg!("Cannot use borrow reserve liquidity supply as destination account input");
        return Err(LendingError::InvalidAccountInput.into());
    }

    // TODO: handle case when neither reserve is the quote currency
    if borrow_reserve.dex_market.is_none() && deposit_reserve.dex_market.is_none() {
        msg!("One reserve must have a dex market");
        return Err(LendingError::InvalidAccountInput.into());
    }
    if let COption::Some(dex_market_pubkey) = borrow_reserve.dex_market {
        if &dex_market_pubkey != dex_market_info.key {
            msg!("Invalid dex market account input");
            return Err(LendingError::InvalidAccountInput.into());
        }
    }
    if let COption::Some(dex_market_pubkey) = deposit_reserve.dex_market {
        if &dex_market_pubkey != dex_market_info.key {
            msg!("Invalid dex market account input");
            return Err(LendingError::InvalidAccountInput.into());
        }
    }

    assert_last_update_slot(&borrow_reserve, clock.slot)?;
    assert_last_update_slot(&deposit_reserve, clock.slot)?;

    let trade_simulator = TradeSimulator::new(
        dex_market_info,
        dex_market_orders_info,
        memory,
        &lending_market.quote_token_mint,
        &borrow_reserve.liquidity.mint_pubkey,
        &deposit_reserve.liquidity.mint_pubkey,
    )?;

    let loan = deposit_reserve.create_loan(
        token_amount,
        token_amount_type,
        trade_simulator,
        &borrow_reserve.liquidity.mint_pubkey,
    )?;

    borrow_reserve.liquidity.borrow(loan.borrow_amount)?;

    Reserve::pack(borrow_reserve, &mut borrow_reserve_info.data.borrow_mut())?;

    let authority_signer_seeds = &[
        lending_market_info.key.as_ref(),
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, program_id)?;
    if lending_market_authority_info.key != &lending_market_authority_pubkey {
        return Err(LendingError::InvalidMarketAuthority.into());
    }

    // deposit collateral
    spl_token_transfer(TokenTransferParams {
        source: source_collateral_info.clone(),
        destination: deposit_reserve_collateral_supply_info.clone(),
        amount: loan.collateral_amount,
        authority: user_transfer_authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    // transfer host fees if host is specified
    let mut owner_fee = loan.origination_fee;
    if let Ok(host_fee_recipient) = next_account_info(account_info_iter) {
        if loan.host_fee > 0 {
            owner_fee -= loan.host_fee;
            spl_token_transfer(TokenTransferParams {
                source: source_collateral_info.clone(),
                destination: host_fee_recipient.clone(),
                amount: loan.host_fee,
                authority: user_transfer_authority_info.clone(),
                authority_signer_seeds: &[],
                token_program: token_program_id.clone(),
            })?;
        }
    }

    // transfer remaining fees to owner
    if owner_fee > 0 {
        spl_token_transfer(TokenTransferParams {
            source: source_collateral_info.clone(),
            destination: deposit_reserve_collateral_fees_receiver_info.clone(),
            amount: owner_fee,
            authority: user_transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program_id.clone(),
        })?;
    }

    // borrow liquidity
    spl_token_transfer(TokenTransferParams {
        source: borrow_reserve_liquidity_supply_info.clone(),
        destination: destination_liquidity_info.clone(),
        amount: loan.borrow_amount,
        authority: lending_market_authority_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}
