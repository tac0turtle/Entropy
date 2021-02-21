# Margin Accounts

Margin traders first open a specific margin account that they can deposit an underlying currency to be used as collateral. For the moment we will consider just a single currency (isolated) but it is perfectly feasible to combine a bundle of currencies as collateral. Eventually, when the trader opens a position, buying a different currency on margin, they can choose a collateral ratio with which to leverage the position. A 3x long position on SOL-USDC for example means 1x SOL will be used as collateral to back the loan, and the loan (2x SOL worth of USDC) will be swapped for SOL. Lending pools, which fund the debt, are characterized by a maximum collateral ratio (more is addressed in [governance](./governance.md)) and an interest rate (further addressed in [lending pools](./lending.md)). 

Let's roughly define some of these concepts.

```rust
pub struct MarginAccount {
    pub owner: Pubkey
    pub positions Vec<Position>
}

// This represents a single trade. The bought and sold is just a fractional representation of the price
pub struct Token {
    pub denomination: Denom
    pub value: u64
}

pub struct Loan {
    pub value: u64
    pub interest: u64 // this could be with respect to blocks or time
    pub height: u64 // the height that the loan was made (this is used to calculate the accrual of interest)
}

pub struct Position {
    pub trade: MarketTrade
    pub collateral: u64
    pub debt: Vec<Loans>
    pub base_denmination: Denom
    pub tokens: Vec<Token> // Note that the user could buy several tokens on margin
}
```

As we can see, a position represents a static snapshot of the time of the transaction. Naturally, this changes over the course of time as the value of the tokens in other denominations
changes with respect to the base denomination. Monitoring of the positions of each of the margin accounts in order to avoid a state where the collateral doesn't cover the loss is thus the responsibility of liquidity bots (see [liquidation](./liquidation.md)). 

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

## Messages

The margin contract defines 6 messages, five of which can only be accessed by the trader.

### InitializeAccount

Initialize creates a margin account on behalf of the caller.

```rust
#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(init)]
    margin_account: ProgramAccount<'info, MarginAccount>,
}
```

### InitObligation

Deposits funds into an obligation account to be used to open a leveraged trade.

- Can only be called by the trader.

    ///   0. `[]` Deposit reserve account.
    ///   1. `[]` Borrow reserve account.
    ///   2. `[writable]` Obligation
    ///   3. `[writable]` Obligation token mint
    ///   4. `[writable]` Obligation token output
    ///   5. `[]` Obligation token owner
    ///   6. `[]` Lending market account.
    ///   7. `[]` Derived lending market authority.

    let deposit_reserve_info = next_account_info(account_info_iter)?;
    let borrow_reserve_info = next_account_info(account_info_iter)?;

```rust
#[derive(Accounts)]
pub struct InitObligation<'info> {
    // TODO
}
```

### OpenPosition

Opens a leveraged position through the lending pool. This will use the collateral to allow a non-backed loan of the complementary token of the pair. These tokens will be exchanged through the Serum dex to the token denomination of the leveraged position.

- Can only be called by the trader

```rust
#[derive(Accounts)]
pub struct OpenPosition<'info> {
    // TODO
}
```

### ClosePosition

Closes a position which was opened in `OpenPosition`. This will exchange tokens to cover the original loan and withdraw the remaining tokens into the trader's account passed in.

- Can only be called by the trader

```rust
#[derive(Accounts)]
pub struct ClosePosition<'info> {
    // TODO
}
```

### Withdraw

Withdraw withdraws the funds from a liquidated position into an account owned by the trader. This is separate from the liquidate function because that can be called from any user, and the trader will need a function to be able to withdraw remaining funds from a liquidation event.

TODO: Determine if this can be combined with close, as it should be functionally similar

- Withdraw can only be called by the trader. 

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

Liquidate is only performed when an account has hit their liquidation limit. This message tries to close the current position as fast as possible. It may need to place multiple trades based on liquidity of markets

- Liquidate can be called from any actor for a reward.

```rust
#[derive(Accounts)]
pub struct Liquidate<'info> {

}
```

## Cross program interactions

The margin account contract makes a few cross program calls. To start the margin account has the right to take non-backed loans, and make trades on the Serum Dex. 

1. When a leveraged trade is initiated, the margin account will make a request to the lending contract requesting funds to allow a leveraged trade. 
   - When the user makes the trade the liquidation price, loan interest rate and fees are known already. The interest rate is fixed rate that is charged entirely, even if the trade is opened for a minute. 

2. When the margin account has taken the loan or needs to repay, it will exchange tokens through the serum dex to swap the token from one denomination of the pair to the other.

3. When a position is closed, the funds need to be put back in the lending pool and do any necessary logic to close the obligation account.
