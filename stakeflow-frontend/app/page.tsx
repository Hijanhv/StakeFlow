'use client';

import { useState } from 'react';
import Image from 'next/image';

export default function Home() {
  const [amount, setAmount] = useState('');
  const [activeTab, setActiveTab] = useState<'deposit' | 'withdraw'>('deposit');
  const [isConnected, setIsConnected] = useState(false);

  // Mock data
  const totalStaked = '1,245,000';
  const userBalance = '150.5';
  const apy = '12.5';
  const totalValueLocked = '2.5M';

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-black to-red-900">
      {/* Navigation */}
      <nav className="border-b border-gray-800 bg-black/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-3">
              <Image src="/logo.png" alt="StakeFlow" width={60} height={60} className="rounded-lg" />
              <span className="text-2xl font-bold text-white">StakeFlow</span>
            </div>
            <button
              onClick={() => setIsConnected(!isConnected)}
              className="px-6 py-2 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-all transform hover:scale-105"
            >
              {isConnected ? 'Disconnect Wallet' : 'Connect Wallet'}
            </button>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="text-center mb-12">
          <h1 className="text-5xl font-bold text-white mb-4">
            Liquid Staking & DeFi Yield Optimization
          </h1>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            One-click yield optimization that stakes your CSPR, generates liquid tokens, and automatically compounds returns across DeFi protocols
          </p>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-12">
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Total Value Locked</div>
            <div className="text-3xl font-bold text-white">${totalValueLocked} CSPR</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Current APY</div>
            <div className="text-3xl font-bold text-green-400">{apy}%</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Total Staked</div>
            <div className="text-3xl font-bold text-white">{totalStaked} CSPR</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-gray-400 text-sm mb-2">Your Balance</div>
            <div className="text-3xl font-bold text-blue-400">{userBalance} CSPR</div>
          </div>
        </div>

        {/* Main Action Card */}
        <div className="max-w-2xl mx-auto">
          <div className="bg-gray-800/70 backdrop-blur-sm border border-gray-700 rounded-2xl p-8 shadow-2xl">
            {/* Tabs */}
            <div className="flex space-x-2 mb-6 bg-gray-900/50 rounded-lg p-1">
              <button
                onClick={() => setActiveTab('deposit')}
                className={`flex-1 py-3 px-6 rounded-lg font-semibold transition-all ${
                  activeTab === 'deposit'
                    ? 'bg-red-600 text-white'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                Deposit
              </button>
              <button
                onClick={() => setActiveTab('withdraw')}
                className={`flex-1 py-3 px-6 rounded-lg font-semibold transition-all ${
                  activeTab === 'withdraw'
                    ? 'bg-red-600 text-white'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                Withdraw
              </button>
            </div>

            {/* Input Section */}
            <div className="mb-6">
              <label className="block text-gray-300 text-sm font-medium mb-2">
                {activeTab === 'deposit' ? 'Deposit Amount' : 'Withdraw Amount'}
              </label>
              <div className="relative">
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                  className="w-full bg-gray-900/50 border border-gray-600 rounded-lg px-4 py-4 text-white text-xl focus:outline-none focus:ring-2 focus:ring-red-500"
                  disabled={!isConnected}
                />
                <div className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-400 font-semibold">
                  CSPR
                </div>
              </div>
              <div className="flex justify-between mt-2 text-sm text-gray-400">
                <span>Min: 10 CSPR</span>
                <button
                  onClick={() => setAmount(userBalance)}
                  className="text-red-400 hover:text-red-300 font-medium"
                  disabled={!isConnected}
                >
                  Use Max
                </button>
              </div>
            </div>

            {/* Expected Returns */}
            {amount && parseFloat(amount) > 0 && (
              <div className="bg-gray-900/50 rounded-lg p-4 mb-6 space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">You will receive</span>
                  <span className="text-white font-semibold">{amount} sCSPR</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-400">Expected yearly return</span>
                  <span className="text-green-400 font-semibold">
                    ~{(parseFloat(amount) * parseFloat(apy)) / 100} CSPR
                  </span>
                </div>
              </div>
            )}

            {/* Action Button */}
            <button
              disabled={!isConnected || !amount || parseFloat(amount) < 10}
              className="w-full py-4 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white font-bold text-lg rounded-lg transition-all transform hover:scale-[1.02] disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none shadow-lg"
            >
              {!isConnected
                ? 'Connect Wallet First'
                : activeTab === 'deposit'
                ? 'Stake CSPR'
                : 'Unstake sCSPR'}
            </button>

            {/* Info Box */}
            {!isConnected && (
              <div className="mt-6 bg-blue-500/10 border border-blue-500/30 rounded-lg p-4 text-center">
                <p className="text-blue-300 text-sm">
                  🔗 Connect your Casper Wallet to start earning yield on your CSPR
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Features Section */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-16">
          <div className="bg-gray-800/30 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-4xl mb-4">🚀</div>
            <h3 className="text-xl font-bold text-white mb-2">Instant Liquidity</h3>
            <p className="text-gray-400">
              Receive sCSPR tokens immediately and use them across DeFi while earning staking rewards
            </p>
          </div>
          <div className="bg-gray-800/30 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-4xl mb-4">💰</div>
            <h3 className="text-xl font-bold text-white mb-2">Auto-Compounding</h3>
            <p className="text-gray-400">
              Rewards are automatically reinvested to maximize your returns without any action needed
            </p>
          </div>
          <div className="bg-gray-800/30 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
            <div className="text-4xl mb-4">🔒</div>
            <h3 className="text-xl font-bold text-white mb-2">Secure & Tested</h3>
            <p className="text-gray-400">
              Built with Rust on Casper Network with comprehensive security testing and audits
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-16 text-center text-gray-500 text-sm">
          <p>Built for Casper Network Hackathon 2026 | Tracks: Liquid Staking + DeFi</p>
          <p className="mt-2">
            <a
              href="https://github.com/Hijanhv/StakeFlow"
              target="_blank"
              rel="noopener noreferrer"
              className="text-red-400 hover:text-red-300"
            >
              View on GitHub →
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}
