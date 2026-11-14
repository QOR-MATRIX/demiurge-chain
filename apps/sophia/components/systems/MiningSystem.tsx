// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
"use client";

import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { GlassPanel } from "@components/GlassPanel";
import { AnimatedBackground } from "@components/AnimatedBackground";
import { useAuth } from "@lib/contexts/AuthContext";
import Link from "next/link";

interface ValidatorStats {
  activeValidators: number;
  totalStake: string;
  blockReward: string;
  era: number;
  estimatedReward: string;
}

interface MiningReward {
  timestamp: Date;
  amount: string;
  txHash: string;
}

export const MiningSystem: React.FC = () => {
  const { user, isAuthenticated } = useAuth();
  const [stats, setStats] = useState<ValidatorStats>({
    activeValidators: 342,
    totalStake: "15,240,500 CGT",
    blockReward: "5.2 CGT",
    era: 1247,
    estimatedReward: "2.8 CGT",
  });
  const [rewards, setRewards] = useState<MiningReward[]>([
    {
      timestamp: new Date(Date.now() - 3600000),
      amount: "2.8 CGT",
      txHash: "0x1a2b3c4d5e6f7g8h9i0j",
    },
    {
      timestamp: new Date(Date.now() - 7200000),
      amount: "2.8 CGT",
      txHash: "0x2b3c4d5e6f7g8h9i0j1k",
    },
    {
      timestamp: new Date(Date.now() - 10800000),
      amount: "2.8 CGT",
      txHash: "0x3c4d5e6f7g8h9i0j1k2l",
    },
  ]);
  const [isStaking, setIsStaking] = useState(false);
  const [stakingAmount, setStakingAmount] = useState("");

  const handleStake = async () => {
    if (!stakingAmount) return;
    setIsStaking(true);

    try {
      // Simulate staking transaction
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Add new reward
      const newReward: MiningReward = {
        timestamp: new Date(),
        amount: `${(Math.random() * 5 + 2).toFixed(2)} CGT`,
        txHash: `0x${Math.random().toString(16).slice(2)}`,
      };

      setRewards((prev) => [newReward, ...prev]);
      setStakingAmount("");
    } finally {
      setIsStaking(false);
    }
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
        delayChildren: 0.2,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.6 },
    },
  };

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-navy-900 flex items-center justify-center">
        <Link href="/auth" className="text-primary-400 hover:text-primary-300">
          Please log in to continue
        </Link>
      </div>
    );
  }

  return (
    <div className="relative w-full min-h-screen bg-navy-900 overflow-hidden">
      {/* Animated background */}
      <AnimatedBackground intensity="medium" />

      {/* Content */}
      <div className="relative z-20 w-full">
        {/* Header */}
        <motion.header
          className="backdrop-blur-md border-b border-white/5 sticky top-0 z-40"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between">
            <Link href="/dashboard" className="flex items-center gap-3 group">
              <span className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-primary-400 to-cyan-400 group-hover:scale-110 transition-transform">
                ◇ Mining
              </span>
            </Link>

            <div className="text-gray-400 text-sm">
              Era {stats.era} • {stats.activeValidators} Active Validators
            </div>

            <Link
              href="/dashboard"
              className="px-4 py-2 text-gray-300 hover:text-white transition-colors"
            >
              ← Back
            </Link>
          </div>
        </motion.header>

        {/* Main content */}
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
          {/* Overview section */}
          <motion.section
            className="mb-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h1 className="text-4xl font-bold gradient-text mb-8">Network Validation</h1>

            {/* Key metrics grid */}
            <motion.div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
              {[
                {
                  label: "Total Stake",
                  value: stats.totalStake,
                  icon: "💎",
                  color: "from-primary-500 to-primary-600",
                },
                {
                  label: "Block Reward",
                  value: stats.blockReward,
                  icon: "💰",
                  color: "from-cyan-400 to-cyan-500",
                },
                {
                  label: "Your Estimated Reward",
                  value: stats.estimatedReward,
                  icon: "📈",
                  color: "from-green-400 to-emerald-500",
                },
                {
                  label: "Network Health",
                  value: "Excellent",
                  icon: "❤️",
                  color: "from-red-400 to-pink-500",
                },
              ].map((metric, idx) => (
                <motion.div key={idx} variants={itemVariants}>
                  <GlassPanel
                    className="p-6 h-full"
                    blur="md"
                    border="medium"
                  >
                    <div className="flex items-start justify-between mb-4">
                      <div className={`text-4xl w-12 h-12 rounded-lg bg-gradient-to-br ${metric.color} flex items-center justify-center`}>
                        {metric.icon}
                      </div>
                    </div>
                    <p className="text-gray-400 text-sm mb-1">{metric.label}</p>
                    <p className="text-2xl font-bold text-white">{metric.value}</p>
                  </GlassPanel>
                </motion.div>
              ))}
            </motion.div>
          </motion.section>

          {/* Staking section */}
          <motion.section
            className="mb-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">Stake CGT Tokens</h2>

            <GlassPanel className="p-8" blur="lg" border="medium">
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-3">
                    Amount to Stake
                  </label>
                  <div className="flex gap-3">
                    <input
                      type="number"
                      value={stakingAmount}
                      onChange={(e) => setStakingAmount(e.target.value)}
                      placeholder="Enter CGT amount"
                      className="flex-1 px-4 py-3 bg-navy-800/50 border border-primary-500/30 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500/50"
                      disabled={isStaking}
                    />
                    <button
                      onClick={handleStake}
                      disabled={isStaking || !stakingAmount}
                      className="px-6 py-3 bg-gradient-to-r from-primary-600 to-primary-500 hover:from-primary-500 hover:to-primary-400 disabled:opacity-50 text-white font-semibold rounded-lg transition"
                    >
                      {isStaking ? "Staking..." : "Stake"}
                    </button>
                  </div>
                  <p className="text-xs text-gray-500 mt-2">
                    Minimum stake: 32 CGT • Rewards: ~5-7% APY
                  </p>
                </div>

                <div className="pt-4 border-t border-white/10">
                  <div className="grid grid-cols-2 gap-4">
                    <GlassPanel className="p-4" blur="sm" variant="light">
                      <p className="text-gray-400 text-xs mb-1">Your Stake</p>
                      <p className="text-xl font-bold text-white">128.5 CGT</p>
                    </GlassPanel>
                    <GlassPanel className="p-4" blur="sm" variant="light">
                      <p className="text-gray-400 text-xs mb-1">Accumulated Rewards</p>
                      <p className="text-xl font-bold text-white">8.42 CGT</p>
                    </GlassPanel>
                  </div>
                </div>
              </div>
            </GlassPanel>
          </motion.section>

          {/* Rewards history */}
          <motion.section
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">Recent Rewards</h2>

            <GlassPanel className="overflow-hidden" blur="md" border="medium">
              <div className="divide-y divide-white/10">
                {rewards.length === 0 ? (
                  <div className="p-8 text-center text-gray-400">
                    No rewards yet. Start staking to earn rewards.
                  </div>
                ) : (
                  rewards.map((reward, idx) => (
                    <motion.div
                      key={idx}
                      className="p-6 hover:bg-white/5 transition-colors"
                      variants={itemVariants}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <p className="text-white font-medium mb-1">
                            Block Validation Reward
                          </p>
                          <p className="text-xs text-gray-500">
                            {reward.timestamp.toLocaleString()}
                          </p>
                        </div>

                        <div className="text-right">
                          <p className="text-lg font-bold gradient-text mb-1">
                            +{reward.amount}
                          </p>
                          <a
                            href={`https://qorsync.dev/tx/${reward.txHash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-xs text-primary-400 hover:text-primary-300 truncate max-w-32"
                          >
                            {reward.txHash.slice(0, 10)}...
                          </a>
                        </div>
                      </div>
                    </motion.div>
                  ))
                )}
              </div>
            </GlassPanel>
          </motion.section>

          {/* Info section */}
          <motion.section
            className="mt-16 mb-8"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">How Mining Works</h2>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              {[
                {
                  title: "Stake CGT",
                  description: "Lock your tokens as validator stake (minimum 32 CGT)",
                  icon: "🔒",
                },
                {
                  title: "Validate Blocks",
                  description: "Help secure the network by participating in consensus",
                  icon: "✓",
                },
                {
                  title: "Earn Rewards",
                  description: "Receive staking rewards proportional to your stake",
                  icon: "🎁",
                },
              ].map((item, idx) => (
                <motion.div key={idx} variants={itemVariants}>
                  <GlassPanel className="p-6 h-full" blur="md" border="subtle">
                    <div className="text-4xl mb-4">{item.icon}</div>
                    <h3 className="text-lg font-semibold text-white mb-2">
                      {item.title}
                    </h3>
                    <p className="text-gray-400 text-sm">{item.description}</p>
                  </GlassPanel>
                </motion.div>
              ))}
            </div>
          </motion.section>
        </main>
      </div>

      {/* Floating accent orbs */}
      <motion.div
        className="absolute top-1/3 right-10 w-96 h-96 bg-primary-500 rounded-full filter blur-3xl opacity-10 pointer-events-none"
        animate={{
          y: [0, -30, 0],
          x: [0, 20, 0],
        }}
        transition={{ duration: 12, repeat: Infinity }}
      />
    </div>
  );
};

export default MiningSystem;
