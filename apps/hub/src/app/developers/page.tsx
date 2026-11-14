// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import { useState } from 'react';
import Link from 'next/link';

type TabType = 'overview' | 'quickstart' | 'sdks' | 'api' | 'examples';

interface CodeExample {
  id: string;
  title: string;
  description: string;
  language: 'typescript' | 'python' | 'rust' | 'bash';
  code: string;
}

const CODE_EXAMPLES: CodeExample[] = [
  {
    id: 'connect',
    title: 'Connect to Demiurge',
    description: 'Initialize the SDK and connect to the network',
    language: 'typescript',
    code: `import { DemiurgeClient } from '@demiurge/sdk';

const client = new DemiurgeClient({
  rpcUrl: 'http://127.0.0.1:9944'
});

// Get current block
const blockNumber = await client.getBlockNumber();
console.log('Connected! Block:', blockNumber);`,
  },
  {
    id: 'balance',
    title: 'Check Balance',
    description: 'Query CGT balance for an address',
    language: 'typescript',
    code: `import { DemiurgeClient } from '@demiurge/sdk';

const client = new DemiurgeClient({
  rpcUrl: 'http://127.0.0.1:9944'
});

const balance = await client.getBalance(
  '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'
);

console.log('Balance:', balance, 'CGT');`,
  },
  {
    id: 'nft',
    title: 'Fetch DRC-369 NFT',
    description: 'Get an evolving NFT with dynamic state',
    language: 'typescript',
    code: `import { DRC369Client } from '@demiurge/drc369-sdk';

const drc369 = new DRC369Client({
  rpcUrl: 'http://127.0.0.1:9944'
});

const nft = await drc369.getToken('nft-001');

console.log('Name:', nft.name);
console.log('Level:', nft.dynamicState.level);
console.log('XP:', nft.dynamicState.xp);`,
  },
  {
    id: 'identity',
    title: 'Resolve QOR ID',
    description: 'Look up a user by their human-readable handle',
    language: 'typescript',
    code: `import { qorAuth } from '@demiurge/qor-sdk';

// Login with QOR ID
const { user, token } = await qorAuth.login(
  'alice#1234',
  'password'
);

console.log('Logged in as:', user.qor_id);
console.log('Address:', user.on_chain?.address);`,
  },
  {
    id: 'agent',
    title: 'Create AI Agent',
    description: 'Deploy an autonomous AI agent',
    language: 'typescript',
    code: `import { createAgent } from '@demiurge/agent-foundry';

const agent = await createAgent({
  name: 'MarketAnalyzer',
  llm: {
    provider: 'openai',
    apiKey: process.env.OPENAI_API_KEY,
    model: 'gpt-4-turbo',
  },
  autonomy: 'bounded',
  capabilities: ['analysis', 'reporting'],
  spendingLimit: '100 CGT',
  controller: 'alice#1234',
});

console.log('Agent DID:', agent.did.did);

// Run inference
const result = await agent.think(
  'Analyze current market trends'
);
console.log(result.output);`,
  },
  {
    id: 'curl',
    title: 'Direct RPC Call',
    description: 'Query the blockchain via JSON-RPC',
    language: 'bash',
    code: `curl -X POST http://127.0.0.1:9944 \\
  -H "Content-Type: application/json" \\
  -d '{
    "jsonrpc": "2.0",
    "method": "chain_getBlockNumber",
    "params": [],
    "id": 1
  }'`,
  },
];

