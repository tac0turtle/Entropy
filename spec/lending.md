# Lending

Lending is essential to a margin based protocol. In order to trade margin a user must be able to get the added funds required to over leverage themselves.

There are three users of the lending protocol.

- **Lender:** is a user who lends out their money in order to accumulate wealth from the interest paid by borrowers.

- **Backed Borrower:** is a user who borrow's money that is backed by collateral that is locked up in an obligation account for the duration of the loan. The c-ratio will be 150%. This means for a loan of $100 the user must put up $150. A user will want to loan money for countless reasons, one being if they believe the underlying collateral token will raise in price.

- **Margin Borrower:** is a user who wants to conduct a leveraged trade. This user has funds but would like to get margin in order to increase their trade size. This sort of trade will be conducted through a [margin account](./margin.md). A margin account does need to over collateralize because it is governed by the margin protocol.

## The Mechanics of Lending

The lending pool is made of a collection of **Reserves**, where each reserve holds liquidity of an underlying token i.e. `USDC`, `USDT` etc.. The protocol uses a tokenization strategy to achieve distribution of accrued interest. This means that each reserve is paired with a token that is given back to the lender i.e `aUSDC` and a **Lending Exchange Rate** between that token and the underlying one (`aUSDC/USDC`). The exchange rate is recalculated after every loan repayment in which it increases and in the averse case, after every loan defaults - in which it decreases. The difference in the exchange rate over the period of a year represents the annualized interest rate of lenders.

Each reserve has a **Utilization Rate** associated with it. The utilization rate is the percentage of the reserve that is being currently borrowed. It changes whenever tokens are loaned, withdrawn, borrowed or repayed. Reserves also have an **Optimal Utilization Rate** which is chiefly a function of the total size of the reserve although in latter iterations it can be adjusted to represent the volatility of the reserve itself (i.e. the rate that funds flow in and out of the reserve). 

Each reserve's **Interest Rate**, the percentage that borrowers must pay on their loans, is the primary tool used to curve the current utilization rate towards the optimal one. Increasing the interest rate is expected to decrease the utilization rate and vice versa. 

The interest rate is fixed per borrower at the time of the loan. Therefore we are likely to observe asymmetrical sensitivity to interest changes. This means that decreasing the interest is likely to result in a relatively quick increase in the utilization rate. However the contrary, increasing the interest rate, doesn't affect current borrowers (only potential new ones) and thus we would expect a much slower decrease in utilization rate. This is something that the controlling algorithm must take into consideration.

Lastly, points of crisis are when lenders try to withdraw more money than is currently available or when borrowers try to borrow more money than is available. The interest rates are used to avoid crisis by trying to achieve a safe utilization rate, however, in the event this happens the transaction will simply be denied. This means that lenders take on the risk of a) the borrowers defaulting and b) potential illiquidity of their tokens.  

## Advanced Features

The basics of the protocol can be furthered by a set of possible advanced features:

- **Lending epochs:** To avoid high volatility in the reserve pool, tokens that are lent could only be withdrawn at prescribed intervals or epochs. This would break the withdraw call into two calls: First, an intention to withdraw which would then set a time in the future with which the tokens become unlocked; Second, the actual withdrawal of tokens so long as the call is made after the tokens are unlocked. 

- **Deposit Rate Limiting:** Another feature to combat high volatility is to set a ratio with which funds can be deposited or withdrawn. As an example, every epoch only 10% of the total liquidity in the reserve can be exchanged. This could also be applied to borrowing.

- **Variable Borrowing Rate:** Currently the protocol charges a fixed interest rate to borrowers because less overhead is required and it is expected that most
borrowers prefer a fixed interest rate. If a reserve were to keep track of the interest rate changes over time then it could be easier to calculate accrued interest over the loan period and thus use variable rates. Variable rates offer better sensitivity to changes in utilization rate. We could further extend this to allow for both rate types and the ability for the borrower to swap in between. 

## Implementation

The lending contract that will be used is https://github.com/solana-labs/solana-program-library/tree/master/token-lending. We will be adding an additional state transition. A non-backed loan, this loan will be given to only a partial controlled account. Read more on this account [here](./margin.md)

Interest rate will be calculated from a simple linear model:

```
interest_rate = base_rate + (utilization_constant * utilization_rate)
```

where: 

```
utilization_rate = borrowed / total_reserve
```

and based from solana's inflation rate of 7 - 9%:

```
base_rate = 6% and utilization_constant = 4%
```

This means that the borrowing rate varies from 6 - 10%

Exchange rates will be updated upon every repayment as follows:

```
exchange_rate = previous_exchange_rate * (accrued_interest + total_reserve / total_reserve)
```

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
