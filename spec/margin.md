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
}
```

```rust
// Position holds a store of two token denominations (or mints). One for the deonomination that the loan is in
// and the other for the denomination that the trade or collateral is in.
pub struct Position {
    // This account holds tokens from the loan before they are used in the trade and conversely to hold
    // tokens after closing the position and before repaying the loan. 
    pub loan_denominated_tokens: PubKey,

    // Tokens are stored here when a position is opened (status becomes locked). When the loan is repayed,
    // status is updated to available and the trader is able to withdraw the tokens.
    pub collateral_denominated_tokens: PubKey,

    // When a position is open, status is locked meaning funds can't be withdrawn. Once a position is closed out,
    // status is updated to available indicating that the trader can now withdraw the tokens.
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

### Create Account

Create Account has a single transaction, `InitializeAccount` which creates a margin account on behalf of the caller.

```rust
#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(init)]
    margin_account: ProgramAccount<'info, MarginAccount>,
    rent: Sysvar<'info, Rent>,
}
```

### Deposit

Deposits funds into an obligation account to be used to open a leveraged trade. This calls [`InitObligation`](./lending.md).

- Can only be called by the trader.
- Deposits of the same denomination will continue to create new obligation accounts as it's not possible to add more funds into an existing account
- Clients will need to liquidate the obligation account and create a new one if they wish to add or remove to the amount of collateral. 
- Clients must keep track of obligation accounts

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    // TODO
    #[account(signer)]
    authority: AccountInfo<'info>,
    vault: TokenAccount<'info>,
}
```

### OpenPosition

Opens a leveraged position through the lending pool. This will use the collateral to allow a non-backed loan of the complementary token of the pair. These tokens will be exchanged through the Serum DEX or AMM to the token denomination of the leveraged position. The action consists of a borrow transaction followed by either an order through an Orderbook or an AMM.

1) Borrow - borrow funds from a lending reserve and move them to the margin account. This calls [`MarginBorrowReserveLiquidity`](./lending.md). The margin account will in turn create a new position and the funds shall be represented as `loan_denominated_tokens`.

- Can only be called by the trader. 
- > TODO: Add collateral constraints
- `margin_account.position.status = status.Locked`

```rust
#[derive(Accounts)]
pub struct Borrow<'info> {
    // TODO
}
```

2. Open Position via AMM - This takes the funds from the margin account (specifically `loan_denominated_tokens`) and performs an AMM trade. Traded tokens are placed directly back into the same address i.e. they remain locked.

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct OpenPositionAMM<'info> {
    // TODO
}
```

### ClosePosition

Closes a position which was opened in `OpenPosition`. This will exchange tokens to cover the original loan plus accrued interest and move the remaining tokens into the margin account. This is perhaps the most complicated action.

1. Close Position via AMM - Calculates the total repayment sum in the loan denomination before trading this with the respective amount of the collateral denomination stored in `locked_tokens`. The funds are sent to the margin account.

- Can only be called by the trader.
- Repayment sum = loan + interest + buffer. We need to add a buffer here because we don't know when the `RepayLoan` call will be made

```rust
#[derive(Accounts)]
pub struct ClosePositionAMM<'info> {
    // TODO
}
```

2. Repay Loan Obligation - calls [`RepayReserveLiquidity`](./lending.md) to send repayment funds from the margin account back into the lending reserve. If this is less than the total amount, then the repayment will trigger the liquidation of the obligation account.

- Can only be called by the trader.
- Upon success, tokens become available `margin_account.position.status = status.Available`

```rust
#[derive(Accounts)]
pub struct RepayLoan<'info> {
    // TODO
}
```

3. Settle Funds - Once the loan has been payed, any remaining tokens within the obligation account can be transfered back to the margin account (specifically `collateral_denominated_tokens`).

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct SettleFunds<'info> {
    // TODO
    pub obligation_account: PubKey,
}
```

4. Withdraw - takes the accumulated funds from `collateral_denominated_tokens` and transfers it to a private account of the users choosing.

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Depositor
    depositor: AccountInfo<'info>,
    // Authority (trader)
    #[account(signer)]
    depositor_authority: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}
```

### Liquidate

Liquidate is only performed when an account has hit their liquidation limit. This replicates much of the functionality as closing a position would but rather than only being executed by the trader, these calls can be executed by anyone.

1. Liquidate Position - wraps around `ClosePositionAMM`

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
