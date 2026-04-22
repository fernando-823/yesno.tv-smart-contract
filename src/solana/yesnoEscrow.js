import { AnchorProvider, BN, Program } from '@coral-xyz/anchor';
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

import idl from '../idl/yesno_escrow.json';

export const PROGRAM_ID = new PublicKey(idl.address);

export function getYesnoEscrowProgram(connection, wallet) {
  const provider = new AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
  });
  // Anchor JS expects `idl.accounts[].type` (full account type layout).
  // This IDL includes account discriminators but no type layouts, which crashes
  // Program construction when it tries to build `program.account.*` clients.
  // We only need instruction builders (`program.methods.*`) in this app, so
  // drop unsupported account namespace metadata.
  const hasAccountTypes =
    Array.isArray(idl?.accounts) && idl.accounts.every((a) => a && typeof a === 'object' && a.type);
  const safeIdl = hasAccountTypes ? idl : { ...idl, accounts: [] };

  // Anchor v0.31 Program constructor is `new Program(idl, provider)`.
  // The program id is taken from `idl.address`.
  return new Program(safeIdl, provider);
}

export function u64ToLeBuffer(value) {
  const bn = BN.isBN(value) ? value : new BN(value);
  return bn.toArrayLike(Buffer, 'le', 8);
}

export function deriveConfigPda() {
  return PublicKey.findProgramAddressSync([Buffer.from('config')], PROGRAM_ID);
}

export function deriveMarketPda(marketId) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('market'), u64ToLeBuffer(marketId)],
    PROGRAM_ID
  );
}

export function deriveVaultPda(marketPda) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), marketPda.toBuffer()],
    PROGRAM_ID
  );
}

export function deriveUserPda(userWallet) {
  const key = userWallet instanceof PublicKey ? userWallet : new PublicKey(userWallet);
  return PublicKey.findProgramAddressSync([Buffer.from('user'), key.toBuffer()], PROGRAM_ID);
}

export function deriveUserPositionPda(marketPda, userWallet) {
  const marketKey = marketPda instanceof PublicKey ? marketPda : new PublicKey(marketPda);
  const userKey = userWallet instanceof PublicKey ? userWallet : new PublicKey(userWallet);
  return PublicKey.findProgramAddressSync(
    [Buffer.from('position'), marketKey.toBuffer(), userKey.toBuffer()],
    PROGRAM_ID
  );
}

export function deriveAffiliateClaimPda(marketPda, referrer) {
  const marketKey = marketPda instanceof PublicKey ? marketPda : new PublicKey(marketPda);
  const refKey = referrer instanceof PublicKey ? referrer : new PublicKey(referrer);
  return PublicKey.findProgramAddressSync(
    [Buffer.from('affiliate'), marketKey.toBuffer(), refKey.toBuffer()],
    PROGRAM_ID
  );
}

export const SYS_ACCOUNTS = {
  systemProgram: SystemProgram.programId,
  tokenProgram: TOKEN_PROGRAM_ID,
  rent: SYSVAR_RENT_PUBKEY,
};

