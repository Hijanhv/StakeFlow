'use client';

import { useState } from 'react';
import Link from 'next/link';
import Image from 'next/image';

export default function Dashboard() {
  const [isConnected, setIsConnected] = useState(true);

  // Mock data - in production, fetch from contract
  const userStats = {
    totalDeposited: '500',
    currentValue: '547.50',
    totalEarned: '47.50',
    apy: '9.5',
    shares: '500',
    daysStaked: 45,
  };

  const validators = [
    { address: '0x1a2b...3c4d', stake: '125.00', score: 98, uptime: 99.8, status: 'active' },
    { address: '0x5e6f...7g8h', stake: '125.00', score: 97, uptime: 99.5, status: 'active' },
    { address: '0x9i0j...1k2l', stake: '125.00', score: 96, uptime: 99.2, status: 'active' },
    { address: '0x3m4n...5o6p', stake: '125.00', score: 99, uptime: 99.9, status: 'active' },
  ];

  const crossChainDeposits = [
    { chain: 'Ethereum', amount: '200', txHash: '0xabc...123', date: '2 days ago' },
    { chain: 'Casper', amount: '300', txHash: '0xdef...456', date: '5 days ago' },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-black to-red-900">
      {/* Navigation */}
      <nav className="border-b border-gray-800 bg-black/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center space-x-8">
              <Link href="/" className="flex items-center space-x-3">
                <Image src="/logo.png" alt="StakeFlow" width={40} height={40} className="rounded-lg" />
                <span className="text-xl font-bold text-white">StakeFlow</span>
              </Link>
              <div className="hidden md:flex space-x-6">
                <Link href="/" className="text-gray-300 hover:text-white transition">
                  Home
                </Link>
                <Link href="/dashboard" className="text-white font-semibold">
                  Dashboard
                </Link>
              </div>
            </div>
            <button
              onClick={() => setIsConnected(!isConnected)}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition text-sm"
            >
              {isConnected ? '🔗 Connected' : 'Connect Wallet'}
            </button>
          </div>
        </div>
      </nav>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Page Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">Portfolio Dashboard</h1>
          <p className="text-gray-400">Track your staking performance and validator metrics</p>
        </div>

        {/* Portfolio Overview */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Total Deposited</div>
            <div className="text-3xl font-bold text-white mb-1">{userStats.totalDeposited} CSPR</div>
            <div className="text-xs text-gray-500">{userStats.shares} shares</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Current Value</div>
            <div className="text-3xl font-bold text-blue-400 mb-1">{userStats.currentValue} CSPR</div>
            <div className="text-xs text-green-400">+{((parseFloat(userStats.currentValue) / parseFloat(userStats.totalDeposited) - 1) * 100).toFixed(2)}%</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Total Earned</div>
            <div className="text-3xl font-bold text-green-400 mb-1">+{userStats.totalEarned} CSPR</div>
            <div className="text-xs text-gray-500">{userStats.apy}% APY</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Risk Score</div>
            <div className="text-3xl font-bold text-purple-400 mb-1">92/100</div>
            <div className="text-xs text-green-400">Excellent</div>
          </div>
        </div>

        {/* Validators Section */}
        <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6 mb-8">
          <div className="flex justify-between items-center mb-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-1">Active Validators</h2>
              <p className="text-gray-400 text-sm">Your stake is diversified across {validators.length} validators</p>
            </div>
            <div className="text-right">
              <div className="text-sm text-gray-400">Avg Performance</div>
              <div className="text-2xl font-bold text-green-400">97.5/100</div>
            </div>
          </div>

          <div className="space-y-4">
            {validators.map((validator, idx) => (
              <div key={idx} className="bg-gray-900/50 rounded-lg p-4 flex items-center justify-between hover:bg-gray-900/70 transition">
                <div className="flex items-center space-x-4">
                  <div className="w-10 h-10 bg-gradient-to-br from-red-600 to-purple-600 rounded-full flex items-center justify-center text-white font-bold">
                    {idx + 1}
                  </div>
                  <div>
                    <div className="text-white font-mono text-sm">{validator.address}</div>
                    <div className="text-gray-400 text-xs mt-1">
                      {validator.stake} CSPR staked • {validator.status}
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-6">
                  <div className="text-center">
                    <div className="text-xs text-gray-400">Score</div>
                    <div className="text-lg font-bold text-white">{validator.score}</div>
                  </div>
                  <div className="text-center">
                    <div className="text-xs text-gray-400">Uptime</div>
                    <div className="text-lg font-bold text-green-400">{validator.uptime}%</div>
                  </div>
                  <div className={`w-3 h-3 rounded-full ${validator.status === 'active' ? 'bg-green-500' : 'bg-gray-500'}`}></div>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-6 bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
            <div className="flex items-center space-x-2 text-blue-300 text-sm">
              <span>💡</span>
              <span>Auto-rebalancing is enabled. Low-performing validators will be automatically replaced.</span>
            </div>
          </div>
        </div>

        {/* Cross-Chain Deposits */}
        <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6 mb-8">
          <div className="flex justify-between items-center mb-6">
            <div>
              <h2 className="text-2xl font-bold text-white mb-1">Cross-Chain Deposits</h2>
              <p className="text-gray-400 text-sm">Deposits received from other blockchains</p>
            </div>
            <button className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white font-semibold rounded-lg transition text-sm">
              + New Deposit
            </button>
          </div>

          {crossChainDeposits.length > 0 ? (
            <div className="space-y-4">
              {crossChainDeposits.map((deposit, idx) => (
                <div key={idx} className="bg-gray-900/50 rounded-lg p-4 flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="w-10 h-10 bg-gradient-to-br from-purple-600 to-blue-600 rounded-full flex items-center justify-center text-white font-bold text-sm">
                      {deposit.chain[0]}
                    </div>
                    <div>
                      <div className="text-white font-semibold">{deposit.chain}</div>
                      <div className="text-gray-400 text-xs mt-1 font-mono">{deposit.txHash}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-lg font-bold text-white">{deposit.amount} CSPR</div>
                    <div className="text-xs text-gray-400">{deposit.date}</div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <div className="text-4xl mb-2">🌉</div>
              <p>No cross-chain deposits yet</p>
            </div>
          )}
        </div>

        {/* Performance Chart Placeholder */}
        <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
          <h2 className="text-2xl font-bold text-white mb-4">Performance Over Time</h2>
          <div className="bg-gray-900/50 rounded-lg p-8 text-center">
            <div className="text-6xl mb-4">📈</div>
            <p className="text-gray-400">Performance chart coming soon</p>
            <p className="text-sm text-gray-500 mt-2">Track your earnings growth over time</p>
          </div>
        </div>
      </div>
    </div>
  );
}
