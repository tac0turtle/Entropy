# Margin Accounts

Margin traders first open a specific margin account that they can deposit an underlying currency to be used as collateral. For the moment we will consider just a single currency but it is perfectly feasible to combine a bundle of currencies as collateral. Eventually, when the trader opens a position, buying a different currency on margin, they can choose a collateral ratio with which to fund the trade. A 50% collateral ratio (or 2x) is when half of the trade is financed from the traders equity and the other half from debt. Lending pools, which fund the debt, are characterized by a maximum collateral ratio (more is addressed in [governance](./governance.md)) and an interest rate (further addressed in [lending pools](./lending.md)). 

## State

The contract has state. 

- Pairs: Pairs represents which trading pairs a margin account is allowed to trade. Serum is a large exchange with countless trading pairs. Many of these pairs do not have much liquidity. For this reason we need to limit which pairs are traded.

```rust 
#[state]
pub struct Margin {
    // pairs represents an array of pairs available to trade
    pub pairs: vec<PubKey>,
}
```

Margin has a single account struct as state. 

```rust
#[account]
pub struct MarginAcc {
    /// The owner of this margin account.
    pub trader: Pubkey,
    /// Address of the account's token vault.
    pub vault: Pubkey,
    /// loan_amount represents the total size of the loan. 
    pub loan_amount: u64,
    /// Signer nonce.
    pub nonce: u8,
    /// Check if there is an open trade
    pub open_trade: bool,
}
```

## Messages

The margin contract defines 5 messages, four of which can only be accessed by the trader. 

### CreateAccount

CreateAccount creates a margin account on behalf of the caller.

```rust
#[derive(Accounts)]
pub struct CreateAccount<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    // Authority (trader)
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    // Misc.
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
}
```

### Deposit

Deposit deposits funds into a program account to be used for trading. Once a trade is initiated, deposits can still be made. 

- Can only be called by the trader.

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
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

### Withdraw

Withdraw withdraws the funds from the account. Withdraw can only be called if there are no open trades. 

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

### Trade

Trade does multiple things in a single step. When a user would like conduct a trade they will specify the amount of margin and total trade they would like to conduct. The program will check that there are enough funds in the margin account and check if there are enough funds in the lending pool. If there are enough funds the loan will be taken and the trade executed. At this point the margin account has been marked as having a open trade. At this point no withdraws can be made. When a user sells their position or part of the position the funds are repaid to the lending protocol, after this, what is left over is considered the amount the user has to withdraw. 

- Trade can only be called by the trader

```rust
#[derive(Accounts)]
pub struct Trade<'info> {
    pub side Side
    pub order_type OrderType
    pub limit_price NonZeroU64
    pub collateral_ratio Ratio
}
```

### Liquidate

Liquidate is only performed when an account has hit their liquidation limit. This message tries to close the current position as fast as possible. It may need to place multiple trades based on liquidity of markets

- Liquidate can only be called by a coordinator. 

```rust
#[derive(Accounts)]
pub struct Liquidate<'info> {

}
```

### Deleverage

Deleveraging is the act of lowering the amount of leverage. When a traders account hits threshold, defined by governance, a coordinator has the right to deleverage the account by another predefined percentage. 

- Deleverage can only be called by the coordinator.

```rust
#[derive(Accounts)]
pub struct Deleverage<'info> {

}
```

## Cross program interactions

The margin account contract makes a few cross program calls. To start the margin account has the right to take non-backed loans, and make trades on the Serum Dex. 

1. The first cross chain program interaction is in the creation of the margin account. When a margin account is created it needs to be registered within the lending protocol to enable non-backed loans. The

2. Secondly, when a leveraged trade is initiated, the margin account will make a request to the lending contract requesting funds to allow a leveraged trade. 
   1. When the user makes the trade the liquidation price, loan interest rate and fees are known already. The interest rate is fixed rate that is charged entirely, even if the trade is opened for a minute. 

3. Thirdly, when the margin account has taken the loan it will execute the trade against the serum dex.
