# Margin Accounts

Margin traders first open a specific margin account that they can deposit an underlying currency to be used as collateral. For the moment we will consider just a single currency (isolated) but it is perfectly feasible to combine a bundle of currencies as collateral. Eventually, when the trader opens a position, buying more of the same currency on margin, they can choose a collateral ratio with which to leverage their position. A 3x long position on SOL-USDC for example means 1x SOL will be used as collateral to back the loan, and the loan (2x SOL worth of USDC) will be swapped for SOL. Lending pools, which fund the debt, are characterized by a maximum collateral ratio (more is addressed in [governance](./governance.md)) and an interest rate (further addressed in [lending pools](./lending.md)).

![margin lifecycle](./assets/lifecycle.png)

Here is an example of the typical use case of a margin account, where each box illustrates and action which is described in detail further down this page. The dotted lines symbolise that a trader can potentially hold and manage multiple positions simultaneously and from within the same margin account. Closing of any of these positions can be done voluntarily by the trader or can be involuntarily called by anyone when certain liquidation requirements are met (see [liquidation](./liquidation.md)). 

A trader can have a set of simultaneous positions at once; each position corresponding to a different underlying denomination. The trader has the option to close any of the open positions at any time. To do this, tokens are exchanged to cover the original loan plus interest and puts this back in the lending pool to repay. The remaining funds are then withdrawn into a trader's account.

## State

The contract has state. 

- Pairs: Pairs represents which trading pairs a margin account is allowed to trade. Serum is a large exchange with countless trading pairs. Many of these pairs do not have much liquidity. For this reason we need to limit which pairs are traded.

```rust 
#[state]
pub struct Margin {
    // pairs represents an array of pairs available to trade
    pub pairs: vec<(PubKey, Pubkey)>,
}
```

Margin has a single account struct as state. 

```rust
#[account]
pub struct MarginAccount {
    /// The owner of this margin account.
    pub trader: Pubkey,
    /// Open positions held by the margin account.
    pub positions: Vec<Position>,

    /// nonce for program derived address
    pub nonce: u8,
}
```

Which contains positions opened by the trader.

```rust
/// Tracks position opened my margin account.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Position {
    /// Tracks the size of the loan to know if the amount being paid back is the total amount in order to unlock the account
    pub loan_amount: u64,
    /// This account holds tokens from the loan before they are used in the trade and conversely to hold
    /// tokens after closing the position and before repaying the loan.
    pub loaned_vault: Pubkey,
    /// Tokens are stored here when a position is opened (status becomes locked). When the loan is repaid,
    /// status is updated to available and the trader is able to withdraw the tokens.
    pub collateral_vault: Option<Pubkey>,
    /// When a position is open, status is locked meaning funds can't be withdrawn. Once a position is closed out,
    /// status is updated to available indicating that the trader can now withdraw the tokens.
    pub status: Status,
}

pub enum Status {
    Locked = 0,
    Available =  1,
}
```

## Actions

The margin contract is comprised of 6 possible actions, each of which can be broken down into a set of transactions. Initially we will look at this from the perspective of using an AMM with which to perform trades.
See [using an orderbook](##-Using-An-Orderbook) for how the same process could be applied with an order book.

### Initialize Account

Initialize Account has a single transaction, `InitializeAccount` which creates a margin account on behalf of the caller.

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    rent: Sysvar<'info, Rent>,
}
```

### Init Obligation

Deposits funds into an obligation account to be used to open a leveraged trade.

- Can only be called by the trader.
- Deposits of the same denomination will continue to create new obligation accounts as it's not possible to add more funds into an existing account
- Clients will need to liquidate the obligation account and create a new one if they wish to add or remove to the amount of collateral. 
- Clients must keep track of obligation accounts

```rust
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

    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

```

### Borrow

Opens a leveraged position through the lending pool. This will use the collateral to allow a non-backed loan of the complementary token of the pair. These tokens will be exchanged through the Serum DEX or AMM to the token denomination of the leveraged position. The action consists of a borrow transaction followed by either an order through an Orderbook or an AMM.

1) Borrow - borrow funds from a lending reserve and move them to the margin account. This calls [`MarginBorrowReserveLiquidity`](./lending.md). The margin account will in turn create a new position and the funds shall be represented as `loan_denominated_tokens`.

- Can only be called by the trader. 
- > TODO: Add collateral constraints
- `margin_account.position.status = status.Locked`

```rust
#[derive(Accounts)]
pub struct Borrow<'info> {
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
```

2. Open Position/trade via AMM - This takes the funds from the margin account (specifically `loan_denominated_tokens`) and performs an AMM trade. Traded tokens are placed directly back into the same address i.e. they remain locked.

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct TradeAmm<'info> {
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
```

