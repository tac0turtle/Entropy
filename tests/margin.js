const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const assert = require("assert");
const BufferLayout = require("buffer-layout");

describe("margin-account", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const lendingProgram = new anchor.web3.PublicKey(
    "TokenLending2222222222222222222222222222222"
  );

  const program = anchor.workspace.MarginAccount;

  let liquidityMint = null;
  let liquidityVault = null;
  let liquidityReserveVault = null;
  let userLiquidity = null;
  let liquidityFeesReceiver = null;
  let collateralMint = null;
  let collateralVault = null;
  let collateralReserveVault = null;
  let userCollateral = null;
  let collateralFeesReceiver = null;

  it("Sets up initial test state", async () => {
    // TODO all this could be done in one tx, idc this is quicker
    // Setup vault accounts for interactions
    const [_liqMint, _liqVault] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(2000000)
    );
    liquidityMint = _liqMint;
    liquidityVault = _liqVault;

    const [_colMint, _colVault] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(1000000)
    );
    collateralMint = _colMint;
    collateralVault = _colVault;

    liquidityReserveVault = await serumCmn.createTokenAccount(
      program.provider,
      liquidityMint,
      program.provider.wallet.publicKey
    );

    userLiquidity = await serumCmn.createTokenAccount(
      program.provider,
      liquidityMint,
      program.provider.wallet.publicKey
    );

    liquidityFeesReceiver = await serumCmn.createTokenAccount(
      program.provider,
      liquidityMint,
      program.provider.wallet.publicKey
    );

    collateralReserveVault = await serumCmn.createTokenAccount(
      program.provider,
      collateralMint,
      program.provider.wallet.publicKey
    );

    userCollateral = await serumCmn.createTokenAccount(
      program.provider,
      collateralMint,
      program.provider.wallet.publicKey
    );

    collateralFeesReceiver = await serumCmn.createTokenAccount(
      program.provider,
      collateralMint,
      program.provider.wallet.publicKey
    );

    // Assert that the embedded program is executable
    let accInfo = await anchor.getProvider().connection.getAccountInfo(lendingProgram);
    assert.ok(accInfo.executable);
  });

  const marginAcc = new anchor.web3.Account();
  // const vault = new anchor.web3.Account();
  let marginProgram = null

  it("Initializes margin program state", async () => {
    let accounts = {
      authority: provider.wallet.publicKey,
    };

    await program.state.rpc.new({ accounts });
    await program.state.rpc.clearPairs({ accounts });

    let state = await program.state();
    assert.ok(state.authority.equals(provider.wallet.publicKey));
    assert.ok(state.tokenPairs.length == 0);

    // Add token pair used for test
    await program.state.rpc.addTokenPair(collateralMint, liquidityMint, { accounts });
    state = await program.state();
    assert.ok(state.tokenPairs.length === 1);
    assert.ok(state.tokenPairs[0].firstToken.equals(collateralMint));
    assert.ok(state.tokenPairs[0].secondToken.equals(liquidityMint));
  });

  it("Initializes margin account", async () => {
    // Arbitrary size for now, just need it to be large enough
    const marginSize = 600;
    const nonce = 0;
    await program.rpc.initialize(provider.wallet.publicKey, nonce, {
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

  it.skip("Initializes obligation account", async () => {
    // Create transaction to create all accounts (need to avoid tx limit)
    let tx = new anchor.web3.Transaction();
    let create_signers = []

    const transferAuthority = new anchor.web3.Account();

    // Setup lending market for reserves
    const lendingMarket = new anchor.web3.Account();
    create_signers.push(lendingMarket);
    tx.add(await createSolAccountInstruction(lendingMarket, provider, lendingProgram, 160, provider.wallet.publicKey));
    tx.add(initLendingMarketInstruction(
      lendingMarket.publicKey, // new account key
      provider.wallet.publicKey, // market owner
      liquidityMint, // quote mint
      lendingProgram,
    ),
    );

    // TODO can remove this and not split txs, doing here to pinpoint issues while testing
    // Split the txs into two, because over cap
    await provider.send(tx, create_signers);
    tx = new anchor.web3.Transaction();
    create_signers = []

    let [
      _lendingMarketAuthority,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [lendingMarket.publicKey.toBuffer()],
      lendingProgram
    );
    const lendingMarketAuthority = _lendingMarketAuthority;

    // Initialize reserves
    const depositReserve = new anchor.web3.Account();
    create_signers.push(depositReserve);
    create_signers.push(transferAuthority);
    tx.add(await createSolAccountInstruction(depositReserve, provider, lendingProgram, 602, provider.wallet.publicKey));

    tx.add(initReserveInstruction(
      new anchor.BN(10000), // liquidity
      liquidityVault, // from
      userCollateral, // to (init)
      depositReserve.publicKey, // reserve account
      liquidityMint, // liquidity mint
      liquidityReserveVault, // liq supply (init)
      collateralMint, // coll mint (init)
      collateralReserveVault, // col supply (init)
      collateralFeesReceiver, // col output (init)
      lendingMarket.publicKey, // lending market
      provider.wallet.publicKey, // lending market owner
      provider.wallet.publicKey, // lending market auth
      transferAuthority.publicKey, // transferauth
      lendingProgram, // Lending program
    ),
    );

    // TODO can remove this and not split txs, doing here to pinpoint issues while testing
    // Split the txs into two, because over cap
    await provider.send(tx, create_signers);
    tx = new anchor.web3.Transaction();
    create_signers = []

    // TODO not certain about these, should the reserve be the same or swapped pair?
    const borrowReserve = new anchor.web3.Account();
    create_signers.push(borrowReserve);
    tx.add(await createSolAccountInstruction(borrowReserve, provider, lendingProgram, 602, provider.wallet.publicKey));

    tx.add(initReserveInstruction(
      new anchor.BN(10000), // liquidity
      collateralVault, // from
      userLiquidity, // to (init)
      borrowReserve.publicKey, // reserve account
      collateralMint, // liquidity mint
      collateralReserveVault, // liq supply (init)
      liquidityMint, // coll mint (init)
      liquidityReserveVault, // col supply (init)
      liquidityFeesReceiver, // col output (init)
      lendingMarket.publicKey, // lending market
      provider.wallet.publicKey, // lending market owner
      provider.wallet.publicKey, // lending market auth
      provider.wallet.publicKey, // transferauth
      lendingProgram, // Lending program
    ),
    );

    // Split the txs into two, because over cap
    await provider.send(tx, create_signers);
    tx = new anchor.web3.Transaction();
    create_signers = []

    const obligation = new anchor.web3.Account();
    create_signers.push(obligation);
    tx.add(await createSolAccountInstruction(obligation, provider, program.programId, 500, provider.wallet.publicKey));

    // Lending obligation output account
    const obligationTokenOutput = new anchor.web3.Account();
    create_signers.push(obligationTokenOutput);
    tx.add(await createSolAccountInstruction(obligationTokenOutput, provider, program.programId, 500, provider.wallet.publicKey));

    const obligationTokenOwner = provider.wallet.publicKey;

    // Execute the transaction against the cluster.
    await provider.send(tx, create_signers);


    await program.rpc.initObligation({
      accounts: {
        lendingProgram,
        depositReserve: depositReserve.publicKey,
        borrowReserve: borrowReserve.publicKey,
        obligation: obligation.publicKey,
        obligationTokenMint: obligationMint,
        obligationTokenOutput: obligationTokenOutput.publicKey,
        obligationTokenOwner: obligationTokenOwner.publicKey,
        lendingMarket: lendingMarket.publicKey,
        lendingMarketAuthority,

        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
  });
});

async function createSolAccountInstruction(account, provider, program, size, from) {
  return anchor.web3.SystemProgram.createAccount({
    fromPubkey: from,
    newAccountPubkey: account.publicKey,
    space: size,
    lamports: await provider.connection.getMinimumBalanceForRentExemption(
      size
    ),
    programId: program,
  });
}

// * ported/modified from lending frontend
const initReserveInstruction = (
  liquidityAmount,

  from,
  to,

  reserveAccount,
  liquidityMint,
  liquiditySupply,
  collateralMint,
  collateralSupply,
  collateralOutput,
  lendingMarket,
  lendingMarketOwner,
  lendingMarketAuthority,
  transferAuthority,

  lendingProgram,
) => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8("instruction"),
    uint64("liquidityAmount"),
    BufferLayout.u8("optimalUtilizationRate"),
    BufferLayout.u8("loanToValueRatio"),
    BufferLayout.u8("liquidationBonus"),
    BufferLayout.u8("liquidationThreshold"),
    BufferLayout.u8("minBorrowRate"),
    BufferLayout.u8("optimalBorrowRate"),
    BufferLayout.u8("maxBorrowRate"),
    uint64("borrowFeeWad"),
    BufferLayout.u8("hostFeePercentage"),
  ]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 1, // Init reserve instruction
      // * Params taken from sol reserve config on mainnet
      optimalUtilizationRate: 80,
      liquidityAmount: new anchor.BN(liquidityAmount),
      loanToValueRatio: 75,
      liquidationBonus: 10,
      liquidationThreshold: 80,
      minBorrowRate: 0,
      optimalBorrowRate: 2,
      maxBorrowRate: 15,
      borrowFeeWad: new anchor.BN(1_000_000_000_000),
      hostFeePercentage: 20,
    },
    data
  );

  const keys = [
    { pubkey: from, isSigner: false, isWritable: true },
    { pubkey: to, isSigner: false, isWritable: true },
    { pubkey: reserveAccount, isSigner: false, isWritable: true },
    { pubkey: liquidityMint, isSigner: false, isWritable: false },
    { pubkey: liquiditySupply, isSigner: false, isWritable: true },
    { pubkey: collateralMint, isSigner: false, isWritable: true },
    { pubkey: collateralSupply, isSigner: false, isWritable: true },
    { pubkey: collateralOutput, isSigner: false, isWritable: true },

    // Oyster had lending market a signer, seems wrong
    { pubkey: lendingMarket, isSigner: false, isWritable: true },
    { pubkey: lendingMarketOwner, isSigner: true, isWritable: true },
    { pubkey: lendingMarketAuthority, isSigner: false, isWritable: false },
    { pubkey: transferAuthority, isSigner: false, isWritable: false },
    { pubkey: anchor.web3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: TokenInstructions.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];
  return new anchor.web3.TransactionInstruction({
    keys,
    programId: lendingProgram,
    data,
  });
};

const initLendingMarketInstruction = (
  lendingMarketPubkey,
  lendingMarketOwner,
  quoteTokenMint,

  lendingProgram,
) => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8("instruction"),
    publicKey("marketOwner"),
  ]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 0, // Init lending market
      marketOwner: lendingMarketOwner
    },
    data
  );

  const keys = [
    { pubkey: lendingMarketPubkey, isSigner: false, isWritable: true },
    { pubkey: quoteTokenMint, isSigner: false, isWritable: false },
    { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: TokenInstructions.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];
  return new anchor.web3.TransactionInstruction({
    keys,
    programId: lendingProgram,
    data,
  });
};

