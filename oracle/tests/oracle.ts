/*****************************************************************
 * tests/oracle_depin.ts
 * ---------------------------------------------------------------
 * Run with:  anchor test
 *
 * You can override the URL with:
 *   THINGSPEAK_URL="https://api.thingspeak.com/…/channels/<id>/…" anchor test
 *****************************************************************/

import * as anchor from "@coral-xyz/anchor";
import { BN } from "bn.js";
import { assert, expect } from "chai";
import { Oracle } from "../target/types/oracle";

/* ────────────────────────── config ──────────────────────────── */

// Default URL = the one you posted; override via env if you like
const THINGSPEAK_URL =
  process.env.THINGSPEAK_URL ??
  "https://api.thingspeak.com/channels/2890626/fields/1.json?results=11";

/** extract the numeric channel id from “…/channels/<num>/…” */
function extractChannelId(url: string): number {
  const match = url.match(/channels\/(\d+)\//);
  if (!match) throw new Error("Unable to parse channelId from URL");
  return Number(match[1]);
}

const CHANNEL_ID = extractChannelId(THINGSPEAK_URL);

/* Test constants (simulate two sequential keeper pushes) */
const INITIAL_VIEWS = 17n;
const INITIAL_TAPS = 5n;
const NEXT_VIEWS = 9n;
const NEXT_TAPS = 2n;

/* ───────────────────── boiler-plate setup ───────────────────── */

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const connection = provider.connection;
const wallet = provider.wallet;

const program = anchor.workspace
  .Oracle as anchor.Program<Oracle>;

const DEVICE_SEED = Buffer.from("device_feed");

// Will be derived in before() hook
let feedPda: anchor.web3.PublicKey;
let bump!: number;

/* ────────────────────────── test-suite ─────────────────────── */

describe(`oracle-depin program (channel ${CHANNEL_ID})`, () => {
  before(async () => {
    [feedPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        DEVICE_SEED,
        new BN(CHANNEL_ID).toArrayLike(Buffer, "le", 4), // u32-LE
      ],
      program.programId
    );
  });

  it("initialises the device feed and emits DeviceFeedInitialized", async () => {
    const initEvt = new Promise<any>((resolve) => {
      const listener = program.addEventListener(
        "deviceFeedInitialized",
        (ev) => {
          program.removeEventListener(listener).catch(() => { });
          resolve(ev);
        }
      );
    });

    await program.methods
      .initializeDeviceFeed(CHANNEL_ID, bump)
      .accounts({
        feed: feedPda,
        payer: wallet.publicKey,
        authority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const ev = await initEvt;
    assert.equal(ev.channelId, CHANNEL_ID, "wrong channel in event");

    const acc = await program.account.deviceFeed.fetch(feedPda);
    assert.equal(acc.channelId, CHANNEL_ID);
    assert.equal(acc.totalViews.toNumber(), 0);
    assert.equal(acc.totalTaps.toNumber(), 0);
    assert.equal(acc.lastEntryId, 0);
    assert.equal(acc.authority.toBase58(), wallet.publicKey.toBase58());
  });

  it("updates the feed, increments counters, emits DeviceFeedUpdated", async () => {
    const NEW_ENTRY_ID = 104;

    const updEvt = new Promise<any>((resolve) => {
      const listener = program.addEventListener(
        "deviceFeedUpdated",
        (ev) => {
          program.removeEventListener(listener).catch(() => { });
          resolve(ev);
        }
      );
    });

    await program.methods
      .updateDeviceFeed(
        CHANNEL_ID,
        NEW_ENTRY_ID,
        new BN(INITIAL_VIEWS),
        new BN(INITIAL_TAPS)
      )
      .accounts({ feed: feedPda, signer: wallet.publicKey })
      .rpc();

    const ev = await updEvt;
    assert.equal(ev.newEntryId, NEW_ENTRY_ID);
    assert.equal(ev.deltaViews.toString(), INITIAL_VIEWS.toString());
    assert.equal(ev.deltaTaps.toString(), INITIAL_TAPS.toString());

    const acc = await program.account.deviceFeed.fetch(feedPda);
    assert.equal(acc.lastEntryId, NEW_ENTRY_ID);
    assert.equal(acc.totalViews.toString(), INITIAL_VIEWS.toString());
    assert.equal(acc.totalTaps.toString(), INITIAL_TAPS.toString());
  });

  it("rejects an update from a *lower* entry_id (NoNewData)", async () => {
    try {
      await program.methods
        .updateDeviceFeed(
          CHANNEL_ID,
          50, // lower than last_entry_id (104)
          new BN(1),
          new BN(1)
        )
        .accounts({ feed: feedPda, signer: wallet.publicKey })
        .rpc();
      assert.fail("transaction should have reverted");
    } catch (err: any) {
      expect(err.error.errorCode.code).to.equal("NoNewData");
    }
  });

  it("rejects an update from a non-authority signer (BadAuthority)", async () => {
    const rogue = anchor.web3.Keypair.generate();
    await connection.confirmTransaction(
      await connection.requestAirdrop(rogue.publicKey, 1_000_000_000)
    );

    try {
      await program.methods
        .updateDeviceFeed(
          CHANNEL_ID,
          105,
          new BN(NEXT_VIEWS),
          new BN(NEXT_TAPS)
        )
        .accounts({ feed: feedPda, signer: rogue.publicKey })
        .signers([rogue])
        .rpc();
      assert.fail("rogue update should fail");
    } catch (err: any) {
      expect(err.error.errorCode.code).to.equal("BadAuthority");
    }
  });

  it("handles a valid second update (totals accumulate)", async () => {
    const NEW_ENTRY_ID = 105;

    await program.methods
      .updateDeviceFeed(
        CHANNEL_ID,
        NEW_ENTRY_ID,
        new BN(NEXT_VIEWS),
        new BN(NEXT_TAPS)
      )
      .accounts({ feed: feedPda, signer: wallet.publicKey })
      .rpc();

    const acc = await program.account.deviceFeed.fetch(feedPda);
    assert.equal(
      acc.totalViews.toString(),
      (INITIAL_VIEWS + NEXT_VIEWS).toString()
    );
    assert.equal(
      acc.totalTaps.toString(),
      (INITIAL_TAPS + NEXT_TAPS).toString()
    );
    assert.equal(acc.lastEntryId, NEW_ENTRY_ID);
  });
});