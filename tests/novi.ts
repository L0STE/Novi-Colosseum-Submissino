import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Novi } from "../target/types/novi";

describe("novi", () => {
  const wallet = anchor.Wallet.local();
  const provider = anchor.getProvider();
  const connection = provider.connection;
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Novi as Program<Novi>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