const uint64 = (property = "uint64") => {
  const layout = BufferLayout.blob(8, property);

  const _decode = layout.decode.bind(layout);
  const _encode = layout.encode.bind(layout);

  layout.decode = (buffer, offset) => {
    const data = _decode(buffer, offset);
    return new BN(
      [...data]
        .reverse()
        .map((i) => `00${i.toString(16)}`.slice(-2))
        .join(""),
      16
    );
  };

  layout.encode = (num, buffer, offset) => {
    const a = num.toArray().reverse();
    let b = Buffer.from(a);
    if (b.length !== 8) {
      const zeroPad = Buffer.alloc(8);
      b.copy(zeroPad);
      b = zeroPad;
    }
    return _encode(b, buffer, offset);
  };

  return layout;
};

const publicKey = (property = "publicKey") => {
  const publicKeyLayout = BufferLayout.blob(32, property);

  const _decode = publicKeyLayout.decode.bind(publicKeyLayout);
  const _encode = publicKeyLayout.encode.bind(publicKeyLayout);

  publicKeyLayout.decode = (buffer, offset) => {
    const data = _decode(buffer, offset);
    return new PublicKey(data);
  };

  publicKeyLayout.encode = (key, buffer, offset) => {
    return _encode(key.toBuffer(), buffer, offset);
  };

  return publicKeyLayout;
};