const SDKS = [
  {
    name: '@demiurge/sdk',
    description: 'Core protocol SDK for blockchain interaction',
    install: 'npm install @demiurge/sdk',
    docs: '/docs/developers/sdk',
    features: ['RPC client', 'Transaction building', 'Wallet management', 'CVP utilities'],
  },
  {
    name: '@demiurge/qor-sdk',
    description: 'QOR Identity SDK for authentication and profiles',
    install: 'npm install @demiurge/qor-sdk',
    docs: '/docs/developers/qor-sdk',
    features: ['Login/Registration', 'Profile management', 'Session handling', 'Leveling system'],
  },
  {
    name: '@demiurge/drc369-sdk',
    description: 'DRC-369 NFT SDK with React hooks',
    install: 'npm install @demiurge/drc369-sdk',
    docs: '/docs/developers/drc369-sdk',
    features: ['NFT queries', 'Dynamic state', 'React hooks', 'Real-time subscriptions'],
  },
  {
    name: '@demiurge/agent-foundry',
    description: 'AI Agent SDK for autonomous agents',
    install: 'npm install @demiurge/agent-foundry',
    docs: '/docs/developers/agent-foundry',
    features: ['Agent creation', 'Multi-LLM support', 'Memory system', 'Tool integration'],
  },
];

