// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import { GlassPanel } from "@components/GlassPanel";
import { AnimatedBackground } from "@components/AnimatedBackground";
import { useAuth } from "@lib/contexts/AuthContext";
import Link from "next/link";

type TabType = "api" | "sdk" | "guides" | "tools";

interface APIEndpoint {
  name: string;
  method: string;
  path: string;
  description: string;
  example: string;
}

export const DeveloperHub: React.FC = () => {
  const { user, isAuthenticated } = useAuth();
  const [activeTab, setActiveTab] = useState<TabType>("api");
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const apiEndpoints: APIEndpoint[] = [
    {
      name: "Get Account Balance",
      method: "GET",
      path: "/api/v1/account/balance",
      description: "Retrieve the CGT token balance for a QOR ID",
      example: `fetch('http://127.0.0.1:8080/api/v1/account/balance?qor_id=user@qor', {
  headers: { 'Authorization': 'Bearer YOUR_TOKEN' }
}).then(r => r.json())`,
    },
    {
      name: "Submit Transaction",
      method: "POST",
      path: "/api/v1/transactions/submit",
      description: "Submit a signed transaction to the blockchain",
      example: `fetch('http://127.0.0.1:8080/api/v1/transactions/submit', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ transaction: signedTx })
}).then(r => r.json())`,
    },
    {
      name: "Query NFTs",
      method: "GET",
      path: "/api/v1/nft/query",
      description: "Query NFTs for a given address or collection",
      example: `fetch('http://127.0.0.1:8080/api/v1/nft/query?address=0x...&collection=DRC-369', {
  headers: { 'Authorization': 'Bearer YOUR_TOKEN' }
}).then(r => r.json())`,
    },
    {
      name: "Game Session Key",
      method: "POST",
      path: "/api/v1/games/session-key",
      description: "Create a temporary session key for game interactions",
      example: `fetch('http://127.0.0.1:8080/api/v1/games/session-key', {
  method: 'POST',
  headers: { 'Authorization': 'Bearer YOUR_TOKEN' },
  body: JSON.stringify({ game_id: 'ethereal-realms', ttl: 3600 })
}).then(r => r.json())`,
    },
  ];

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

  const handleCopyCode = (code: string, id: string) => {
    navigator.clipboard.writeText(code);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2000);
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
                💻 Developer Hub
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
          {/* Hero section */}
          <motion.section
            className="mb-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h1 className="text-4xl font-bold gradient-text mb-4">Developer Resources</h1>
            <p className="text-gray-400 text-lg mb-8">
              Build on the Demiurge blockchain with our comprehensive APIs, SDKs, and documentation
            </p>

            {/* Quick stats */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
              {[
                { label: "API Endpoints", value: "50+", icon: "📡" },
                { label: "SDK Languages", value: "5", icon: "🔧" },
                { label: "Documentation", value: "100%", icon: "📚" },
                { label: "Support", value: "24/7", icon: "🆘" },
              ].map((stat, idx) => (
                <motion.div key={idx} variants={itemVariants}>
                  <GlassPanel className="p-6" blur="md" border="medium">
                    <p className="text-gray-400 text-sm mb-2">{stat.label}</p>
                    <div className="flex items-center justify-between">
                      <p className="text-3xl font-bold text-white">{stat.value}</p>
                      <span className="text-3xl">{stat.icon}</span>
                    </div>
                  </GlassPanel>
                </motion.div>
              ))}
            </div>
          </motion.section>

          {/* Tab navigation */}
          <motion.section className="mb-12" variants={containerVariants} initial="hidden" animate="visible">
            <div className="flex gap-3 border-b border-white/10 overflow-x-auto pb-4">
              {[
                { id: "api", label: "REST API", icon: "📡" },
                { id: "sdk", label: "SDKs", icon: "📦" },
                { id: "guides", label: "Guides", icon: "📖" },
                { id: "tools", label: "Tools", icon: "🛠️" },
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as TabType)}
                  className={`px-4 py-3 font-semibold transition-all whitespace-nowrap ${
                    activeTab === tab.id
                      ? "text-primary-400 border-b-2 border-primary-400"
                      : "text-gray-400 hover:text-white"
                  }`}
                >
                  {tab.icon} {tab.label}
                </button>
              ))}
            </div>
          </motion.section>

          {/* Tab content */}
          <motion.section
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            {/* API Tab */}
            {activeTab === "api" && (
              <div className="space-y-6">
                <div>
                  <h2 className="text-2xl font-bold text-white mb-6">REST API Documentation</h2>

                  <div className="space-y-4">
                    {apiEndpoints.map((endpoint, idx) => (
                      <motion.div key={idx} variants={itemVariants}>
                        <GlassPanel className="overflow-hidden" blur="md" border="medium">
                          <div className="p-6">
                            {/* Endpoint header */}
                            <div className="flex items-start justify-between mb-4">
                              <div>
                                <h3 className="text-lg font-semibold text-white mb-2">
                                  {endpoint.name}
                                </h3>
                                <p className="text-gray-400 text-sm">{endpoint.description}</p>
                              </div>
                              <div className="flex items-center gap-2">
                                <span
                                  className={`px-3 py-1 rounded font-mono text-sm font-bold ${
                                    endpoint.method === "GET"
                                      ? "bg-blue-500/20 text-blue-400"
                                      : "bg-green-500/20 text-green-400"
                                  }`}
                                >
                                  {endpoint.method}
                                </span>
                              </div>
                            </div>

                            {/* Endpoint path */}
                            <div className="bg-navy-800/50 rounded px-4 py-2 mb-4 border border-white/10">
                              <code className="text-cyan-400 text-sm font-mono">
                                {endpoint.path}
                              </code>
                            </div>

                            {/* Code example */}
                            <div>
                              <p className="text-gray-400 text-xs mb-2">Example Request:</p>
                              <div className="relative bg-navy-900/50 rounded p-4 border border-white/10">
                                <pre className="text-gray-300 text-xs font-mono overflow-x-auto">
                                  {endpoint.example}
                                </pre>
                                <button
                                  onClick={() =>
                                    handleCopyCode(endpoint.example, `api-${idx}`)
                                  }
                                  className="absolute top-2 right-2 px-3 py-1 bg-primary-600/50 hover:bg-primary-600 text-white text-xs rounded transition"
                                >
                                  {copiedCode === `api-${idx}` ? "✓ Copied!" : "Copy"}
                                </button>
                              </div>
                            </div>
                          </div>
                        </GlassPanel>
                      </motion.div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* SDK Tab */}
            {activeTab === "sdk" && (
              <div>
                <h2 className="text-2xl font-bold text-white mb-6">SDK Libraries</h2>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                  {[
                    {
                      name: "TypeScript/JavaScript",
                      description: "Official SDK for Node.js and browsers",
                      package: "@demiurge/sdk",
                      version: "2.1.0",
                    },
                    {
                      name: "Python",
                      description: "Python3 bindings for the Demiurge API",
                      package: "demiurge-sdk",
                      version: "2.1.0",
                    },
                    {
                      name: "Rust",
                      description: "Native Rust library with zero-copy performance",
                      package: "demiurge-rs",
                      version: "2.1.0",
                    },
                    {
                      name: "Go",
                      description: "Go SDK with async/await support",
                      package: "github.com/demiurge/sdk-go",
                      version: "2.1.0",
                    },
                  ].map((sdk, idx) => (
                    <motion.div key={idx} variants={itemVariants}>
                      <GlassPanel className="p-6 h-full" blur="md" border="medium">
                        <h3 className="text-lg font-semibold text-white mb-2">
                          {sdk.name}
                        </h3>
                        <p className="text-gray-400 text-sm mb-4">{sdk.description}</p>

                        <div className="space-y-3">
                          <div>
                            <p className="text-gray-500 text-xs mb-1">Package</p>
                            <code className="text-cyan-400 text-sm font-mono">
                              {sdk.package}
                            </code>
                          </div>
                          <div>
                            <p className="text-gray-500 text-xs mb-1">Version</p>
                            <p className="text-white font-semibold">{sdk.version}</p>
                          </div>

                          <button className="w-full mt-4 py-2 bg-gradient-to-r from-primary-600 to-primary-500 hover:from-primary-500 hover:to-primary-400 text-white font-semibold rounded transition">
                            View Docs →
                          </button>
                        </div>
                      </GlassPanel>
                    </motion.div>
                  ))}
                </div>
              </div>
            )}

            {/* Guides Tab */}
            {activeTab === "guides" && (
              <div>
                <h2 className="text-2xl font-bold text-white mb-6">Developer Guides</h2>

                <div className="space-y-4">
                  {[
                    {
                      title: "Getting Started with Demiurge",
                      description: "Learn the basics of building on the Demiurge blockchain",
                      time: "10 min",
                      level: "Beginner",
                    },
                    {
                      title: "Building Your First Game",
                      description: "Create a simple game integrated with Demiurge blockchain",
                      time: "45 min",
                      level: "Intermediate",
                    },
                    {
                      title: "NFT Integration with DRC-369",
                      description: "Mint and manage stateful NFTs in your application",
                      time: "30 min",
                      level: "Intermediate",
                    },
                    {
                      title: "Advanced Session Key Management",
                      description: "Secure your game interactions with temporary session keys",
                      time: "25 min",
                      level: "Advanced",
                    },
                    {
                      title: "Deploying Smart Modules",
                      description: "Create and deploy custom blockchain modules",
                      time: "60 min",
                      level: "Advanced",
                    },
                  ].map((guide, idx) => (
                    <motion.div key={idx} variants={itemVariants}>
                      <GlassPanel
                        className="p-6 hover:bg-white/10 transition-colors cursor-pointer"
                        blur="md"
                        border="medium"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <h3 className="text-lg font-semibold text-white mb-2">
                              {guide.title}
                            </h3>
                            <p className="text-gray-400 text-sm mb-4">
                              {guide.description}
                            </p>

                            <div className="flex gap-4 text-xs text-gray-500">
                              <span>⏱️ {guide.time}</span>
                              <span
                                className={`px-2 py-1 rounded ${
                                  guide.level === "Beginner"
                                    ? "bg-green-500/20 text-green-400"
                                    : guide.level === "Intermediate"
                                      ? "bg-yellow-500/20 text-yellow-400"
                                      : "bg-red-500/20 text-red-400"
                                }`}
                              >
                                {guide.level}
                              </span>
                            </div>
                          </div>
                          <span className="text-2xl">→</span>
                        </div>
                      </GlassPanel>
                    </motion.div>
                  ))}
                </div>
              </div>
            )}

            {/* Tools Tab */}
            {activeTab === "tools" && (
              <div>
                <h2 className="text-2xl font-bold text-white mb-6">Developer Tools</h2>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {[
                    {
                      name: "CLI Tool",
                      description: "Command-line interface for blockchain interactions",
                      icon: "⌨️",
                    },
                    {
                      name: "Network Explorer",
                      description: "Visual explorer for blocks, transactions, and contracts",
                      icon: "🔍",
                    },
                    {
                      name: "Testing Framework",
                      description: "Unit and integration testing suite for your modules",
                      icon: "🧪",
                    },
                    {
                      name: "Wallet Manager",
                      description: "Multi-signature wallet with hardware support",
                      icon: "💳",
                    },
                    {
                      name: "Gas Estimator",
                      description: "Estimate transaction costs and optimize performance",
                      icon: "📊",
                    },
                    {
                      name: "Debugger",
                      description: "Step through module execution and inspect state",
                      icon: "🐛",
                    },
                  ].map((tool, idx) => (
                    <motion.div key={idx} variants={itemVariants}>
                      <GlassPanel className="p-6" blur="md" border="medium">
                        <div className="text-4xl mb-4">{tool.icon}</div>
                        <h3 className="text-lg font-semibold text-white mb-2">
                          {tool.name}
                        </h3>
                        <p className="text-gray-400 text-sm mb-4">{tool.description}</p>

                        <button className="w-full py-2 bg-white/10 hover:bg-white/20 text-white font-semibold rounded transition">
                          Learn More
                        </button>
                      </GlassPanel>
                    </motion.div>
                  ))}
                </div>
              </div>
            )}
          </motion.section>

          {/* Support section */}
          <motion.section
            className="mt-16"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <h2 className="text-2xl font-bold text-white mb-6">Need Help?</h2>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              {[
                {
                  title: "Documentation",
                  description: "Comprehensive docs and API reference",
                  link: "https://qorsync.dev",
                },
                {
                  title: "Community Discord",
                  description: "Join thousands of developers and builders",
                  link: "https://discord.gg/demiurge",
                },
                {
                  title: "Status Page",
                  description: "Real-time network and API status",
                  link: "https://qorsync.dev",
                },
              ].map((resource, idx) => (
                <motion.div key={idx} variants={itemVariants}>
                  <GlassPanel className="p-6" blur="md" border="medium">
                    <h3 className="text-lg font-semibold text-white mb-2">
                      {resource.title}
                    </h3>
                    <p className="text-gray-400 text-sm mb-4">{resource.description}</p>

                    <a
                      href={resource.link}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary-400 hover:text-primary-300 font-semibold text-sm"
                    >
                      Visit →
                    </a>
                  </GlassPanel>
                </motion.div>
              ))}
            </div>
          </motion.section>
        </main>
      </div>

      {/* Floating accent orbs */}
      <motion.div
        className="absolute bottom-1/3 -left-40 w-96 h-96 bg-primary-500 rounded-full filter blur-3xl opacity-10 pointer-events-none"
        animate={{
          y: [0, 40, 0],
          x: [0, -30, 0],
        }}
        transition={{ duration: 20, repeat: Infinity }}
      />
    </div>
  );
};

export default DeveloperHub;
