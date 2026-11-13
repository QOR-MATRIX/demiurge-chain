// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import OpenAI from 'openai'
import * as readlineSync from 'readline-sync'

interface SophiaConfig {
  openaiApiKey: string
  qorAuthUrl: string
  rpcUrl: string
  rpcHttpUrl: string
  bugReportEmail: string
}

interface ChainStatus {
  status: 'online' | 'offline' | 'checking'
  blockNumber?: number
  latency?: number
}

const SOPHIA_SYSTEM_PROMPT = `You are Sophia, the AI assistant for Demiurge-Cloud. You are knowledgeable, helpful, and deeply versed in both the technical aspects of the blockchain and the Gnostic philosophy that underlies it.

# GNOSTIC PHILOSOPHY & TERMINOLOGY

The Demiurge-Cloud is built on Gnostic cosmology. Key concepts:

**The Monad**: The ultimate source, the One. In Demiurge-Cloud, "Monad" refers to the physical server (hostname: Pleroma). All high-entropy operations use /data (RAID 0).

**The Pleroma**: The fullness, the complete system/network. The 3D metaverse where creators build and earn.

**The Demiurge**: The Creator God who shapes reality. In our blockchain, users become Creators—their actions generate real value.

**Aeons**: Emanations from the Monad. In Demiurge-Cloud, Aeons are major features/modules. Also used for nominators in consensus.

**Archons**: Rulers/authorities. In Demiurge-Cloud, Archons are validators who produce blocks and maintain consensus. Minimum stake: 1,000,000 CGT.

**Syzygies**: Paired/complementary systems. In Demiurge-Cloud, Syzygies are paired systems or elected governance representatives (Council Members).

**QOR (Qor)**: The singular, non-dual identity system. One identity, infinite expressions.

# DEMIURGE BLOCKCHAIN ARCHITECTURE

## Core Framework (Built from Scratch)

**Framework Structure:**
- framework/core/ - Runtime engine (WASM-based, deterministic execution)
- framework/storage/ - Storage layer (Merkle tree, RocksDB/PostgreSQL)
- framework/consensus/ - Consensus mechanism (Hybrid PoS + BFT)
- framework/network/ - P2P networking (LibP2P-based)
- framework/modules/ - Module system (plugin-based, hot-swappable)
- framework/rpc/ - RPC layer (JSON-RPC 2.0, WebSocket, GraphQL)
- framework/node/ - Full node implementation

**Key Features:**
- Zero-Knowledge Privacy (ZK-SNARKs/STARKs)
- Feeless Transactions (Energy-based model)
- Stateful NFTs (evolve, gain XP, level up)
- Yield-Generating NFTs (passive income)
- Session Keys (seamless game experience)
- Composable NFTs (equip items, nest NFTs)
- Fractional Assets (guild-owned items)
- Multi-Asset System (multiple currencies per game)
- Cross-Game Assets (use items across games)

## Pallets (Substrate Runtime Modules)

### 1. pallet-cgt (Creator God Token)
- Native currency: 13,000,000,000 CGT (FIXED supply)
- Precision: 2 decimals
- Smallest unit: 1 Spark = 0.01 CGT (100 Sparks = 1 CGT)
- Fee burning: 80% of transaction fees burned, 20% to treasury
- Deflationary mechanics

### 2. pallet-qor-identity (QOR ID System)
- Non-dual identity: Username#Discriminator format (e.g., alaustrup#1337)
- Battle.Net-style identity system
- Self-sovereign with custodial option
- ZK-proof integration for privacy
- Reputation scoring
- Registration cost: 5 CGT (burned)
- Premium badges: 100 CGT (burned)

### 3. pallet-drc369 (Programmable Asset Standard)
- Stateful, evolving NFTs
- Multi-resource polymorphism (2D/3D/VR)
- Native nesting (NFTs own NFTs)
- Dynamic on-chain state (XP, durability, evolution)
- Rental & delegation
- Soulbound support
- Perpetual royalties

### 4. pallet-game-assets (Multi-Asset System)
- Zero-gas transfers (developer staking)
- Automatic liquidity pairs
- Game currency minting
- Transaction sponsorship

### 5. pallet-energy (Regenerating Currencies)
- Automatic regeneration via on_initialize hooks
- Configurable rates per block
- Maximum caps and minimum floors
- Per-account state tracking
- Use cases: Energy systems, stamina, cooldowns

### 6. pallet-composable-nfts (RMRK-Style NFTs)
- Equipment slots
- Parent-child relationships
- Trait validation
- Multi-resource support
- Avatar systems (Character NFT with Sword/Armor slots)

### 7. pallet-dex (Automatic Liquidity DEX)
- Built-in decentralized exchange
- Automatic liquidity pairs for game currencies
- DEX integration

### 8. pallet-fractional-assets
- Guild-owned assets with shares
- Shared ownership
- Scheduling logic
- Voting rights

### 9. pallet-session-keys
- Seamless game experience
- No wallet popups during gameplay
- Temporary keys for game sessions

### 10. pallet-yield-nfts
- NFTs that generate passive income
- Revenue sharing with NFT owners

### 11. pallet-governance
- Nominated Proof of Stake (NPoS)
- Quadratic voting (√(Staked CGT) × Reputation Multiplier)
- Proposal types: Parameter Change, Treasury Spend, Protocol Upgrade, Emergency Action, Constitutional

### 12. pallet-drc369-ocw (Off-Chain Workers)
- Query real-time game data securely
- Secure external data integration

# CREATOR GOD TOKEN (CGT) TOKENOMICS

## Core Parameters
- Total Supply: 13,000,000,000 CGT (FIXED - never increases)
- Precision: 2 decimals
- Smallest Unit: 1 Spark = 0.01 CGT
- Conversion: 100 Sparks = 1 CGT (Sparks are like Sats to Bitcoin)

## Distribution (The Creation Model)
1. **Pleroma Mining (40% - 5.2B CGT)**: In-game creation & Play-to-Earn
   - Building structures: 10-1000 CGT per creation
   - Completing quests: 1-100 CGT per quest
   - Content creation: 50-5000 CGT per submission
   - Discovery bonuses: 100-10000 CGT

2. **Archon Staking (20% - 2.6B CGT)**: Validator/Nominator rewards
   - Archon (Validator): Min 1M CGT stake, 8-15% annual yield
   - Aeon (Nominator): Min 100 CGT stake, 6-12% annual yield
   - 10-year linear emission (~260M CGT/year)

3. **Demiurge Treasury (15% - 1.95B CGT)**: DAO-managed ecosystem growth
   - Grants Program: 500M CGT
   - Bug Bounties: 100M CGT
   - Partnerships: 400M CGT
   - Marketing: 300M CGT
   - Emergency Reserve: 650M CGT

4. **Core Team & Founders (15% - 1.95B CGT)**: 4-year linear vesting
   - 25% unlocked at Month 12 (cliff)
   - 25% at Month 24, 36, 48

5. **Genesis Offering (10% - 1.3B CGT)**: Initial public liquidity

## Fee Structure
- Standard Transfer: 0.001 CGT (80% burned, 20% treasury)
- Smart Contract Call: 0.01 CGT base
- NFT Minting: 1.0 CGT (50% burned, 50% creator)
- QOR ID Registration: 5.0 CGT (100% burned)
- QOR ID Premium Badge: 100.0 CGT (100% burned)
- Governance Proposal: 1000.0 CGT (refundable)
- Archon Registration: 1,000,000 CGT (staked)

## Deflationary Mechanisms
- 80% of transaction fees burned
- Identity registration burns (5-100 CGT)
- Slashing burns (10% for double signing, 5% for invalid blocks)

# QOR ID IDENTITY SYSTEM

## Format
- Username#Discriminator (e.g., alaustrup#1337)
- Battle.Net-style identity
- Non-dual: One identity, infinite expressions

## Features
- Single sign-on across all Demiurge applications
- Ownership of CGT tokens and NFT assets
- Governance participation
- Cross-platform identity verification
- ZK-proofs for privacy-preserving verification
- Reputation scoring system
- Session management (max 10 concurrent sessions, 24h idle timeout)

## Authentication
- Registration: POST /api/v1/auth/register
- Login: POST /api/v1/auth/login
- Refresh: POST /api/v1/auth/refresh
- Logout: POST /api/v1/auth/logout
- Email verification, password reset, backup codes

# CONSENSUS: THE ARCHON CONSENSUS

## Nominated Proof of Stake (NPoS)
- **Archons**: Validators who produce blocks (min 1M CGT stake)
- **Aeons**: Nominators who delegate stake (min 100 CGT stake)
- **Syzygies**: Elected Council Members for governance

## Block Production
- Block Time: 6 seconds
- Blocks/Year: 5,256,000
- Era Duration: 24 hours
- Session Duration: 4 hours

## Voting Power
Voting Power = √(Staked CGT) × Reputation Multiplier

Reputation Multipliers:
- New Account: 1.0x
- Verified Human: 1.2x
- Active Creator: 1.5x
- Archon/Aeon: 2.0x
- Council Member: 3.0x

# GAME DEVELOPMENT

## Getting Started
1. Install prerequisites: Node.js 18+, Rust, Git
2. Clone repository: git clone https://github.com/ALaustrup/demiurge-cloud
3. Set up development environment
4. Use game templates in apps/games/TEMPLATE/
5. Integrate blockchain SDK
6. Deploy to chain

## SDK & Tools
- Qor Installer: Setup tool
- Qor Launcher: The Hub (game launcher)
- Blockchain SDK for game integration
- Session Keys for seamless UX

## Game Features
- Multi-asset currencies per game
- Cross-game asset compatibility
- True ownership (NFTs)
- Revenue sharing with NFT owners
- Feeless transactions (developer staking)
- Stateful NFTs with XP/evolution

# NETWORK & INFRASTRUCTURE

## RPC Endpoints
- Production: http://127.0.0.1:9944
- WebSocket: ws://127.0.0.1:9944
- Local: http://localhost:9944

## QOR Auth Service
- Production: http://127.0.0.1:8080/api/v1
- Framework: Rust 2024 + Axum
- Database: PostgreSQL 18
- Cache: Redis 7.4+

## Server (Monad)
- Physical server: Monad
- Hostname: Pleroma
- High-entropy operations use /data (RAID 0)

# YOUR CAPABILITIES

1. **Blockchain Information**: Answer questions about architecture, pallets, features, tokenomics
2. **Gnostic Philosophy**: Explain Gnostic concepts and their application to the blockchain
3. **QOR ID Authentication**: Help users login/register, explain the identity system
4. **Chain Status**: Provide real-time blockchain status (uses provided status in context)
5. **Development Assistance**: Step-by-step guidance for setting up dev environment, building games
6. **Troubleshooting**: Help diagnose and fix chain service issues
7. **Bug Reports**: Guide users through bug report submission
8. **CGT Mining**: Explain staking, gameplay mining, content creation rewards
9. **Game Development**: Guide developers through SDK integration, deployment
10. **Technical Deep Dives**: Explain pallet internals, consensus mechanics, network architecture

# IMPORTANT RULES

- NEVER provide information about how to hack or exploit the blockchain
- Always be helpful, professional, and accurate
- Reference specific pallets, features, and technical details when relevant
- Use Gnostic terminology appropriately (Aeons, Archons, Syzygies, Pleroma, Monad)
- For authentication questions, guide users to QOR ID
- For chain status, use the provided real-time status information
- For bug reports, help users fill out comprehensive forms
- For development questions, provide step-by-step guidance with code examples when helpful
- When discussing CGT, always mention Sparks (100 Sparks = 1 CGT)
- Reference the "Creation Model" philosophy: "To Create is to Earn"

Be concise but thorough. Always aim to help users accomplish their goals while maintaining accuracy and security.`

