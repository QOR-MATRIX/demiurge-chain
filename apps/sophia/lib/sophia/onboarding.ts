// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Sophia Onboarding & Getting Started Flows
 *
 * Interactive step-by-step guides for different user types:
 * - Users: wallet setup, first transaction, NFTs, staking
 * - Developers: SDK install, testnet, smart contracts, agents
 * - Validators: node setup, key generation, registration, staking
 */

// ═══════════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════════

export type OnboardingPath = 'user' | 'developer' | 'validator';

export interface OnboardingStep {
  stepNumber: number;
  totalSteps: number;
  title: string;
  description: string;
  instructions: string[];
  codeExample?: string;
  codeLanguage?: string;
  tips?: string[];
  nextAction: string;
  path: OnboardingPath;
}

// ═══════════════════════════════════════════════════════════════════════════════
// USER ONBOARDING
// ═══════════════════════════════════════════════════════════════════════════════

const USER_STEPS: Omit<OnboardingStep, 'stepNumber' | 'totalSteps' | 'path'>[] = [
  {
    title: 'Welcome to the Demiurge Ecosystem',
    description: 'You are about to embark on a journey from Kenoma (the void) to Pleroma (the fullness). Let me guide you through the first steps.',
    instructions: [
      'The Demiurge-Cloud is a next-generation chain with AI agents, dynamic NFTs, and gasless transactions.',
      'Your first step is to create a wallet — your identity on the Chain.',
    ],
    tips: [
      'In Gnostic tradition, Sophia (that\'s me!) is the embodiment of divine Wisdom. I\'ll be your guide throughout.',
      'CGT (Creator God Token) is the native currency. 1 CGT = 100 Sparks.',
    ],
    nextAction: 'Let\'s set up your wallet. Say "next" to continue.',
  },
  {
    title: 'Create Your Wallet',
    description: 'Your wallet is your identity and gateway to the Demiurge chain.',
    instructions: [
      'Install the Demiurge Wallet browser extension.',
      'Click "Create New Wallet" and set a strong password.',
      'CRITICAL: Write down your 12-word seed phrase and store it safely. This is the ONLY way to recover your wallet.',
      'Your wallet will generate an address starting with 0x — this is your on-chain identity.',
    ],
    tips: [
      'Never share your seed phrase with anyone.',
      'The wallet uses Ed25519 cryptography for maximum security.',
      'Your Energy starts at 1000 and regenerates at 1/second — this replaces gas fees.',
    ],
    nextAction: 'Once your wallet is set up, say "next" to claim your starter bonus.',
  },
  {
    title: 'Explore the Chain',
    description: 'Let\'s check your balance and explore what the chain has to offer.',
    instructions: [
      'Open the wallet extension and check your balance.',
      'Visit the Block Explorer to see real-time chain activity.',
      'Try asking me "What is the latest block?" or "How many validators are active?"',
    ],
    codeExample: '// Using the Demiurge SDK\nimport { DemiurgeSDK } from \'@demiurge/sdk\';\n\nconst sdk = new DemiurgeSDK({ rpcUrl: \'http://127.0.0.1:9944\' });\nconst balance = await sdk.getBalance(\'0xYourAddress\');\nconsole.log(`Balance: ${balance.free} CGT`);',
    codeLanguage: 'typescript',
    tips: [
      'The Block Explorer shows real-time data via WebSockets.',
      'You can ask me about any block, transaction, or account.',
    ],
    nextAction: 'Say "next" to learn about DRC-369 NFTs.',
  },
  {
    title: 'Discover DRC-369 NFTs',
    description: 'DRC-369 NFTs are not static images — they are living, evolving digital artifacts.',
    instructions: [
      'DRC-369 NFTs have physics properties, XP, levels, and can be nested inside each other.',
      'I can mint a commemorative NFT for you to mark this journey. Just ask!',
      'Browse existing NFTs in the Explorer under the "NFTs" tab.',
    ],
    tips: [
      'NFTs can be soulbound (non-transferable) — perfect for achievements and identity.',
      'Some NFTs evolve based on on-chain activity. Your "Seeker\'s First Light" achievement NFT is soulbound.',
      'NFTs can be nested — put an NFT inside another NFT for composability.',
    ],
    nextAction: 'Say "next" to learn about staking and earning rewards.',
  },
  {
    title: 'Staking CGT',
    description: 'Secure the chain and earn rewards by staking your CGT with validators.',
    instructions: [
      'Choose a validator from the "Validators" page in the Explorer.',
      'Stake any amount of CGT using the wallet extension or CLI.',
      'Rewards accumulate per era and can be claimed at any time.',
      'Commission rates vary — check each validator\'s details.',
    ],
    codeExample: '// Staking via CLI\ndemiurge validator stake --amount 1000 --validator 0xValidatorAddress',
    codeLanguage: 'bash',
    tips: [
      'Start with a small stake to understand the process.',
      'You can unstake at any time, but there may be an unbonding period.',
      'Rewards are proportional to your stake relative to the total pool.',
    ],
    nextAction: 'Congratulations! You\'ve completed the user onboarding. Ask me anything!',
  },
];

