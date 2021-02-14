# Lending

Lending is essential to a margin based protocol. In order to trade margin a user must be able to get the added funds required to over leverage themselves.
There will be three users of the lending protocol.

First, a user who wants to lend out their money. This user will come to the entropy protocol, add funds to a lending pool. The reason a user will want to do this is because there are users who will want to borrow money in order to margin trade.

Second, a user who wants to borrow money. This user will create an account to send money to, this money will represent the collateral for the loan. The c-ratio will be 150%. This means for a loan of $100 the user must put up $150. A user will want to loan money for countless reasons, one being if they believe the underlying collateral token will raise in price.

Third, a user who wants to conduct a leveraged trade. This user has funds but would like to get margin in order to increase their trade size. This sort of trade will be conducted through a [margin account](./margin.md). A margin account does need to over collateralize. THe reason they do not need to over collateralize is because when a loan is opened the user can not withdraw funds from the account, only add in order to increase the liquidation to current price difference.

## Design
