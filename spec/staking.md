# Staking

Staking provides a way for users to earn and govern the protocol by holding onto the native token of the platform. Stakers will receive newly minted tokens through an inflation rate and fees from the traders and burrowers. This provides an incentive to help increase the amount of fees going through the protocol.

Users will come to a frontend that has staking enabled. They will deposit the native token in exchange for a token representing their stake. While the funds are within the staking component of the system users will receive the inflation rate and fee of users of the protocol.

## Inflation

The inflation rate of the protocol represents the rate at which new native tokens are minted. The inflation rate is a rate on the total circulating supply of the protocol. For example if there are 10 million tokens and only 2 million staked, this would mean that the 2 million staked tokens are receiving an inflation rate of `(10 million total tokens / 2 million staked tokens) * inflation rate`.

## Staking

Staking will be provided by the registry contract. (https://github.com/project-serum/anchor/tree/master/examples/lockup/programs/registry).