export default function DevelopersPage() {
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [selectedExample, setSelectedExample] = useState<string>('connect');
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const copyCode = (code: string, id: string) => {
    navigator.clipboard.writeText(code);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2000);
  };

  const getLanguageColor = (lang: string) => {
    switch (lang) {
      case 'typescript': return 'text-blue-400';
      case 'python': return 'text-yellow-400';
      case 'rust': return 'text-orange-400';
      case 'bash': return 'text-green-400';
      default: return 'text-gray-400';
    }
  };

  return (
    <main className="min-h-screen">
      {/* Header */}
      <div className="bg-gradient-to-r from-neon-cyan/10 to-neon-purple/10 border-b border-gray-800">
        <div className="max-w-6xl mx-auto px-4 py-8">
          <div className="flex items-center gap-3 mb-2">
            <span className="text-4xl">💻</span>
            <h1 className="text-4xl md:text-5xl font-grunge bg-gradient-to-r from-neon-cyan to-neon-purple bg-clip-text text-transparent">
              Developer Portal
            </h1>
          </div>
          <p className="text-gray-400 text-lg">Build on Demiurge-Cloud - Guides, SDKs, and API Reference</p>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="bg-blockchain-dark border-b border-gray-800 sticky top-16 z-40">
        <div className="max-w-6xl mx-auto px-4">
          <div className="flex gap-1 overflow-x-auto">
            {[
              { id: 'overview', label: '📖 Overview' },
              { id: 'quickstart', label: '🚀 Quick Start' },
              { id: 'sdks', label: '📦 SDKs' },
              { id: 'api', label: '🔌 API Reference' },
              { id: 'examples', label: '💡 Examples' },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as TabType)}
                className={`px-4 py-3 text-sm font-medium whitespace-nowrap transition-colors ${
                  activeTab === tab.id
                    ? 'text-neon-cyan border-b-2 border-neon-cyan'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="max-w-6xl mx-auto px-4 py-8">
        {/* Overview Tab */}
        {activeTab === 'overview' && (
          <div className="space-y-8">
            <div className="glass-panel rounded-xl p-6">
              <h2 className="text-2xl font-grunge text-white mb-4">What is Demiurge?</h2>
              <p className="text-gray-300 leading-relaxed mb-4">
                Demiurge is a next-generation blockchain protocol designed for gaming and AI applications.
                It features sub-2-second finality, feeless transactions via the Energy system, 
                evolving NFTs (DRC-369), and AI agents as first-class citizens.
              </p>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
                <div className="text-center p-4 bg-blockchain-light/30 rounded-lg">
                  <div className="text-3xl font-grunge text-neon-cyan">2s</div>
                  <div className="text-xs text-gray-400">Finality</div>
                </div>
                <div className="text-center p-4 bg-blockchain-light/30 rounded-lg">
                  <div className="text-3xl font-grunge text-green-400">0</div>
                  <div className="text-xs text-gray-400">Energy Fees</div>
                </div>
                <div className="text-center p-4 bg-blockchain-light/30 rounded-lg">
                  <div className="text-3xl font-grunge text-neon-purple">1000+</div>
                  <div className="text-xs text-gray-400">TPS</div>
                </div>
                <div className="text-center p-4 bg-blockchain-light/30 rounded-lg">
                  <div className="text-3xl font-grunge text-yellow-400">4</div>
                  <div className="text-xs text-gray-400">SDKs</div>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="glass-panel rounded-xl p-6">
                <h3 className="text-xl font-grunge text-neon-cyan mb-4">🎮 For Game Developers</h3>
                <ul className="space-y-2 text-gray-300">
                  <li className="flex items-start gap-2">
                    <span className="text-neon-cyan">•</span>
                    <span>DRC-369 evolving NFTs with physics metadata</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-cyan">•</span>
                    <span>Session keys for seamless gameplay</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-cyan">•</span>
                    <span>Unreal Engine 5 SDK</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-cyan">•</span>
                    <span>Real-time state synchronization</span>
                  </li>
                </ul>
              </div>

              <div className="glass-panel rounded-xl p-6">
                <h3 className="text-xl font-grunge text-neon-purple mb-4">🤖 For AI Developers</h3>
                <ul className="space-y-2 text-gray-300">
                  <li className="flex items-start gap-2">
                    <span className="text-neon-purple">•</span>
                    <span>Autonomous AI agents with DIDs</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-purple">•</span>
                    <span>Verifiable compute proofs (VCP)</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-purple">•</span>
                    <span>On-chain memory for agents</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-neon-purple">•</span>
                    <span>Bounty marketplace for AI tasks</span>
                  </li>
                </ul>
              </div>
            </div>
          </div>
        )}

        {/* Quick Start Tab */}
        {activeTab === 'quickstart' && (
          <div className="space-y-8">
            <div className="glass-panel rounded-xl p-6">
              <h2 className="text-2xl font-grunge text-white mb-4">Quick Start</h2>
              <p className="text-gray-400 mb-6">Get started with Demiurge in under 5 minutes.</p>

              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-grunge-alt text-neon-cyan mb-3">1. Install the SDK</h3>
                  <div className="bg-black/50 rounded-lg p-4 font-mono text-sm">
                    <code className="text-green-400">npm install @demiurge/sdk</code>
                  </div>
                </div>

                <div>
                  <h3 className="text-lg font-grunge-alt text-neon-cyan mb-3">2. Connect to the Network</h3>
                  <div className="bg-black/50 rounded-lg p-4 font-mono text-sm overflow-x-auto">
                    <pre className="text-gray-300">{`import { DemiurgeClient } from '@demiurge/sdk';

const client = new DemiurgeClient({
  rpcUrl: 'http://127.0.0.1:9944'
});

const block = await client.getBlockNumber();
console.log('Connected at block:', block);`}</pre>
                  </div>
                </div>

                <div>
                  <h3 className="text-lg font-grunge-alt text-neon-cyan mb-3">3. Query Data</h3>
                  <div className="bg-black/50 rounded-lg p-4 font-mono text-sm overflow-x-auto">
                    <pre className="text-gray-300">{`// Get balance
const balance = await client.getBalance(address);

// Get NFT
const nft = await client.getNFT(tokenId);

// Get validators
const validators = await client.getValidators();`}</pre>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* SDKs Tab */}
        {activeTab === 'sdks' && (
          <div className="space-y-6">
            {SDKS.map((sdk) => (
              <div key={sdk.name} className="glass-panel rounded-xl p-6">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <h3 className="text-xl font-grunge text-white">{sdk.name}</h3>
                    <p className="text-gray-400 text-sm mt-1">{sdk.description}</p>
                  </div>
                  <Link 
                    href={sdk.docs}
                    className="glass-panel px-4 py-2 rounded-lg text-sm text-neon-cyan hover:bg-neon-cyan/10 transition-colors"
                  >
                    Docs →
                  </Link>
                </div>
                
                <div className="mt-4 bg-black/50 rounded-lg p-3 font-mono text-sm flex items-center justify-between">
                  <code className="text-green-400">{sdk.install}</code>
                  <button
                    onClick={() => copyCode(sdk.install, sdk.name)}
                    className="text-gray-400 hover:text-white transition-colors"
                  >
                    {copiedCode === sdk.name ? '✓' : '📋'}
                  </button>
                </div>

                <div className="mt-4 flex flex-wrap gap-2">
                  {sdk.features.map((feature) => (
                    <span 
                      key={feature}
                      className="px-2 py-1 rounded bg-blockchain-light/50 text-gray-300 text-xs"
                    >
                      {feature}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* API Reference Tab */}
        {activeTab === 'api' && (
          <div className="space-y-6">
            <div className="glass-panel rounded-xl p-6">
              <h2 className="text-2xl font-grunge text-white mb-4">RPC API</h2>
              <p className="text-gray-400 mb-4">
                The Demiurge RPC API uses JSON-RPC 2.0 over HTTP POST.
              </p>
              
              <div className="bg-black/50 rounded-lg p-4 mb-6">
                <div className="text-sm text-gray-400 mb-2">Endpoint</div>
                <code className="text-neon-cyan">http://127.0.0.1:9944</code>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {[
                  { category: 'System', methods: ['system_health', 'system_nodeInfo', 'system_peers'] },
                  { category: 'Chain', methods: ['chain_getBlockNumber', 'chain_getBlock', 'chain_getLatestBlock'] },
                  { category: 'Balances', methods: ['balances_getBalance', 'balances_transfer', 'balances_getHistory'] },
                  { category: 'Energy', methods: ['energy_getEnergy', 'energy_estimateCost'] },
                  { category: 'Identity', methods: ['identity_resolve', 'identity_getDid', 'identity_getHandle'] },
                  { category: 'NFT', methods: ['drc369_getToken', 'drc369_getTokensByOwner', 'drc369_mint'] },
                  { category: 'Staking', methods: ['staking_getValidators', 'staking_getPool', 'staking_getCurrentEra'] },
                  { category: 'Agents', methods: ['agent_getAgent', 'agent_listBounties', 'agent_getBounty'] },
                ].map((group) => (
                  <div key={group.category} className="bg-blockchain-light/30 rounded-lg p-4">
                    <h4 className="font-grunge-alt text-neon-cyan mb-2">{group.category}</h4>
                    <ul className="space-y-1">
                      {group.methods.map((method) => (
                        <li key={method} className="text-gray-300 text-sm font-mono">
                          {method}
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>

              <div className="mt-6">
                <Link 
                  href="/docs/api"
                  className="inline-flex items-center gap-2 text-neon-cyan hover:underline"
                >
                  View Full API Documentation →
                </Link>
              </div>
            </div>
          </div>
        )}

        {/* Examples Tab */}
        {activeTab === 'examples' && (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Example List */}
            <div className="space-y-2">
              {CODE_EXAMPLES.map((example) => (
                <button
                  key={example.id}
                  onClick={() => setSelectedExample(example.id)}
                  className={`w-full text-left p-3 rounded-lg transition-colors ${
                    selectedExample === example.id
                      ? 'bg-neon-cyan/10 border border-neon-cyan/30'
                      : 'bg-blockchain-light/30 hover:bg-blockchain-light/50'
                  }`}
                >
                  <div className="font-grunge-alt text-white">{example.title}</div>
                  <div className="text-xs text-gray-400">{example.description}</div>
                </button>
              ))}
            </div>

            {/* Code Display */}
            <div className="lg:col-span-2">
              {CODE_EXAMPLES.filter(e => e.id === selectedExample).map((example) => (
                <div key={example.id} className="glass-panel rounded-xl overflow-hidden">
                  <div className="flex items-center justify-between p-4 border-b border-gray-800">
                    <div>
                      <h3 className="font-grunge text-white">{example.title}</h3>
                      <span className={`text-xs ${getLanguageColor(example.language)}`}>
                        {example.language}
                      </span>
                    </div>
                    <button
                      onClick={() => copyCode(example.code, example.id)}
                      className="glass-panel px-3 py-1 rounded text-sm text-gray-400 hover:text-white transition-colors"
                    >
                      {copiedCode === example.id ? '✓ Copied' : '📋 Copy'}
                    </button>
                  </div>
                  <div className="p-4 bg-black/50 overflow-x-auto">
                    <pre className="text-sm font-mono text-gray-300 whitespace-pre">
                      {example.code}
                    </pre>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
