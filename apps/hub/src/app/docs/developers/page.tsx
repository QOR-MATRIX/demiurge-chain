// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import Link from 'next/link';

const guides = [
  {
    icon: '📥',
    title: 'Complete Setup Guide',
    description: 'Set up your full development environment from scratch. Includes Node.js, Rust, and all dependencies.',
    href: '/docs/developers/complete-setup',
    badge: 'Essential',
    badgeColor: 'bg-cyan-500/20 text-cyan-400',
  },
  {
    icon: '📡',
    title: 'RPC Reference',
    description: 'Complete JSON-RPC 2.0 API documentation. All methods, parameters, and response formats.',
    href: '/docs/developers/rpc-reference',
    badge: 'Reference',
    badgeColor: 'bg-blue-500/20 text-blue-400',
  },
  {
    icon: '⌨️',
    title: 'Validator CLI',
    description: 'Command-line interface for validator operations: register, stake, unstake, claim rewards.',
    href: '/docs/developers/validator-cli',
    badge: 'Tool',
    badgeColor: 'bg-purple-500/20 text-purple-400',
  },
  {
    icon: '🌐',
    title: 'dApp Quickstart',
    description: 'Build your first Demiurge-integrated web application. Connect wallets, read data, send transactions.',
    href: '/docs/developers/dapp-quickstart',
    badge: 'Tutorial',
    badgeColor: 'bg-green-500/20 text-green-400',
  },
];

const codeExamples = [
  {
    title: 'Connect to Wallet',
    language: 'typescript',
    code: `// Check if Demiurge wallet is installed
if (window.demiurge) {
  const wallet = await window.demiurge.connect();
  console.log('Connected:', wallet.address);
}`,
  },
  {
    title: 'Query Balance',
    language: 'typescript',
    code: `import { DemiurgeClient } from '@demiurge/sdk';

const client = new DemiurgeClient('ws://127.0.0.1:9944');
const balance = await client.getBalance(address);
console.log('Balance:', balance.free);`,
  },
  {
    title: 'Send Transaction',
    language: 'typescript',
    code: `const tx = await wallet.signAndSend({
  method: 'balances_transfer',
  params: { to: recipient, amount: '1000000000000' }
});
console.log('TX Hash:', tx.hash);`,
  },
];

const resources = [
  { icon: '📘', label: 'TypeScript SDK', href: '/docs/sdk/typescript' },
  { icon: '🔐', label: 'Wallet Extension', href: '/docs/sdk/wallet-extension' },
  { icon: '🐳', label: 'Docker Testnet', href: '/docs/deployment/docker-testnet' },
  { icon: '🔍', label: 'Block Explorer', href: '/explorer' },
];

export default function DevelopersPage() {
  return (
    <div className="space-y-10">
      {/* Header */}
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <span className="text-4xl">💻</span>
          <h1 className="text-3xl font-bold">Developers</h1>
        </div>
        <p className="text-gray-400 text-lg">
          Everything you need to build on Demiurge. APIs, SDKs, tools, and best practices.
        </p>
      </div>

      {/* Quick Links */}
      <div className="flex flex-wrap gap-3">
        {resources.map((res) => (
          <Link key={res.href} href={res.href}>
            <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-surface)] border border-white/10 hover:border-[var(--accent-primary)]/50 transition-colors">
              <span>{res.icon}</span>
              <span className="text-sm text-gray-300">{res.label}</span>
            </div>
          </Link>
        ))}
      </div>

      {/* Guides */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Developer Guides</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {guides.map((guide) => (
            <Link key={guide.href} href={guide.href}>
              <div className="h-full p-5 rounded-xl bg-[var(--bg-surface)] border border-white/10 hover:border-[var(--accent-primary)]/50 transition-all group">
                <div className="flex items-start gap-4">
                  <div className="text-2xl">{guide.icon}</div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-white group-hover:text-[var(--accent-primary)]">
                        {guide.title}
                      </h3>
                      <span className={`px-2 py-0.5 rounded-full text-xs ${guide.badgeColor}`}>
                        {guide.badge}
                      </span>
                    </div>
                    <p className="text-sm text-gray-400">{guide.description}</p>
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Code Examples */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Quick Examples</h2>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
          {codeExamples.map((example) => (
            <div key={example.title} className="rounded-xl bg-[var(--bg-surface)] border border-white/10 overflow-hidden">
              <div className="px-4 py-2 border-b border-white/10 flex items-center justify-between">
                <span className="text-sm font-medium">{example.title}</span>
                <span className="text-xs text-gray-500">{example.language}</span>
              </div>
              <pre className="p-4 text-xs overflow-x-auto">
                <code className="text-gray-300">{example.code}</code>
              </pre>
            </div>
          ))}
        </div>
      </div>

      {/* RPC Endpoints */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">RPC Endpoints</h2>
        <div className="rounded-xl bg-[var(--bg-surface)] border border-white/10 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="border-b border-white/10">
              <tr className="text-left text-gray-400">
                <th className="px-4 py-3">Network</th>
                <th className="px-4 py-3">Endpoint</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              <tr>
                <td className="px-4 py-3 text-white">Mainnet</td>
                <td className="px-4 py-3 font-mono text-[var(--accent-primary)]">ws://127.0.0.1:9944</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-green-500/10 text-green-400 text-xs">Live</span>
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Testnet</td>
                <td className="px-4 py-3 font-mono text-[var(--accent-primary)]">ws://127.0.0.1:9944</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-green-500/10 text-green-400 text-xs">Live</span>
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Local Devnet</td>
                <td className="px-4 py-3 font-mono text-gray-400">ws://localhost:9944</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-gray-500/10 text-gray-400 text-xs">Local</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* SDK Installation */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Quick Install</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="rounded-xl bg-[var(--bg-surface)] border border-white/10 p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-sm font-medium text-white">npm / yarn</span>
            </div>
            <pre className="text-sm font-mono text-[var(--accent-primary)]">
              npm install @demiurge/sdk
            </pre>
          </div>
          <div className="rounded-xl bg-[var(--bg-surface)] border border-white/10 p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-sm font-medium text-white">CLI</span>
            </div>
            <pre className="text-sm font-mono text-[var(--accent-primary)]">
              npm install -g @demiurge/cli
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}