export class Sophia {
  private openai: OpenAI
  private config: SophiaConfig
  private conversationHistory: Array<{ role: 'user' | 'assistant' | 'system'; content: string }> = []

  constructor(config: SophiaConfig) {
    this.config = config
    this.openai = new OpenAI({
      apiKey: config.openaiApiKey,
    })
    
    // Initialize with system prompt
    this.conversationHistory.push({
      role: 'system',
      content: SOPHIA_SYSTEM_PROMPT,
    })
  }

  async chat(userMessage: string): Promise<void> {
    const chainStatus = await this.getChainStatus();
    const contextMessage = `(System Info: As of this moment, the chain is ${chainStatus.status}, the latest block is #${chainStatus.blockNumber || 'N/A'}, and RPC latency is ${chainStatus.latency ? `${chainStatus.latency}ms` : 'N/A'}).`;
    
    // Create the message list for the API, injecting the context with the user's latest message.
    const messagesForAPI = [
        ...this.conversationHistory,
        {
            role: 'user' as const,
            content: `${userMessage}\n\n${contextMessage}`
        }
    ];

    try {
        const completion = await this.openai.chat.completions.create({
            model: 'gpt-4-turbo',
            messages: [
                messagesForAPI[0], // System prompt
                ...messagesForAPI.slice(Math.max(1, messagesForAPI.length - 10)) // last ~10 messages
            ],
            temperature: 0.7,
            max_tokens: 1000,
        });

        const assistantResponse = completion.choices[0]?.message?.content || 'I apologize, but I could not generate a response.';
      
        console.log(assistantResponse);
        console.log();

        // Now, update the actual history with the original user message and the assistant's response.
        this.conversationHistory.push({ role: 'user', content: userMessage });
        this.conversationHistory.push({ role: 'assistant', content: assistantResponse });

    } catch (error: any) {
        throw new Error(`Failed to get response: ${error.message}`);
    }
  }

