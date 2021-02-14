# Governance

Governance controls the protocol. A user must stake tokens in order to have the right to vote on proposals. Find more about staking [here](./staking.md)

Governance controls the risk parameters of the protocol and which pairs a user can trade. The parameters of the protocol include collateral ratio for a loan, available lending pools, leverage and liquidation amounts.

## Parameters

The parameters of the protocol are:

- C-ratio (Collateral Ratio)
  - The collateral ratio of loans controls how much a user must put in for a loan. At the start it will be set to 175%, as the protocol matures it will be lowered.

- Lending pools
  - Lending pools represents which pools are available to lend and borrow from. Lending pools are created via governance.

- Leverage
  - Leverage is the amount of margin a trader can take. The higher leverage the protocol allows the more at risk the protocol itself becomes. If a trader takes a large amount of leverage in a illiquid market the protocol will have a harder time liquidating the position if the position does not work in favor of the trader

- Pairs
  - Pairs represents which markets a user is allowed to trade. Because there are markets within the serum dex that are illiquid the protocol should not allow traders to trade in these markets. This limits the amount of trades the protocol may entice but it is meant as a safety mechanism.

- Fees
  - Fees will send funds to the stakers, insurance fund, community fund and lenders. Stakers will get a portion of the fees because they are staking, the amount wont be as large as the other pools because stakers are also getting an issuance rate. The insurance and community funds, will get a portion of the fees, the amount each gets can be changed via a governance proposal. The community fund will receive a smaller amount of the fees to start until the insurance fund has reached a certain threshold. The threshold can change, and should change based on the volume of the protocol. At the start of the protocol or when the trading amount is not great the insurance fund amount can stay at the current threshold, if there is an increase of traders and leverage, the protocol can vote to increase the threshold to help de-risk the lenders and stakers. Lenders will get a fee for lending their funds to traders, this fee will vary based on the utilization of the lending pool. Once a lender has taken the loan the borrow rate will stay the same.
  
- Liquidation Risk
  - On centralized exchanges, not only in crypto, a margin account can go into negative. Unfortunately, this can not be allowed in a decentralized protocol this can not be allowed. Once a users margin account hits zero it is up to the insurance fund to bring the users account back to zero. For this reason there must be partial checkins before liquidation risk hits a point where the user's account hits zero. To start there will be three levels to watch out for. Read more on the levels of liquidation [here](./liquidation.md)

- Inflation
  - Inflation represents the amount of tokens to be minted for staking the native token. This number can vary from 0%-100%.
