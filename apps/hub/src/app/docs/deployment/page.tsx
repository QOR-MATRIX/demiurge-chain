// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import Link from 'next/link';

const guides = [
  {
    icon: '🚀',
    title: 'Production Deployment',
    description: 'Deploy a Demiurge node to production. Server setup, systemd services, nginx, and monitoring.',
    href: '/docs/deployment/production',
    badge: 'Essential',
    badgeColor: 'bg-green-500/20 text-green-400',
  },
  {
    icon: '🐳',
    title: 'Docker Testnet',
    description: 'Deploy a multi-node testnet using Docker Compose. Perfect for local development and testing.',
    href: '/docs/deployment/docker-testnet',
    badge: 'Popular',
    badgeColor: 'bg-blue-500/20 text-blue-400',
  },
  {
    icon: '⚙️',
    title: 'Configuration Reference',
    description: 'Complete reference for all node configuration options. TOML files, CLI arguments, and environment variables.',
    href: '/docs/deployment/configuration',
    badge: 'Reference',
    badgeColor: 'bg-purple-500/20 text-purple-400',
  },
  {
    icon: '📝',
    title: 'Environment Variables',
    description: 'All environment variables for node, frontend, and services. Includes production recommendations.',
    href: '/docs/deployment/environment',
    badge: 'Reference',
    badgeColor: 'bg-purple-500/20 text-purple-400',
  },
];

const quickCommands = [
  {
    title: 'Docker Testnet',
    commands: [
      'cd docker',
      'docker-compose up -d',
      '# 4 nodes running at localhost:9944-9947',
    ],
  },
  {
    title: 'Production Binary',
    commands: [
      'cargo build --release',
      './target/release/demiurge-node \\',
      '  --validator --name "my-validator"',
    ],
  },
];

export default function DeploymentPage() {
  return (
    <div className="space-y-10">
      {/* Header */}
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <span className="text-4xl">🚢</span>
          <h1 className="text-3xl font-bold">Deployment</h1>
        </div>
        <p className="text-gray-400 text-lg">
          Deploy Demiurge nodes, testnets, and production infrastructure. 
          From local development to mainnet validators.
        </p>
      </div>

      {/* Quick Deploy */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {quickCommands.map((section) => (
          <div key={section.title} className="rounded-xl bg-[var(--bg-surface)] border border-white/10 overflow-hidden">
            <div className="px-4 py-2 border-b border-white/10 flex items-center gap-2">
              <span className="text-green-400">●</span>
              <span className="font-medium text-sm">{section.title}</span>
            </div>
            <pre className="p-4 text-sm overflow-x-auto">
              <code className="text-gray-300">
                {section.commands.map((cmd, i) => (
                  <div key={i} className={cmd.startsWith('#') ? 'text-gray-500' : 'text-[var(--accent-primary)]'}>
                    {cmd}
                  </div>
                ))}
              </code>
            </pre>
          </div>
        ))}
      </div>

      {/* Deployment Guides */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Deployment Guides</h2>
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

      {/* Deployment Options */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Deployment Options</h2>
        <div className="rounded-xl bg-[var(--bg-surface)] border border-white/10 overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="border-b border-white/10">
              <tr className="text-left text-gray-400">
                <th className="px-4 py-3">Method</th>
                <th className="px-4 py-3">Best For</th>
                <th className="px-4 py-3">Difficulty</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              <tr>
                <td className="px-4 py-3 text-white">Docker Compose</td>
                <td className="px-4 py-3 text-gray-400">Local development, testing</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-green-500/10 text-green-400 text-xs">Easy</span>
                </td>
                <td className="px-4 py-3 text-[var(--accent-primary)]">Recommended</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Binary + Systemd</td>
                <td className="px-4 py-3 text-gray-400">Production validators</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-yellow-500/10 text-yellow-400 text-xs">Medium</span>
                </td>
                <td className="px-4 py-3 text-[var(--accent-primary)]">Production</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Kubernetes</td>
                <td className="px-4 py-3 text-gray-400">Large-scale deployments</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-red-500/10 text-red-400 text-xs">Advanced</span>
                </td>
                <td className="px-4 py-3 text-gray-500">Coming Soon</td>
              </tr>
              <tr>
                <td className="px-4 py-3 text-white">Managed (Cloud)</td>
                <td className="px-4 py-3 text-gray-400">Enterprise deployments</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded-full bg-green-500/10 text-green-400 text-xs">Easy</span>
                </td>
                <td className="px-4 py-3 text-gray-500">Coming Soon</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* Network Endpoints */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Network Endpoints</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="p-4 rounded-xl bg-[var(--bg-surface)] border border-white/10">
            <div className="flex items-center gap-2 mb-2">
              <span className="w-2 h-2 rounded-full bg-green-400"></span>
              <span className="font-medium text-white">Mainnet</span>
            </div>
            <code className="text-sm text-[var(--accent-primary)]">ws://127.0.0.1:9944</code>
          </div>
          <div className="p-4 rounded-xl bg-[var(--bg-surface)] border border-white/10">
            <div className="flex items-center gap-2 mb-2">
              <span className="w-2 h-2 rounded-full bg-yellow-400"></span>
              <span className="font-medium text-white">Testnet</span>
            </div>
            <code className="text-sm text-[var(--accent-primary)]">ws://127.0.0.1:9944</code>
          </div>
          <div className="p-4 rounded-xl bg-[var(--bg-surface)] border border-white/10">
            <div className="flex items-center gap-2 mb-2">
              <span className="w-2 h-2 rounded-full bg-gray-400"></span>
              <span className="font-medium text-white">Local</span>
            </div>
            <code className="text-sm text-gray-400">ws://localhost:9944</code>
          </div>
        </div>
      </div>

      {/* Monitoring */}
      <div className="p-6 rounded-xl bg-gradient-to-r from-blue-500/10 to-purple-500/10 border border-white/10">
        <div className="flex items-start gap-4">
          <span className="text-3xl">📊</span>
          <div className="flex-1">
            <h3 className="font-semibold text-lg mb-2">Monitoring & Observability</h3>
            <p className="text-gray-400 mb-4">
              Demiurge nodes expose Prometheus metrics on port 9615 by default. 
              Use Grafana dashboards for visualization.
            </p>
            <div className="flex gap-3">
              <Link href="/docs/deployment/monitoring">
                <button className="px-4 py-2 rounded-lg bg-white/5 hover:bg-white/10 text-sm transition-colors">
                  Monitoring Guide →
                </button>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