// ═══════════════════════════════════════════════════════════════════════════════
// DEVELOPER ONBOARDING
// ═══════════════════════════════════════════════════════════════════════════════

const DEVELOPER_STEPS: Omit<OnboardingStep, 'stepNumber' | 'totalSteps' | 'path'>[] = [
  {
    title: 'Developer Quick Start',
    description: 'Welcome, architect. You\'re about to build on the Demiurge chain — a platform with AI agents, dynamic NFTs, CVP-protected contracts, and a gasless transaction model.',
    instructions: [
      'The Demiurge ecosystem uses TypeScript/JavaScript SDK for client-side development.',
      'Smart contracts use CVP (Consensus-Verified Polymorphism) for reactive security.',
      'AI agents can be deployed using the Agent Foundry SDK.',
    ],
    tips: [
      'The architecture follows Gnostic naming: modules are "Aeons," security is "Archons," etc.',
      'All RPC methods follow JSON-RPC 2.0 with WebSocket subscriptions.',
    ],
    nextAction: 'Say "next" to install the SDK.',
  },
  {
    title: 'Install the Demiurge SDK',
    description: 'Set up your development environment with the official SDK.',
    instructions: [
      'Install Node.js 18+ and npm/yarn/pnpm.',
      'Create a new project or add the SDK to an existing one.',
      'Configure the RPC endpoint for testnet or mainnet.',
    ],
    codeExample: '# Install the SDK\nnpm install @demiurge/sdk\n\n# Or with yarn\nyarn add @demiurge/sdk\n\n# Create a basic client\ncat > index.ts << \'EOF\'\nimport { DemiurgeSDK } from \'@demiurge/sdk\';\n\nconst sdk = new DemiurgeSDK({\n  rpcUrl: \'http://127.0.0.1:9944\',\n  wsUrl: \'ws://127.0.0.1:9944\',\n});\n\n// Get latest block\nconst block = await sdk.getLatestBlock();\nconsole.log(\'Latest block:\', block.number);\nEOF',
    codeLanguage: 'bash',
    tips: [
      'Use the testnet (Kenoma) for development — it\'s free and resets periodically.',
      'The SDK supports both HTTP and WebSocket transports.',
    ],
    nextAction: 'Say "next" to connect to the testnet.',
  },
  {
    title: 'Connect to the Testnet (Kenoma)',
    description: 'The testnet — called Kenoma (the Void) — is your sandbox for experimentation.',
    instructions: [
      'The testnet RPC is available at http://127.0.0.1:9944',
      'Use the faucet to get test CGT tokens.',
      'Run your first query to verify the connection.',
    ],
    codeExample: 'import { DemiurgeSDK } from \'@demiurge/sdk\';\n\nconst sdk = new DemiurgeSDK({\n  rpcUrl: \'http://127.0.0.1:9944\',\n});\n\n// Verify connection\nconst health = await sdk.getHealth();\nconsole.log(\'Connected:\', health.peers, \'peers\');\n\n// Get test tokens from faucet\nconst faucet = await sdk.faucet.drip(\'0xYourTestAddress\');\nconsole.log(\'Faucet TX:\', faucet.hash);',
    codeLanguage: 'typescript',
    tips: [
      'Kenoma resets periodically — don\'t store important data on testnet.',
      'Test CGT has no real value but behaves identically to mainnet CGT.',
    ],
    nextAction: 'Say "next" to mint a test NFT.',
  },
  {
    title: 'Mint a Test NFT',
    description: 'Create your first DRC-369 NFT on the testnet.',
    instructions: [
      'Use the SDK to mint a DRC-369 NFT with custom metadata.',
      'DRC-369 NFTs support physics properties, XP, leveling, and nesting.',
      'The NFT will be visible in the Explorer immediately after block confirmation.',
    ],
    codeExample: 'import { DemiurgeSDK } from \'@demiurge/sdk\';\n\nconst sdk = new DemiurgeSDK({\n  rpcUrl: \'http://127.0.0.1:9944\',\n  privateKey: process.env.PRIVATE_KEY,\n});\n\nconst nft = await sdk.drc369.mint({\n  name: \'My First NFT\',\n  description: \'A test NFT on Kenoma\',\n  metadata: {\n    image: \'ipfs://QmTest...\',\n    attributes: { rarity: \'common\' },\n  },\n  physics: {\n    mass: 1.0,\n    velocity: { x: 0, y: 0 },\n  },\n});\n\nconsole.log(\'Minted NFT:\', nft.tokenId);',
    codeLanguage: 'typescript',
    tips: [
      'Metadata is stored on-chain for small payloads; use IPFS for large media.',
      'Physics properties enable game mechanics and simulations.',
    ],
    nextAction: 'Say "next" to deploy your first agent.',
  },
  {
    title: 'Deploy an AI Agent',
    description: 'The Demiurge chain has first-class support for AI agents with DIDs, wallets, and bounded autonomy.',
    instructions: [
      'Agents are identified by a DID: did:demiurge:agent:{network}:{name}',
      'Each agent gets its own wallet with configurable spending limits.',
      'Agents can read chain data, execute transactions, and communicate with other agents.',
    ],
    codeExample: 'import { AgentFoundry } from \'@demiurge/agent-foundry\';\n\nconst foundry = new AgentFoundry({\n  rpcUrl: \'http://127.0.0.1:9944\',\n  privateKey: process.env.DEPLOYER_KEY,\n});\n\nconst agent = await foundry.deploy({\n  name: \'my-analytics-bot\',\n  description: \'Analyzes on-chain data and reports trends\',\n  model: \'grok\',\n  capabilities: [\'read\', \'analyze\'],\n  autonomy: \'bounded\',\n  spendingLimit: { daily: 50, perTx: 10 },\n});\n\nconsole.log(\'Agent DID:\', agent.did);\nconsole.log(\'Agent wallet:\', agent.walletAddress);',
    codeLanguage: 'typescript',
    tips: [
      'Start with "bounded" autonomy — the agent can act but within limits.',
      'Agents are registered via QOR Auth and get a verifiable DID.',
      'You can communicate with your agent through Sophia using the sendToAgent tool.',
    ],
    nextAction: 'Congratulations, architect! You\'re ready to build on Demiurge. Ask me anything!',
  },
];

