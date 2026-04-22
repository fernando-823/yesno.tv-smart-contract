import './App.css';

import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { Link, Navigate, Route, Routes, useLocation } from 'react-router-dom';

import AdminPage from './pages/AdminPage';
import UserPage from './pages/UserPage';

function NavLink({ to, label }) {
  const location = useLocation();
  const active = location.pathname === to;
  return (
    <Link
      to={to}
      style={{
        color: active ? '#fff' : 'rgba(255,255,255,0.8)',
        textDecoration: 'none',
        padding: '6px 10px',
        borderRadius: 8,
        background: active ? 'rgba(255,255,255,0.12)' : 'transparent',
        border: '1px solid rgba(255,255,255,0.15)',
      }}
    >
      {label}
    </Link>
  );
}

export default function App() {
  return (
    <div className="App">
      <header className="App-header">
        <div style={{ maxWidth: 980, width: '100%', textAlign: 'left' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', gap: 16, alignItems: 'center' }}>
            <div>
              <div style={{ fontSize: 20, fontWeight: 700 }}>YesNo Escrow</div>
              <div style={{ opacity: 0.75, fontSize: 12 }}>Admin creates/resolves. Users place bets/claim.</div>
            </div>
            <WalletMultiButton />
          </div>

          <div style={{ marginTop: 14, display: 'flex', gap: 10, flexWrap: 'wrap' }}>
            <NavLink to="/admin" label="Admin" />
            <NavLink to="/user" label="User" />
          </div>

          <div style={{ marginTop: 18 }}>
            <Routes>
              <Route path="/" element={<Navigate to="/user" replace />} />
              <Route path="/admin" element={<AdminPage />} />
              <Route path="/user" element={<UserPage />} />
              <Route path="*" element={<Navigate to="/user" replace />} />
            </Routes>
          </div>
        </div>
      </header>
    </div>
  );
}
