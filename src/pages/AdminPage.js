import { useCallback, useMemo, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

import {
  deriveConfigPda,
  deriveMarketPda,
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

export default function AdminPage() {
  const { connection } = useConnection();
  const wallet = useWallet();

  const program = useMemo(() => {
    if (!wallet?.publicKey || !wallet?.signTransaction) return null;
    return getYesnoEscrowProgram(connection, wallet);
  }, [connection, wallet]);

  const [usdcMint, setUsdcMint] = useState(process.env.REACT_APP_USDC_MINT || '');
  const [platformTreasury, setPlatformTreasury] = useState(process.env.REACT_APP_PLATFORM_TREASURY || '');
  const [marketId, setMarketId] = useState('1');
  const [status, setStatus] = useState('');

  const [proposeYesWins, setProposeYesWins] = useState(true);
  const [finalizeYesWins, setFinalizeYesWins] = useState(true);

  const setErr = useCallback((e) => {
    const msg = e?.message || String(e);
    setStatus(`Error: ${msg}`);
    console.error(e);
  }, []);

  const marketContext = useCallback(() => {
    const idBn = new BN(marketId);
    const [config] = deriveConfigPda();
    const [market] = deriveMarketPda(idBn);
    const [vault] = deriveVaultPda(market);
    const mintPk = parsePubkey('USDC mint', usdcMint);
    return { idBn, config, market, vault, mintPk };
  }, [marketId, usdcMint]);

  const onInitializeConfig = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending initializeConfig...');
    try {
      const [config] = deriveConfigPda();
      const sig = await program.methods
        .initializeConfig()
        .accounts({
          config,
          admin: wallet.publicKey,
          usdcMint: parsePubkey('USDC mint', usdcMint),
          platformTreasury: parsePubkey('Platform treasury', platformTreasury),
          systemProgram: SYS_ACCOUNTS.systemProgram,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
        })
        .rpc();
      setStatus(`initializeConfig tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [platformTreasury, program, usdcMint, wallet.publicKey, setErr]);

  const onCreateMarket = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending createMarket...');
    try {
      const { idBn, config, market, vault, mintPk } = marketContext();
      const sig = await program.methods
        .createMarket(idBn, wallet.publicKey)
        .accounts({
          config,
          admin: wallet.publicKey,
          market,
          vault,
          usdcMint: mintPk,
          systemProgram: SYS_ACCOUNTS.systemProgram,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
          rent: SYS_ACCOUNTS.rent,
        })
        .rpc();
      setStatus(`createMarket tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, wallet.publicKey, setErr]);

  const onProposeOutcome = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending proposeOutcome...');
    try {
      const { config, market } = marketContext();
      const sig = await program.methods
        .proposeOutcome(proposeYesWins)
        .accounts({
          config,
          admin: wallet.publicKey,
          market,
        })
        .rpc();
      setStatus(`proposeOutcome tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, proposeYesWins, wallet.publicKey, setErr]);

  const onFinalize = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending finalize...');
    try {
      const { config, market } = marketContext();
      const sig = await program.methods
        .finalize(finalizeYesWins)
        .accounts({
          config,
          admin: wallet.publicKey,
          market,
        })
        .rpc();
      setStatus(`finalize tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [finalizeYesWins, marketContext, program, wallet.publicKey, setErr]);

  const onVoidMarket = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending voidMarket...');
    try {
      const { config, market } = marketContext();
      const sig = await program.methods
        .voidMarket()
        .accounts({
          config,
          admin: wallet.publicKey,
          market,
        })
        .rpc();
      setStatus(`voidMarket tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, program, wallet.publicKey, setErr]);

  const onClaimPlatform = useCallback(async () => {
    if (!program || !wallet.publicKey) return;
    setStatus('Sending claimPlatform...');
    try {
      const { config, market, mintPk } = marketContext();
      const [vault] = deriveVaultPda(market);
      const treasury = parsePubkey('Platform treasury', platformTreasury);
      const sig = await program.methods
        .claimPlatform()
        .accounts({
          config,
          admin: wallet.publicKey,
          market,
          vault,
          platformTreasury: treasury,
          usdcMint: mintPk,
          tokenProgram: SYS_ACCOUNTS.tokenProgram,
        })
        .rpc();
      setStatus(`claimPlatform tx: ${sig}`);
    } catch (e) {
      setErr(e);
    }
  }, [marketContext, platformTreasury, program, wallet.publicKey, setErr]);

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
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>Platform Treasury (USDC token account)</div>
          <input
            style={inputStyle}
            value={platformTreasury}
            onChange={(e) => setPlatformTreasury(e.target.value)}
            placeholder="Token account public key"
          />
        </label>

        <label>
          <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 6 }}>Market ID (u64)</div>
          <input style={inputStyle} value={marketId} onChange={(e) => setMarketId(e.target.value)} placeholder="1" />
        </label>

        <div style={{ fontSize: 12, opacity: 0.75 }}>
          Admin flow: initialize config → create market → propose outcome → finalize/void → claim platform.
        </div>

        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'flex-end' }}>
          <button disabled={!canSend} onClick={onInitializeConfig} style={btnStyle}>
            Initialize Config (admin)
          </button>
          <button disabled={!canSend} onClick={onCreateMarket} style={btnStyle}>
            Create Market
          </button>
        </div>

        <div style={sectionTitle}>Resolution</div>
        <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' }}>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={proposeYesWins} onChange={() => setProposeYesWins(true)} /> Propose YES wins
          </label>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={!proposeYesWins} onChange={() => setProposeYesWins(false)} /> Propose NO wins
          </label>
          <button disabled={!canSend} onClick={onProposeOutcome} style={btnStyle}>
            Propose outcome
          </button>
        </div>

        <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' }}>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={finalizeYesWins} onChange={() => setFinalizeYesWins(true)} /> Finalize YES wins
          </label>
          <label style={{ fontSize: 13 }}>
            <input type="radio" checked={!finalizeYesWins} onChange={() => setFinalizeYesWins(false)} /> Finalize NO wins
          </label>
          <button disabled={!canSend} onClick={onFinalize} style={btnStyle}>
            Finalize
          </button>
          <button disabled={!canSend} onClick={onVoidMarket} style={btnStyle}>
            Void market
          </button>
        </div>

        <div style={sectionTitle}>Platform sweep</div>
        <button disabled={!canSend} onClick={onClaimPlatform} style={btnStyle}>
          Claim platform (admin → treasury)
        </button>

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