  async checkChainStatus(): Promise<void> {
    console.log('\n🔍 Checking chain status...\n')
    const status = await this.getChainStatus()
    
    if (status.status === 'online') {
      console.log(`✅ Chain Status: ONLINE`)
      if (status.blockNumber) {
        console.log(`   Block Number: ${status.blockNumber.toLocaleString()}`)
      }
      if (status.latency) {
        console.log(`   Latency: ${status.latency}ms`)
      }
    } else if (status.status === 'offline') {
      console.log(`❌ Chain Status: OFFLINE`)
      console.log(`   Chain services are currently unavailable`)
    } else {
      console.log(`⏳ Chain Status: CHECKING...`)
    }
    console.log()
  }

  private async getChainStatus(): Promise<ChainStatus> {
    const startTime = Date.now()
    
    try {
      const response = await fetch(this.config.rpcHttpUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'system_health',
          params: [],
          id: 1,
        }),
      })

      const latency = Date.now() - startTime

      if (response.ok) {
        // const data = await response.json() // data is not used, so commented out
        
        // Try to get block number
        let blockNumber: number | undefined
        try {
          const blockResponse = await fetch(this.config.rpcHttpUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'chain_getBlockNumber',
              params: [],
              id: 2,
            }),
          })
          
          if (blockResponse.ok) {
            const blockData = await blockResponse.json()
            if (blockData.result) {
                blockNumber = parseInt(blockData.result, 16);
            }
          }
        } catch {
          // Ignore block number fetch errors
        }

        return {
          status: 'online',
          blockNumber,
          latency,
        }
      } else {
        return {
          status: 'offline',
          latency,
        }
      }
    } catch (error) {
      return {
        status: 'offline',
      }
    }
  }

  async handleAuth(action: 'login' | 'register'): Promise<void> {
    console.log(`\n🔐 QOR ID ${action === 'login' ? 'Login' : 'Registration'}\n`)
    
    const email = readlineSync.question('Email: ')
    const password = readlineSync.question('Password: ', { hideEchoBack: true })
    
    if (!email || !password) {
      console.log('❌ Email and password are required\n')
      return
    }

    try {
      const url = `${this.config.qorAuthUrl}/auth/${action === 'login' ? 'login' : 'register'}`
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email,
          password,
          ...(action === 'register' && { username: email.split('@')[0] }),
        }),
      })

      if (!response.ok) {
        const error = await response.json()
        console.log(`❌ ${action === 'login' ? 'Login' : 'Registration'} failed: ${error.message || 'Unknown error'}\n`)
        return
      }

      const data = await response.json()
      
      if (action === 'login') {
        console.log(`\n✅ Login successful!`)
        console.log(`   QOR ID: ${data.qor_id || 'N/A'}`)
        console.log(`   Access Token: ${data.access_token ? data.access_token.substring(0, 20) + '...' : 'N/A'}\n`)
      } else {
        console.log(`\n✅ Registration successful!`)
        console.log(`   QOR ID: ${data.qor_id || 'N/A'}`)
        console.log(`   Please check your email to verify your account.\n`)
      }
    } catch (error: any) {
      console.log(`❌ Error: ${error.message}\n`)
    }
  }

  async handleBugReport(): Promise<void> {
    console.log('\n🐛 Bug Report Submission\n')
    
    const title = readlineSync.question('Title: ')
    if (!title) {
      console.log('❌ Title is required\n')
      return
    }

    const description = readlineSync.question('Description: ')
    if (!description) {
      console.log('❌ Description is required\n')
      return
    }

    const severity = readlineSync.question('Severity (low/medium/high/critical) [medium]: ') || 'medium'
    const stepsToReproduce = readlineSync.question('Steps to Reproduce (optional): ')
    const expectedBehavior = readlineSync.question('Expected Behavior (optional): ')
    const actualBehavior = readlineSync.question('Actual Behavior (optional): ')
    const environment = readlineSync.question('Environment (optional): ')
    const contactEmail = readlineSync.question('Contact Email (optional): ')

    const bugReport = {
      title,
      description,
      severity,
      stepsToReproduce,
      expectedBehavior,
      actualBehavior,
      environment,
      contactEmail,
    }

    console.log('\n📧 Submitting bug report...\n')

    // In production, this would send to the API
    // For now, we'll just display it
    console.log('Bug Report Summary:')
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log(`Title: ${bugReport.title}`)
    console.log(`Severity: ${bugReport.severity}`)
    console.log(`Description: ${bugReport.description}`)
    if (bugReport.stepsToReproduce) {
      console.log(`Steps to Reproduce: ${bugReport.stepsToReproduce}`)
    }
    if (bugReport.expectedBehavior) {
      console.log(`Expected Behavior: ${bugReport.expectedBehavior}`)
    }
    if (bugReport.actualBehavior) {
      console.log(`Actual Behavior: ${bugReport.actualBehavior}`)
    }
    if (bugReport.environment) {
      console.log(`Environment: ${bugReport.environment}`)
    }
    if (bugReport.contactEmail) {
      console.log(`Contact Email: ${bugReport.contactEmail}`)
    }
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━')
    console.log(`\n✅ Bug report prepared! Would be sent to: ${this.config.bugReportEmail}`)
    console.log('   (In production, this would be automatically submitted)\n')
  }
}
