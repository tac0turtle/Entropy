# Lending

Lending is essential to a margin based protocol. In order to trade margin a user must be able to get the added funds required to over leverage themselves.
There will be three users of the lending protocol.

First, a user who wants to lend out their money. This user will come to the entropy protocol, add funds to a lending pool. The reason a user will want to do this is because there are users who will want to borrow money in order to margin trade.

Second, a user who wants to borrow money. This user will create an account to send money to, this money will represent the collateral for the loan. The c-ratio will be 150%. This means for a loan of $100 the user must put up $150. A user will want to loan money for countless reasons, one being if they believe the underlying collateral token will raise in price.

Third, a user who wants to conduct a leveraged trade. This user has funds but would like to get margin in order to increase their trade size. This sort of trade will be conducted through a [margin account](./margin.md). A margin account does need to over collateralize. THe reason they do not need to over collateralize is because when a loan is opened the user can not withdraw funds from the account, only add in order to increase the liquidation to current price difference.

The lending contract that will be used is https://github.com/solana-labs/solana-program-library/tree/master/token-lending. We will be adding an additional state transition. A non-backed loan, this loan will be given to only a partial controlled account. Read more on this account [here](./margin.md)

## Messages

There will be an extra message added to the lending contract. A non-backed loan. 

```rust
    /// Borrow tokens from a reserve by depositing collateral tokens. The number of borrowed tokens
    /// is calculated by market price. The debt obligation is tokenized.
    ///
    ///   0. `[writable]` Source collateral token account, minted by deposit reserve collateral mint,
    ///                     $authority can transfer $collateral_amount
    ///   1. `[writable]` Destination liquidity token account, minted by borrow reserve liquidity mint
    ///   2. `[]` Deposit reserve account.
    ///   3. `[writable]` Deposit reserve collateral supply SPL Token account
    ///   4. `[writable]` Deposit reserve collateral fee receiver account.
    ///                     Must be the fee account specified at InitReserve.
    ///   5. `[writable]` Borrow reserve account.
    ///   6. `[writable]` Borrow reserve liquidity supply SPL Token account
    ///   10 `[]` Lending market account.
    ///   11 `[]` Derived lending market authority.
    ///   12 `[]` User transfer authority ($authority).
    ///   13 `[]` Dex market
    ///   14 `[]` Dex market order book side
    ///   15 `[]` Temporary memory
    ///   16 `[]` Clock sysvar
    ///   17 '[]` Token program id
    ///   18 `[optional, writable]` Deposit reserve collateral host fee receiver account.
    BorrowReserveLiquidity {
        // TODO: slippage constraint
        /// Amount whose usage depends on `amount_type`
        amount: u64,
        /// Describe how the amount should be treated
        amount_type: BorrowAmountType,
    },
  ```
