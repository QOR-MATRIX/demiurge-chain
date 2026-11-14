// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import { GlassPanel } from "@components/GlassPanel";
import { AnimatedBackground } from "@components/AnimatedBackground";
import { useAuth } from "@lib/contexts/AuthContext";
import Link from "next/link";

interface Transaction {
  id: string;
  from: string;
  to: string;
  amount: string;
  type: "sent" | "received" | "staking" | "reward";
  timestamp: Date;
  status: "confirmed" | "pending";
  hash: string;
}

export const WalletSystem: React.FC = () => {
  const { user, isAuthenticated } = useAuth();
  const [balance] = useState("2,847.52 CGT");
  const [totalValue] = useState("$284,752.00 USD");
  const [transactions, setTransactions] = useState<Transaction[]>([
    {
      id: "1",
      from: "You",
      to: "0x1a2b3c4d5e6f...",
      amount: "100.00 CGT",
      type: "sent",
      timestamp: new Date(Date.now() - 3600000),
      status: "confirmed",
      hash: "0x1a2b3c4d5e6f7g8h9i0j",
    },
    {
      id: "2",
      from: "0x2b3c4d5e6f7g...",
      to: "You",
      amount: "50.00 CGT",
      type: "received",
      timestamp: new Date(Date.now() - 7200000),
      status: "confirmed",
      hash: "0x2b3c4d5e6f7g8h9i0j1k",
    },
    {
      id: "3",
      from: "You",
      to: "Staking Pool",
      amount: "500.00 CGT",
      type: "staking",
      timestamp: new Date(Date.now() - 10800000),
      status: "confirmed",
      hash: "0x3c4d5e6f7g8h9i0j1k2l",
    },
    {
      id: "4",
      from: "Mining Rewards",
      to: "You",
      amount: "2.80 CGT",
      type: "reward",
      timestamp: new Date(Date.now() - 14400000),
      status: "confirmed",
      hash: "0x4d5e6f7g8h9i0j1k2l3m",
    },
  ]);
  const [transferForm, setTransferForm] = useState({ to: "", amount: "" });
  const [isTransferring, setIsTransferring] = useState(false);

  const handleTransfer = async () => {
    if (!transferForm.to || !transferForm.amount) return;
    setIsTransferring(true);

    try {
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const newTransaction: Transaction = {
        id: `${transactions.length + 1}`,
        from: "You",
        to: transferForm.to,
        amount: `${parseFloat(transferForm.amount).toFixed(2)} CGT`,
        type: "sent",
        timestamp: new Date(),
        status: "pending",
        hash: `0x${Math.random().toString(16).slice(2)}`,
      };

      setTransactions((prev) => [newTransaction, ...prev]);
      setTransferForm({ to: "", amount: "" });
    } finally {
      setIsTransferring(false);
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

  const getTransactionIcon = (type: Transaction["type"]) => {
    switch (type) {
      case "sent":
        return "📤";
      case "received":
        return "📥";
      case "staking":
        return "🔒";
      case "reward":
        return "🎁";
    }
  };

  const getTransactionColor = (type: Transaction["type"]) => {
    switch (type) {
      case "sent":
        return "text-red-400";
      case "received":
        return "text-green-400";
      case "staking":
        return "text-primary-400";
      case "reward":
        return "text-cyan-400";
    }
  };

  return (
    <div className="relative w-full min-h-screen bg-navy-900 overflow-hidden">
      <AnimatedBackground intensity="medium" />

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
                💼 Wallet
              </span>
            </Link>

            <Link
              href="/dashboard"
              className="px-4 py-2 text-gray-300 hover:text-white transition-colors"
            >
              ← Back
            </Link>
          </div>
        </motion.header>

        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
          {/* Balance section */}
          <motion.section
            className="mb-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h1 className="text-4xl font-bold gradient-text mb-8">Your Wallet</h1>

            <GlassPanel className="p-8 mb-8" blur="xl" border="medium">
              <div className="mb-8">
                <p className="text-gray-400 text-sm mb-2">Total Balance</p>
                <p className="text-5xl font-bold text-white mb-2">{balance}</p>
                <p className="text-lg text-cyan-400">{totalValue}</p>
              </div>

              <div className="grid grid-cols-3 gap-4 pt-8 border-t border-white/10">
                {[
                  {
                    label: "Available",
                    value: "2,347.52 CGT",
                    percent: "82%",
                  },
                  {
                    label: "Staked",
                    value: "500.00 CGT",
                    percent: "17.6%",
                  },
                  {
                    label: "Reserved",
                    value: "0 CGT",
                    percent: "0.4%",
                  },
                ].map((item, idx) => (
                  <div key={idx}>
                    <p className="text-gray-400 text-xs mb-1">{item.label}</p>
                    <p className="text-sm font-semibold text-white">{item.value}</p>
                    <p className="text-xs text-primary-400 mt-1">{item.percent}</p>
                  </div>
                ))}
              </div>
            </GlassPanel>

            {/* Action buttons */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {[
                { label: "Send", icon: "📤", color: "from-blue-500 to-blue-600" },
                { label: "Receive", icon: "📥", color: "from-green-500 to-green-600" },
                { label: "Stake", icon: "🔒", color: "from-primary-500 to-primary-600" },
              ].map((action, idx) => (
                <motion.div key={idx} variants={itemVariants}>
                  <button className="w-full p-4 bg-gradient-to-br from-white/10 to-white/5 hover:from-white/20 hover:to-white/10 border border-white/10 rounded-lg transition-all group">
                    <span className="text-2xl block mb-2">{action.icon}</span>
                    <span className="text-white font-semibold">{action.label}</span>
                  </button>
                </motion.div>
              ))}
            </div>
          </motion.section>

          {/* Transfer form */}
          <motion.section
            className="mb-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">Send Tokens</h2>

            <GlassPanel className="p-8" blur="lg" border="medium">
              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Recipient Address
                  </label>
                  <input
                    type="text"
                    value={transferForm.to}
                    onChange={(e) =>
                      setTransferForm((prev) => ({ ...prev, to: e.target.value }))
                    }
                    placeholder="Enter recipient address or QOR ID"
                    className="w-full px-4 py-3 bg-navy-800/50 border border-primary-500/30 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500/50"
                    disabled={isTransferring}
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Amount (CGT)
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="number"
                      value={transferForm.amount}
                      onChange={(e) =>
                        setTransferForm((prev) => ({ ...prev, amount: e.target.value }))
                      }
                      placeholder="0.00"
                      className="flex-1 px-4 py-3 bg-navy-800/50 border border-primary-500/30 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500/50"
                      disabled={isTransferring}
                    />
                    <button
                      onClick={() =>
                        setTransferForm((prev) => ({ ...prev, amount: "2347.52" }))
                      }
                      className="px-4 py-3 bg-white/10 hover:bg-white/20 rounded-lg text-white text-sm font-semibold transition"
                    >
                      Max
                    </button>
                  </div>
                </div>

                <button
                  onClick={handleTransfer}
                  disabled={isTransferring || !transferForm.to || !transferForm.amount}
                  className="w-full py-3 bg-gradient-to-r from-primary-600 to-primary-500 hover:from-primary-500 hover:to-primary-400 disabled:opacity-50 text-white font-semibold rounded-lg transition"
                >
                  {isTransferring ? "Sending..." : "Send Tokens"}
                </button>

                <p className="text-xs text-gray-500 text-center">
                  Network fee: ~0.01 CGT (Energy-based)
                </p>
              </div>
            </GlassPanel>
          </motion.section>

          {/* Transaction history */}
          <motion.section
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">Transaction History</h2>

            <GlassPanel className="overflow-hidden" blur="md" border="medium">
              <div className="divide-y divide-white/10">
                {transactions.length === 0 ? (
                  <div className="p-8 text-center text-gray-400">
                    No transactions yet. Send or receive CGT to get started.
                  </div>
                ) : (
                  transactions.map((tx) => (
                    <motion.div
                      key={tx.id}
                      className="p-6 hover:bg-white/5 transition-colors"
                      variants={itemVariants}
                    >
                      <div className="flex items-center justify-between gap-4">
                        {/* Left side */}
                        <div className="flex items-center gap-4 flex-1">
                          <div className="text-3xl">{getTransactionIcon(tx.type)}</div>
                          <div className="flex-1 min-w-0">
                            <p className="text-white font-medium capitalize mb-1">
                              {tx.type === "sent"
                                ? "Sent to"
                                : tx.type === "received"
                                  ? "Received from"
                                  : tx.type === "staking"
                                    ? "Staking"
                                    : "Mining reward"}
                            </p>
                            <div className="flex items-center gap-2">
                              <p className="text-gray-400 text-sm truncate">
                                {tx.type === "reward"
                                  ? "Mining Pool"
                                  : tx.type === "staking"
                                    ? "Staking Contract"
                                    : tx.type === "sent"
                                      ? tx.to
                                      : tx.from}
                              </p>
                              <span
                                className={`px-2 py-1 rounded text-xs font-semibold ${
                                  tx.status === "confirmed"
                                    ? "bg-green-500/20 text-green-400"
                                    : "bg-yellow-500/20 text-yellow-400"
                                }`}
                              >
                                {tx.status === "confirmed" ? "✓ Confirmed" : "⏳ Pending"}
                              </span>
                            </div>
                          </div>
                        </div>

                        {/* Right side */}
                        <div className="text-right flex-shrink-0">
                          <p
                            className={`text-lg font-bold mb-1 ${getTransactionColor(tx.type)}`}
                          >
                            {tx.type === "sent" || tx.type === "staking"
                              ? "-"
                              : "+"}
                            {tx.amount}
                          </p>
                          <a
                            href={`https://qorsync.dev/tx/${tx.hash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-xs text-primary-400 hover:text-primary-300"
                          >
                            {tx.hash.slice(0, 10)}...
                          </a>
                        </div>
                      </div>

                      <p className="text-xs text-gray-500 mt-3">
                        {tx.timestamp.toLocaleString()}
                      </p>
                    </motion.div>
                  ))
                )}
              </div>
            </GlassPanel>
          </motion.section>
        </main>
      </div>

      {/* Floating accent orbs */}
      <motion.div
        className="absolute top-1/4 -right-32 w-96 h-96 bg-cyan-500 rounded-full filter blur-3xl opacity-10 pointer-events-none"
        animate={{
          y: [0, -40, 0],
          x: [0, 30, 0],
        }}
        transition={{ duration: 15, repeat: Infinity }}
      />
    </div>
  );
};

export default WalletSystem;
