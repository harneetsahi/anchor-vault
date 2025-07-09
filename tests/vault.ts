import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import { Vault } from "../target/types/vault";
import { assert } from "chai";

describe("vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.vault as Program<Vault>;

  const vaultState = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), provider.publicKey.toBuffer()],
    program.programId
  )[0];

  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultState.toBuffer()],
    program.programId
  )[0];

  const getBalance = async (pubkey: PublicKey) => {
    return provider.connection.getBalance(pubkey);
  };

  it("Is initialized!", async () => {
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        signer: provider.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposits SOL", async () => {
    const beforeVaultBalance = await getBalance(vault);

    console.log(
      "Vault balance before deposit",
      beforeVaultBalance / LAMPORTS_PER_SOL,
      "SOL"
    );

    const amount = 1 * LAMPORTS_PER_SOL;

    const tx = await program.methods
      .deposit(new anchor.BN(amount))
      .accountsPartial({
        signer: provider.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Deposit tx signature", tx);

    const afterVaultBalance = await getBalance(vault);

    console.log(
      "Vault balance after deposit",
      afterVaultBalance / LAMPORTS_PER_SOL,
      "SOL"
    );

    assert.equal(
      afterVaultBalance,
      beforeVaultBalance + amount,
      "vault balance should increase by deposit amount"
    );
  });

  it("Withdraws SOL", async () => {
    const beforeVaultBalance = await getBalance(vault);

    const amount = 1 * LAMPORTS_PER_SOL;

    const tx = await program.methods
      .withraw(new anchor.BN(amount))
      .accountsPartial({
        signer: provider.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Withdraw tx signature", tx);

    const afterVaultBalance = await getBalance(vault);

    assert.equal(
      afterVaultBalance,
      beforeVaultBalance - amount,
      "vault balance should reduce by withdrawal amount"
    );
  });

  it("Closes vault account", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        signer: provider.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("close tx signature", tx);

    const finalVaultStateBalance = await getBalance(vaultState);
    const finalVaultBalance = await getBalance(vault);

    assert.equal(
      finalVaultStateBalance,
      0,
      "vault state balance should be zero"
    );
    assert.equal(finalVaultBalance, 0, "vault balance should be zero");
  });
});
