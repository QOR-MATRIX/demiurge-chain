// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import Link from 'next/link';

const sdks = [
  {
    icon: '📘',
    title: 'TypeScript SDK',
    description: 'The official TypeScript/JavaScript SDK for Demiurge. Wallet management, RPC calls, transaction signing, and WebSocket subscriptions.',
    href: '/docs/sdk/typescript',
    install: 'npm install @demiurge/sdk',
    features: ['Wallet Generation', 'Transaction Signing', 'RPC Client', 'WebSocket Subscriptions'],
    status: 'Stable',
    statusColor: 'bg-green-500/20 text-green-400',
  },
  {
    icon: '🔐',
    title: 'Wallet Extension',
    description: 'Browser extension SDK for Chrome and Firefox. Secure key storage, dApp integration, and transaction approval.',
    href: '/docs/sdk/wallet-extension',
    install: 'Developer Sideload (see instructions)',
    features: ['Ed25519 Keys', 'BIP39 Mnemonics', 'AES-256-GCM', 'dApp Provider', 'QOR ID Login', 'Sophia AI'],
    status: 'Beta',
    statusColor: 'bg-yellow-500/20 text-yellow-400',
  },
  {
    icon: '🎮',
    title: 'Unreal Engine SDK',
    description: 'Native C++ SDK for Unreal Engine 5. Integrate blockchain functionality directly into your game.',
    href: '/docs/sdk/unreal',
    install: 'Plugins/DemiurgeSDK',
    features: ['Blueprint Support', 'Async Queries', 'Session Keys', 'NFT Integration'],
    status: 'Beta',
    statusColor: 'bg-yellow-500/20 text-yellow-400',
  },
  {
    icon: '🎯',
    title: 'Unity SDK',
    description: 'C# SDK for Unity game engine. Easy-to-use components for blockchain integration in Unity games.',
    href: '/docs/sdk/unity',
    install: 'Unity Package Manager',
    features: ['MonoBehaviour Components', 'Coroutine Support', 'Scriptable Objects', 'Editor Tools'],
    status: 'Coming Soon',
    statusColor: 'bg-gray-500/20 text-gray-400',
  },
];

const tools = [
  {
    icon: '⌨️',
    title: 'CLI',
    description: 'Command-line interface for wallet operations, validator management, and development workflows.',
    href: '/docs/developers/validator-cli',
    install: 'npm install -g @demiurge/cli',
  },
  {
    icon: '🐳',
    title: 'Docker Images',
    description: 'Official Docker images for running Demiurge nodes in containerized environments.',
    href: '/docs/deployment/docker-testnet',
    install: 'docker pull demiurge/node:latest',
  },
];

export default function SDKPage() {
  return (
    <div className="space-y-10">
      {/* Header */}
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <span className="text-4xl">🔧</span>
          <h1 className="text-3xl font-bold">SDK & Tools</h1>
        </div>
        <p className="text-gray-400 text-lg">
          Official SDKs, libraries, and tools for building on Demiurge. Choose your platform and get started.
        </p>
      </div>

      {/* SDK Cards */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">SDKs</h2>
        <div className="grid grid-cols-1 gap-4">
          {sdks.map((sdk) => (
            <Link key={sdk.href} href={sdk.href}>
              <div className="p-6 rounded-xl bg-[var(--bg-surface)] border border-white/10 hover:border-[var(--accent-primary)]/50 transition-all group">
                <div className="flex flex-col md:flex-row md:items-start gap-4">
                  <div className="text-4xl">{sdk.icon}</div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="font-semibold text-xl text-white group-hover:text-[var(--accent-primary)]">
                        {sdk.title}
                      </h3>
                      <span className={`px-2 py-0.5 rounded-full text-xs ${sdk.statusColor}`}>
                        {sdk.status}
                      </span>
                    </div>
                    <p className="text-gray-400 mb-3">{sdk.description}</p>
                    <div className="flex flex-wrap gap-2 mb-3">
                      {sdk.features.map((feature) => (
                        <span
                          key={feature}
                          className="px-2 py-1 rounded-md bg-white/5 text-xs text-gray-400"
                        >
                          {feature}
                        </span>
                      ))}
                    </div>
                    <div className="font-mono text-sm text-[var(--accent-primary)]">
                      {sdk.install}
                    </div>
                  </div>
                  <div className="hidden md:flex items-center text-gray-500">
                    →
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Tools */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Tools</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {tools.map((tool) => (
            <Link key={tool.href} href={tool.href}>
              <div className="h-full p-5 rounded-xl bg-[var(--bg-surface)] border border-white/10 hover:border-[var(--accent-primary)]/50 transition-all group">
                <div className="flex items-start gap-4">
                  <div className="text-2xl">{tool.icon}</div>
                  <div className="flex-1">
                    <h3 className="font-semibold text-white group-hover:text-[var(--accent-primary)]">
                      {tool.title}
                    </h3>
                    <p className="text-sm text-gray-400 mb-2">{tool.description}</p>
                    <div className="font-mono text-xs text-[var(--accent-primary)]">
                      {tool.install}
                    </div>
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Quick Start */}
      <div className="rounded-xl bg-gradient-to-r from-cyan-500/10 to-blue-500/10 border border-white/10 p-6">
        <h3 className="font-semibold text-lg mb-4">Quick Start</h3>
        <div className="space-y-4">
          <div>
            <div className="text-sm text-gray-400 mb-2">1. Install the SDK</div>
            <pre className="bg-black/30 rounded-lg p-3 font-mono text-sm text-[var(--accent-primary)]">
              npm install @demiurge/sdk
            </pre>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-2">2. Initialize client</div>
            <pre className="bg-black/30 rounded-lg p-3 font-mono text-sm text-gray-300 overflow-x-auto">
{`import { DemiurgeClient } from '@demiurge/sdk';

const client = new DemiurgeClient('ws://127.0.0.1:9944');
const balance = await client.getBalance('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');`}
            </pre>
          </div>
        </div>
      </div>

      {/* Comparison Table */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">SDK Comparison</h2>
        <div className="rounded-xl bg-[var(--bg-surface)] border border-white/10 overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="border-b border-white/10">
              <tr className="text-left text-gray-400">
                <th className="px-4 py-3">Feature</th>
                <th className="px-4 py-3">TypeScript</th>
                <th className="px-4 py-3">Extension</th>
                <th className="px-4 py-3">Unreal</th>
                <th className="px-4 py-3">Unity</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              <tr>
                <td className="px-4 py-3 text-white">Wallet Creation</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-gray-500">—</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Transaction Signing</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-gray-500">—</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">WebSocket</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-gray-500">—</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-gray-500">—</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Session Keys</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-gray-500">—</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">NFT Operations</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-green-400">✓</td>
                <td className="px-4 py-3 text-yellow-400">Beta</td>
                <td className="px-4 py-3 text-gray-500">—</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
