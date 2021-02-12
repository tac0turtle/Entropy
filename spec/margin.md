# Margin Accounts

Margin traders first open a specific margin account that they can deposit an underlying currency to be used as collateral. For the moment we will consider just a single currency but it is perfectly feasible to combine a bundle of currencies as collateral. Eventually, when the trader opens a position, buying a different currency on margin, they can choose a collateral ratio with which to fund the trade. A 50% collateral ratio (or 2x) is when half of the trade is financed from the traders equity and the other half from debt. Lending pools, which fund the debt, are characterized by a maximum collateral ratio (more is addressed in [governance](./governance.md)) and an interest rate (further addressed in [lending pools](./lending.md)). 

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

A trader can have a set of simultaneous positions at once; each position corresponding to a different underlying denomination. At any point, the trader is able to perform a series of possible
actions (each action corresponding to a transaction): 

1. Close the position. This sells all the tokens at the market rate, repays the loan with interest and returns the remainder into a personal account.
2. Increase the collateral. This can be after the equivalent of a "margin call" is issued and the owner must increase his/her collateral to avoid entering the red zone (marked by the liquidation ratio (see [governance](./governance.md))
3. Buy more tokens. Expand the position by buying more tokens on margin.

