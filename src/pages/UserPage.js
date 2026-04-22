import { useCallback, useMemo, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { getAssociatedTokenAddressSync } from '@solana/spl-token';

import {
  deriveMarketPda,
  deriveUserPda,
  deriveUserPositionPda,
  deriveVaultPda,
  getYesnoEscrowProgram,
  PROGRAM_ID,
  SYS_ACCOUNTS,
} from '../solana/yesnoEscrow';

const inputStyle = {
  width: '100%',
  padding: 10,
  borderRadius: 8,
  border: '1px solid #333',
  background: '#111',
  color: '#fff',
};

const btnStyle = { padding: '10px 14px', borderRadius: 10, border: 0, cursor: 'pointer' };
const sectionTitle = { fontSize: 14, fontWeight: 700, marginTop: 20, marginBottom: 8, opacity: 0.95 };

function parsePubkey(label, s) {
  const t = (s || '').trim();
  if (!t) throw new Error(`${label} is required`);
  try {
    return new PublicKey(t);
  } catch {
    throw new Error(`Invalid ${label} public key`);
  }
}

export default function UserPage() {
  const { connection } = useConnection();
  const wallet = useWallet();

  const program = useMemo(() => {
    if (!wallet?.publicKey || !wallet?.signTransaction) return null;
    return getYesnoEscrowProgram(connection, wallet);
  }, [connection, wallet]);

  const [usdcMint, setUsdcMint] = useState(process.env.REACT_APP_USDC_MINT || '');
  const [marketId, setMarketId] = useState('1');
  const [status, setStatus] = useState('');

  const [userTokenAccount, setUserTokenAccount] = useState('');
  const [betAmount, setBetAmount] = useState('1000000');
  const [betSideYes, setBetSideYes] = useState(true);
  const [referrerPk, setReferrerPk] = useState('');

  const setErr = useCallback((e) => {
    const msg = e?.message || String(e);
    setStatus(`Error: ${msg}`);
    console.error(e);
  }, []);

  const marketContext = useCallback(() => {
    const idBn = new BN(marketId);
    const [market] = deriveMarketPda(idBn);
    const [vault] = deriveVaultPda(market);
    const mintPk = parsePubkey('USDC mint', usdcMint);
    return { idBn, market, vault, mintPk };
  }, [marketId, usdcMint]);

  const fillMyAta = useCallback(() => {
    if (!wallet.publicKey || !usdcMint.trim()) {
      setStatus('Connect wallet and set USDC mint first');
      return;
    }
    try {
      const mint = new PublicKey(usdcMint.trim());
      const ata = getAssociatedTokenAddressSync(mint, wallet.publicKey);
      setUserTokenAccount(ata.toBase58());
      setStatus(`Filled your USDC ATA: ${ata.toBase58()}`);
    } catch (e) {
      setErr(e);
    }
  }, [usdcMint, wallet.publicKey, setErr]);

  const onPlaceBet = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending placeBet...');
    try {
      const { idBn, market, vault, mintPk } = marketContext();
      const [userAccount] = deriveUserPda(wallet.publicKey);
      const [userPosition] = deriveUserPositionPda(market, wallet.publicKey);
      const refTrim = referrerPk.trim();
      const referrerOpt = refTrim ? parsePubkey('Referrer', refTrim) : null;
      const amount = new BN(betAmount);
      const sig = await program.methods
        .placeBet(idBn, betSideYes, amount, referrerOpt)
        .accounts({
          market,
          vault,
          userAccount,
          userPosition,
          userWallet: wallet.publicKey,
          userTokenAccount: parsePubkey('Your USDC token account', userTokenAccount),
          usdcMint: mintPk,
          systemProgram: SYS_ACCOUNTS.systemProgram,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
        })
        .rpc();
      setStatus(`placeBet tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [betAmount, betSideYes, marketContext, program, referrerPk, userTokenAccount, wallet.publicKey, setErr]);

  const onSwitchSide = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending switchSide...');
    try {
      const { market } = marketContext();
      const [userPosition] = deriveUserPositionPda(market, wallet.publicKey);
      const sig = await program.methods
        .switchSide()
        .accounts({
          market,
          userPosition,
          userWallet: wallet.publicKey,
        })
        .rpc();
      setStatus(`switchSide tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, wallet.publicKey, setErr]);

  const onClaimWinner = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending claimWinner...');
    try {
      const { market, mintPk } = marketContext();
      const [vault] = deriveVaultPda(market);
      const [userPosition] = deriveUserPositionPda(market, wallet.publicKey);
      const sig = await program.methods
        .claimWinner()
        .accounts({
          market,
          vault,
          userPosition,
          userWallet: wallet.publicKey,
          userTokenAccount: parsePubkey('Your USDC token account', userTokenAccount),
          usdcMint: mintPk,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
        })
        .rpc();
      setStatus(`claimWinner tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, userTokenAccount, wallet.publicKey, setErr]);

  const onClaimVoid = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending claimVoid...');
    try {
      const { market, mintPk } = marketContext();
      const [vault] = deriveVaultPda(market);
      const [userPosition] = deriveUserPositionPda(market, wallet.publicKey);
      const sig = await program.methods
        .claimVoid()
        .accounts({
          market,
          vault,
          userPosition,
          userWallet: wallet.publicKey,
          userTokenAccount: parsePubkey('Your USDC token account', userTokenAccount),
          usdcMint: mintPk,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
        })
        .rpc();
      setStatus(`claimVoid tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, userTokenAccount, wallet.publicKey, setErr]);

  const canSend = Boolean(program && wallet.publicKey);

  return (
    <div style={{ maxWidth: 900, width: '100%', textAlign: 'left' }}>
      <div style={{ opacity: 0.8, fontSize: 12 }}>Program: {PROGRAM_ID.toBase58()}</div>

      <div style={{ marginTop: 18, display: 'grid', gap: 10 }}>
        <label>
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>USDC Mint (cluster-dependent)</div>
          <input style={inputStyle} value={usdcMint} onChange={(e) => setUsdcMint(e.target.value)} placeholder="Mint public key" />
        </label>

        <label>
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>Market ID (u64)</div>
          <input style={inputStyle} value={marketId} onChange={(e) => setMarketId(e.target.value)} placeholder="1" />
        </label>

        <div style={sectionTitle}>Betting (connected wallet)</div>
        <label>
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>Your USDC token account (source for bets / destination for claims)</div>
          <input
            style={inputStyle}
            value={userTokenAccount}
            onChange={(e) => setUserTokenAccount(e.target.value)}
            placeholder="ATA / token account pubkey"
          />
        </label>
        <button type="button" onClick={fillMyAta} disabled={!wallet.publicKey} style={{ ...btnStyle, alignSelf: 'flex-start' }}>
          Fill my ATA for mint above
        </button>

        <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' }}>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={betSideYes} onChange={() => setBetSideYes(true)} /> YES
          </label>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={!betSideYes} onChange={() => setBetSideYes(false)} /> NO
          </label>
          <label style={{ flex: 1, minWidth: 160 }}>
            <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 4 }}>Amount (raw units, 6 decimals)</div>
            <input style={inputStyle} value={betAmount} onChange={(e) => setBetAmount(e.target.value)} />
          </label>
        </div>

        <label>
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>Referrer (optional; locked on first bet)</div>
          <input style={inputStyle} value={referrerPk} onChange={(e) => setReferrerPk(e.target.value)} placeholder="Empty = none" />
        </label>

        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <button disabled={!canSend} onClick={onPlaceBet} style={btnStyle}>
            Place bet
          </button>
          <button disabled={!canSend} onClick={onSwitchSide} style={btnStyle}>
            Switch side
          </button>
        </div>

        <div style={sectionTitle}>Claims</div>
        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <button disabled={!canSend} onClick={onClaimWinner} style={btnStyle}>
            Claim winner
          </button>
          <button disabled={!canSend} onClick={onClaimVoid} style={btnStyle}>
            Claim void refund
          </button>
        </div>

        <div style={{ marginTop: 14, fontSize: 12, opacity: 0.85, wordBreak: 'break-word' }}>
          <div>
            <strong>Wallet</strong>: {wallet.publicKey ? wallet.publicKey.toBase58() : 'Not connected'}
          </div>
          <div>
            <strong>Status</strong>: {status || '—'}
          </div>
        </div>
      </div>
    </div>
  );
}

