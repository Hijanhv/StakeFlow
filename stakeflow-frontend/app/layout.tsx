import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'StakeFlow - Liquid Staking & DeFi Yield Optimization',
  description: 'One-click yield optimization that stakes your CSPR, generates liquid tokens, and automatically compounds returns across DeFi protocols',
  icons: {
    icon: '/logo.png',
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">
        {children}
      </body>
    </html>
  );
}
