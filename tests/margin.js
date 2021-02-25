const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
// const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const assert = require("assert");

describe("margin-account", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.MarginAccount;
  const lendingProgram = new anchor.web3.PublicKey(
    "FtMNMKp9DZHKWUyVAsj3Q5QV8ow4P3fUPP7ZrWEQJzKr"
  );

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

    // Assert that the embedded program is executable
    let accInfo = await anchor.getProvider().connection.getAccountInfo(lendingProgram);
    assert.ok(accInfo.executable);
  });

  const marginAcc = new anchor.web3.Account();
  // const vault = new anchor.web3.Account();
  let marginProgram = null

  it("Initializes margin account", async () => {
    // Arbitrary size for now, just need it to be large enough
    const marginSize = 600;
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
    marginProgram = await program.account.marginAccount(marginAcc.publicKey);
    assert.ok(marginProgram.trader.equals(provider.wallet.publicKey));
  });

  it("Initializes obligation account", async () => {
    // let instructions = []
    // let signers = []

    // // Arbitrary size because I don't have access to the actual layout yet
    // const obligation_size = 500;
    // let obligation = new anchor.web3.Account();
    // signers.push(obligation);
    // instructions.push(anchor.web3.SystemProgram.createAccount({
    //   fromPubkey: provider.wallet.publicKey,
    //   newAccountPubkey: obligation.publicKey,
    //   space: obligation_size,
    //   lamports: await provider.connection.getMinimumBalanceForRentExemption(
    //     obligation_size
    //   ),
    //   programId: program.programId,
    // }));

    // let deposit_reserve = provider.wallet.publicKey;
    // let borrow_reserve = provider.wallet.publicKey;
    // let obligation = provider.wallet.publicKey;
    // let obligation_token_mint = provider.wallet.publicKey;
    // let obligation_token_output = provider.wallet.publicKey;
    // let obligation_token_owner = provider.wallet.publicKey;
    // let lending_market = provider.wallet.publicKey;
    // let lending_market_authority = provider.wallet.publicKey;

    // await registry.rpc.initObligation({
    //   accounts: {
    //     lending_program: lendingProgram,
    //     deposit_reserve,
    //     borrow_reserve,
    //     obligation,
    //     obligation_token_mint,
    //     obligation_token_output,
    //     obligation_token_owner,
    //     lending_market,
    //     lending_market_authority,
    //   },
    //   signers,
    //   instructions,
    // });
  });
});
