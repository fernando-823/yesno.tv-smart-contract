import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from './App';

jest.mock('@solana/wallet-adapter-react', () => ({
  useConnection: () => ({ connection: {} }),
  useWallet: () => ({
    publicKey: null,
    signTransaction: null,
    wallets: [],
  }),
}));

jest.mock('@solana/wallet-adapter-react-ui', () => ({
  WalletMultiButton: () => null,
}));

test('renders nav links', () => {
  render(
    <MemoryRouter initialEntries={['/user']}>
      <App />
    </MemoryRouter>
  );
  expect(screen.getByText('Admin')).toBeInTheDocument();
  expect(screen.getByText('User')).toBeInTheDocument();
});