### Repay

Closes a position which was opened in `Borrow`. This will exchange tokens to cover the original loan plus accrued interest and move the remaining tokens into the margin account. This is perhaps the most complicated action.

1. Close Position via AMM - Calculates the total repayment sum in the loan denomination before trading this with the respective amount of the collateral denomination stored in `locked_tokens`. The funds are sent to the margin account.

- Can only be called by the trader.
- Repayment sum = loan + interest + buffer. We need to add a buffer here because we don't know when the `RepayLoan` call will be made

Call `TradeAmm` (listed above) to swap back into the loan denomination.

2. Repay Loan Obligation - calls [`RepayReserveLiquidity`](./lending.md) to send repayment funds from the margin account back into the lending reserve. If this is less than the total amount, then the repayment will trigger the liquidation of the obligation account.

- Can only be called by the trader.
- Upon success, tokens become available `margin_account.position.status = status.Available`

```rust
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
```

4. Withdraw - takes the accumulated funds from `collateral_denominated_tokens` and transfers it to a private account of the users choosing.

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    user_token_account: AccountInfo<'info>,
    #[account(mut)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    #[account(seeds = [margin_account.to_account_info().key.as_ref(), &[margin_account.nonce]])]
    vault_signer: AccountInfo<'info>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}
```

### Liquidate

Liquidate is only performed when an account has hit their liquidation limit. This replicates much of the functionality as closing a position would but rather than only being executed by the trader, these calls can be executed by anyone.

1. Liquidate Position - requires swap with `TradeAmm`

- Can be called by anyone.
- Check's [liquidation limit](./liquidation.md) has been reached
- Successful liquidation transfers a small reward to a specified beneficiary. This is used to incentivise bots to liquidate unhealthy positions in a timely manner

```rust
#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    pub beneficiary: AccountInfo<'info>
}
```

2. Force Repay Loan - wraps around `RepayLoan`.

- Can be called by anyone.
- Successful liquidation transfers a small reward to a specified beneficiary. This is used to incentivise bots to liquidate unhealthy positions in a timely manner.


```rust
#[derive(Accounts)]
pub struct ForceRepayLoan<'info> {

}
```

The other steps, `SettleFunds` and `Withdraw` aren't critical to ensuring that loans are successfully repaid and thus is still left to the responsibility of the trader to execute when they want. 

## Cross program interactions

The margin account contract makes a few cross program calls. To start the margin account has the right to take non-backed loans, and make trades on the Serum Dex. 

1. When a leveraged trade is initiated, the margin account will make a request to the lending contract requesting funds to allow a leveraged trade. 
   - When the user makes the trade the liquidation price, loan interest rate and fees are known already. The interest rate is fixed rate that is charged entirely, even if the trade is opened for a minute. 

2. When the margin account has taken the loan or needs to repay, it will exchange tokens through the serum dex to swap the token from one denomination of the pair to the other.

3. When a position is closed, the funds need to be put back in the lending pool and do any necessary logic to close the obligation account.


## Using an Orderbook

This is outside the scope of the current implementation and has therefore been left more as an appendix. It is however possible to wrap the same functionality of marign trading but using an orderbook to perform trades instead.

> NOTE: This is an incomplete list of transactions. 

Open Position via Orderbook - This takes the funds from the margin account and opens an order by calling [`NewOrder`](https://github.com/project-serum/serum-dex/blob/1a9aee6e745e77155b7974e1df06c1ebc97bfae0/dex/src/instruction.rs#L194). Funds are kept in the settle account, and the margin account keeps a reference of this account for closing the position.

```rust
#[derive(Accounts)]
pub struct OpenPositionOrderbook<'info> {
    // TODO

}
```

Close Position via Orderbook - Calculates the total repayment sum in the loan denomination and places a sell order of type `Immediate or Cancel` of the same value. If the total repayment sum is greater than the value then the entire amount in the settle account will be used.

```rust
#[derive(Accounts)]
pub struct ClosePositionOrderBook<'info> {
    // TODO
}
```

Settle Repayment Fund - If the sell order is successful, the client then needs to move the funds from the settle account to the margin account.

```rust
#[derive(Accounts)]
pub struct SettleRepaymentFunds<'info> {
    // TODO
}
```
