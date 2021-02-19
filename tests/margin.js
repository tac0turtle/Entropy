const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
// const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const assert = require("assert");

describe("margin-account", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.MarginAccount;

  let mint = null;
  let baseVault = null;
  let receiver = null;

  it("Sets up initial test state", async () => {
    // Setup vault accounts for interactions
    const [_mint, _baseVault] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(1000000)
    );
    mint = _mint;
    baseVault = _baseVault;

    receiver = await serumCmn.createTokenAccount(
      program.provider,
      mint,
      program.provider.wallet.publicKey
    );
  });

  const marginAcc = new anchor.web3.Account();
  // const vault = new anchor.web3.Account();

  it("Initializes margin account", async () => {
    // Arbitrary size for now, just need it to be large enough
    const marginSize = 1200;
    await program.rpc.initialize(provider.wallet.publicKey, {
      accounts: {
        marginAccount: marginAcc.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [marginAcc],
      instructions: [
        await program.account.marginAccount.createInstruction(marginAcc, marginSize),
      ],
    });


    // Assert state after initialization
    const marginAccount = await program.account.marginAccount(marginAcc.publicKey);
    assert.ok(marginAccount.trader.equals(provider.wallet.publicKey));
  });
});