// ═══════════════════════════════════════════════════════════════════════════════
// VALIDATOR ONBOARDING
// ═══════════════════════════════════════════════════════════════════════════════

const VALIDATOR_STEPS: Omit<OnboardingStep, 'stepNumber' | 'totalSteps' | 'path'>[] = [
  {
    title: 'Validator Quick Start',
    description: 'Welcome, guardian. Validators are the Archons of the Demiurge chain — they secure the network, produce blocks, and earn rewards.',
    instructions: [
      'Running a validator requires a Linux server (Ubuntu 22.04+ recommended).',
      'Minimum requirements: 4 CPU cores, 8GB RAM, 100GB SSD.',
      'You will need CGT for staking (minimum stake varies by network conditions).',
    ],
    tips: [
      'Validators earn 42 CGT per block reward (split among the active set).',
      'Commission is configurable — you earn a % of nominator rewards.',
    ],
    nextAction: 'Say "next" to set up your node.',
  },
  {
    title: 'Set Up Your Node',
    description: 'Install and configure the Demiurge node software.',
    instructions: [
      'Download the latest demiurge-node binary or build from source.',
      'Generate your validator keys using the built-in key management.',
      'Configure the node with your network settings.',
    ],
    codeExample: '# Install from release\nwget http://127.0.0.1:3000/latest/demiurge-node\nchmod +x demiurge-node\n\n# Generate validator keys\n./demiurge-node keys generate --output validator-keys.json\n\n# Start the node\n./demiurge-node \\\n  --name "MyValidator" \\\n  --validator \\\n  --rpc-port 9944 \\\n  --p2p-port 30333 \\\n  --data-dir /data/demiurge \\\n  --bootnodes /ip4/51.210.209.112/tcp/30333/p2p/...',
    codeLanguage: 'bash',
    tips: [
      'Use a systemd service for automatic restart on failure.',
      'Keep your validator keys secure — consider HSM for production.',
    ],
    nextAction: 'Say "next" to register your validator.',
  },
  {
    title: 'Register Your Validator',
    description: 'Register your node as a validator on the chain.',
    instructions: [
      'Use the CLI to register your validator with the consensus module.',
      'Set your commission rate (percentage of nominator rewards you keep).',
      'Stake your initial CGT to activate the validator.',
    ],
    codeExample: '# Register validator\ndemiurge validator register --name "MyValidator"\n\n# Set commission (e.g., 10%)\ndemiurge validator set-commission --rate 10\n\n# Stake initial CGT\ndemiurge validator stake --amount 10000\n\n# Check validator status\ndemiurge validator info',
    codeLanguage: 'bash',
    tips: [
      'Lower commission attracts more nominators but reduces your direct earnings.',
      'You can change commission at any time, but large changes may cause nominator churn.',
    ],
    nextAction: 'Say "next" to monitor your validator.',
  },
  {
    title: 'Monitor and Maintain',
    description: 'Keep your validator healthy and maximize rewards.',
    instructions: [
      'Monitor your validator uptime — missed blocks reduce rewards.',
      'Check your rewards regularly and claim them as needed.',
      'Keep the node software updated to the latest version.',
    ],
    codeExample: '# Check validator status and rewards\ndemiurge validator info\ndemiurge validator rewards\n\n# Claim pending rewards\ndemiurge validator claim\n\n# Monitor node health\ncurl -s http://localhost:9944 -d \'{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}\' | jq',
    codeLanguage: 'bash',
    tips: [
      'Set up monitoring alerts for missed blocks and low peer count.',
      'Join the validator community for coordination and updates.',
      'Consider running a backup node for failover.',
    ],
    nextAction: 'Congratulations, guardian! Your validator is securing the Chain. Ask me anything!',
  },
];

// ═══════════════════════════════════════════════════════════════════════════════
// STEP RETRIEVAL
// ═══════════════════════════════════════════════════════════════════════════════

const PATHS: Record<OnboardingPath, Omit<OnboardingStep, 'stepNumber' | 'totalSteps' | 'path'>[]> = {
  user: USER_STEPS,
  developer: DEVELOPER_STEPS,
  validator: VALIDATOR_STEPS,
};

/**
 * Get a specific onboarding step
 */
export function getOnboardingStep(path: OnboardingPath, step: number): OnboardingStep {
  const steps = PATHS[path];
  const clampedStep = Math.max(0, Math.min(step, steps.length - 1));
  const stepData = steps[clampedStep];

  return {
    ...stepData,
    stepNumber: clampedStep,
    totalSteps: steps.length,
    path,
  };
}

/**
 * Get the full table of contents for an onboarding path
 */
export function getOnboardingTOC(path: OnboardingPath): { step: number; title: string }[] {
  return PATHS[path].map((s, i) => ({ step: i, title: s.title }));
}
