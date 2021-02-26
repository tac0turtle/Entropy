const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const assert = require("assert");

describe("margin-account", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.MarginAccount;
  const lendingProgram = new anchor.web3.PublicKey(
    "TokenLending1111111111111111111111111111111"
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
    // Create transaction to create all accounts (need to avoid tx limit)
    let tx = new anchor.web3.Transaction();
    let create_signers = []

    const obligation = new anchor.web3.Account();
    create_signers.push(obligation);
    tx.add(await createSolAccountInstruction(obligation, provider, program, 500));

    const depositReserve = new anchor.web3.Account();
    create_signers.push(depositReserve);
    tx.add(await createSolAccountInstruction(depositReserve, provider, program, 500));

    const borrowReserve = new anchor.web3.Account();
    create_signers.push(borrowReserve);
    tx.add(await createSolAccountInstruction(borrowReserve, provider, program, 500));

    const obligationTokenMint = new anchor.web3.Account();
    create_signers.push(obligationTokenMint);
    tx.add(await createSolAccountInstruction(obligationTokenMint, provider, program, 500));

    // Split the txs into two, because over cap
    await provider.send(tx, create_signers);
    tx = new anchor.web3.Transaction();
    create_signers = []

    const obligationTokenOutput = new anchor.web3.Account();
    create_signers.push(obligationTokenOutput);
    tx.add(await createSolAccountInstruction(obligationTokenOutput, provider, program, 500));

    const obligationTokenOwner = new anchor.web3.Account();
    create_signers.push(obligationTokenOwner);
    tx.add(await createSolAccountInstruction(obligationTokenOwner, provider, program, 500));

    const lendingMarket = new anchor.web3.Account();
    create_signers.push(lendingMarket);
    tx.add(await createSolAccountInstruction(lendingMarket, provider, program, 500));


    // Execute the transaction against the cluster.
    await provider.send(tx, create_signers);

    let [
      _lendingMarketAuthority,
      _nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [lendingMarket.publicKey.toBuffer()],
      lendingProgram.programId
    );
    const lendingMarketAuthority = _lendingMarketAuthority;

    await program.rpc.initObligation({
      accounts: {
        lendingProgram,
        depositReserve: depositReserve.publicKey,
        borrowReserve: borrowReserve.publicKey,
        obligation: obligation.publicKey,
        obligationTokenMint: obligationTokenMint.publicKey,
        obligationTokenOutput: obligationTokenOutput.publicKey,
        obligationTokenOwner: obligationTokenOwner.publicKey,
        lendingMarket: lendingMarket.publicKey,
        lendingMarketAuthority: lendingMarketAuthority.publicKey,

        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
  });
});

async function createSolAccountInstruction(account, provider, program, size) {
  return anchor.web3.SystemProgram.createAccount({
    fromPubkey: provider.wallet.publicKey,
    newAccountPubkey: account.publicKey,
    space: size,
    lamports: await provider.connection.getMinimumBalanceForRentExemption(
      size
    ),
    programId: program.programId,
  });
}
