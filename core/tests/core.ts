import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SoulboardCore as Core } from "../target/types/soulboard_core";
import { assert } from "chai";

describe("core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SoulboardCore as Program<Core>;

  const adProvider = anchor.web3.Keypair.generate();
  const advertiser = anchor.web3.Keypair.generate();
  const adProviderTwo = anchor.web3.Keypair.generate();

  const adProviderPDA = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ad_provider"), adProvider.publicKey.toBuffer()],
    program.programId
  )[0];

  const adProviderTwoPDA = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ad_provider"), adProviderTwo.publicKey.toBuffer()],
    program.programId
  )[0];

  const campaignId = new anchor.BN(1);
  const campaignPDA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("campaign"),
      advertiser.publicKey.toBuffer(),
      campaignId.toBuffer("le", 4),
    ],
    program.programId
  )[0];
  const deviceId = new anchor.BN(123);
  const deviceIdTwo = new anchor.BN(456);
  const campaignIdTwo = new anchor.BN(2);
  const campaignPDATwo = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("campaign"),
      advertiser.publicKey.toBuffer(),
      campaignIdTwo.toBuffer("le", 4),
    ],
    program.programId
  )[0];

  it("Airdrop", async () => {
    const adProviderSig = await provider.connection.requestAirdrop(
      adProvider.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockhashAd = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhashAd.blockhash,
      lastValidBlockHeight: latestBlockhashAd.lastValidBlockHeight,
      signature: adProviderSig,
    });

    const advertiserSig = await provider.connection.requestAirdrop(
      advertiser.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockhashAdvertiser =
      await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhashAdvertiser.blockhash,
      lastValidBlockHeight: latestBlockhashAdvertiser.lastValidBlockHeight,
      signature: advertiserSig,
    });

    const adProviderTwoSig = await provider.connection.requestAirdrop(
      adProviderTwo.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockhashAdTwo =
      await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockhashAdTwo.blockhash,
      lastValidBlockHeight: latestBlockhashAdTwo.lastValidBlockHeight,
      signature: adProviderTwoSig,
    });
  });

  it("Registers a provider", async () => {
    await program.methods
      .registerProvider()
      .accounts({
        adProvider: adProviderPDA,
        authority: adProvider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([adProvider])
      .rpc();

    let pda = await program.account.adProvider.fetch(adProviderPDA);
    assert.ok(pda.authority.equals(adProvider.publicKey));
  });

  it("Gets a device", async () => {
    await program.methods
      .getDevice(deviceId.toNumber())
      .accounts({
        adProvider: adProviderPDA,
        authority: adProvider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([adProvider])
      .rpc();

    let pda = await program.account.adProvider.fetch(adProviderPDA);
    assert.equal(pda.devices[0].deviceId, deviceId.toNumber());
    assert.equal(pda.devices[0].deviceState.available,
      pda.devices[0].deviceState.available
    );
  });

  it("Creates a campaign", async () => {
    const campaignName = "Test Campaign";
    const campaignDescription = "This is a test campaign";

    await program.methods
      .createCampaign(
        campaignId.toNumber(),
        campaignName,
        campaignDescription
      )
      .accounts({
        campaign: campaignPDA,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    let pda = await program.account.campaign.fetch(campaignPDA);
    assert.ok(pda.authority.equals(advertiser.publicKey));
    assert.equal(pda.campaignName, campaignName);
  });

  it("Adds budget to a campaign", async () => {
    const budget = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
    await program.methods
      .addBudget(campaignId.toNumber(), budget)
      .accounts({
        campaign: campaignPDA,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    let pda = await program.account.campaign.fetch(campaignPDA);
    assert.ok(pda.campaignBudget.eq(budget));
  });

  it("Adds a location to a campaign", async () => {
    await program.methods
      .addLocation(
        campaignId.toNumber(),
        adProvider.publicKey,
        deviceId.toNumber()
      )
      .accounts({
        campaign: campaignPDA,
        adProvider: adProviderPDA,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    let campaignAccount = await program.account.campaign.fetch(campaignPDA);
    let providerAccount = await program.account.adProvider.fetch(
      adProviderPDA
    );

    assert.ok(
      campaignAccount.campaignProviders[0].equals(providerAccount.authority)
    );
    assert.ok(
      campaignAccount.campaignLocations[0].equals(adProvider.publicKey)
    );
    assert.equal(providerAccount.devices[0].deviceState.booked,
      providerAccount.devices[0].deviceState.booked
    );
  });

  it("Removes a location from a campaign", async () => {
    await program.methods
      .removeLocation(
        campaignId.toNumber(),
        adProvider.publicKey,
        deviceId.toNumber()
      )
      .accounts({
        campaign: campaignPDA,
        adProvider: adProviderPDA,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    let campaignAccount = await program.account.campaign.fetch(campaignPDA);
    let providerAccount = await program.account.adProvider.fetch(
      adProviderPDA
    );

    assert.equal(campaignAccount.campaignProviders.length, 0);
    assert.equal(campaignAccount.campaignLocations.length, 0);
    assert.equal(providerAccount.devices[0].deviceState.available,
      providerAccount.devices[0].deviceState.available
    );
  });

  it("Books a device from a different provider", async () => {
    // Register the second provider
    await program.methods
      .registerProvider()
      .accounts({
        adProvider: adProviderTwoPDA,
        authority: adProviderTwo.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([adProviderTwo])
      .rpc();

    // Add a device for the second provider
    await program.methods
      .getDevice(deviceIdTwo.toNumber())
      .accounts({
        adProvider: adProviderTwoPDA,
        authority: adProviderTwo.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([adProviderTwo])
      .rpc();

    // Create a new campaign for the original advertiser
    const campaignName = "Test Campaign 2";
    const campaignDescription = "This is a second test campaign";
    await program.methods
      .createCampaign(
        campaignIdTwo.toNumber(),
        campaignName,
        campaignDescription
      )
      .accounts({
        campaign: campaignPDATwo,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    // Book the device from the second provider
    await program.methods
      .addLocation(
        campaignIdTwo.toNumber(),
        adProviderTwo.publicKey,
        deviceIdTwo.toNumber()
      )
      .accounts({
        campaign: campaignPDATwo,
        adProvider: adProviderTwoPDA,
        authority: advertiser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([advertiser])
      .rpc();

    let campaignAccount = await program.account.campaign.fetch(campaignPDATwo);
    let providerAccount = await program.account.adProvider.fetch(
      adProviderTwoPDA
    );

    assert.ok(
      campaignAccount.campaignProviders[0].equals(providerAccount.authority)
    );
    assert.ok(
      campaignAccount.campaignLocations[0].equals(adProviderTwo.publicKey)
    );
    assert.equal(providerAccount.devices[0].deviceState.booked,
      providerAccount.devices[0].deviceState.booked
    );
  });
});
