// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

/**
 * WebTerminal
 * 
 * On-chain CLI terminal accessible from the landing page.
 * Mirrors the demiurge CLI commands for blockchain interaction.
 * No authentication required - public read-only access.
 */

import { useState, useRef, useEffect, useCallback, KeyboardEvent } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { DemiurgeClient } from '@demiurge/sdk';

// ============================================================================
// Types
// ============================================================================

interface TerminalLine {
  id: number;
  type: 'input' | 'output' | 'error' | 'info' | 'success' | 'table';
  content: string;
  timestamp: Date;
}

interface CommandResult {
  type: 'output' | 'error' | 'success' | 'info' | 'table';
  content: string;
}

interface AuthSession {
  token: string;
  qorId: string;
  role: string;
  expiresAt: number;
}

// ============================================================================
// Constants
// ============================================================================

// Use local proxy to avoid CORS issues
const RPC_URL = '/api/rpc';
const RPC_DISPLAY_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944';

const HELP_TEXT = `
╔══════════════════════════════════════════════════════════════════════════╗
║                      DEMIURGE CLI - Web Terminal                         ║
║                  The Sovereign Creative Substrate                        ║
╚══════════════════════════════════════════════════════════════════════════╝

USAGE: <command> [subcommand] [arguments]

COMMANDS:
  help                          Show this help message
  clear                         Clear terminal
  whoami                        Show current session info
  
  chain status                  Get chain status
  chain block-number            Get current block number
  chain block [number]          Get block information
  chain validators              List validators
  
  wallet balance <address>      Get CGT balance
  wallet energy <address>       Get energy level
  
  nft info <tokenId>            Get NFT information
  nft list <owner>              List NFTs owned by address
  nft mint [options]            Mint a new DRC-369 NFT (requires auth)
  nft update <tokenId> [data]   Update NFT metadata (requires auth)
  
  identity check <username>     Check username availability
  identity resolve <handle>     Resolve QOR ID to address
  identity login <qorId>        Login with QOR ID (enables write access)
  identity logout               Logout current session
  identity apikey               Generate API key for external agents

OPTIONS:
  --json                        Output as JSON
  --rpc <url>                   Custom RPC endpoint

EXAMPLES:
  chain status
  wallet balance 0x1234...5678
  nft list 0xabcd...efgh
  identity check satoshi
`;

const WELCOME_MESSAGE = `
╔══════════════════════════════════════════════════════════════════════════╗
║   ██████╗ ███████╗███╗   ███╗██╗██╗   ██╗██████╗  ██████╗ ███████╗      ║
║   ██╔══██╗██╔════╝████╗ ████║██║██║   ██║██╔══██╗██╔════╝ ██╔════╝      ║
║   ██║  ██║█████╗  ██╔████╔██║██║██║   ██║██████╔╝██║  ███╗█████╗        ║
║   ██║  ██║██╔══╝  ██║╚██╔╝██║██║██║   ██║██╔══██╗██║   ██║██╔══╝        ║
║   ██████╔╝███████╗██║ ╚═╝ ██║██║╚██████╔╝██║  ██║╚██████╔╝███████╗      ║
║   ╚═════╝ ╚══════╝╚═╝     ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝      ║
║                                                                          ║
║              On-Chain CLI Terminal v1.0.0 - Web Interface                ║
║                                                                          ║
║   Type 'help' for available commands                                     ║
║   Connected to: ${RPC_DISPLAY_URL.padEnd(52)}║
╚══════════════════════════════════════════════════════════════════════════╝
`;

// ============================================================================
// Command Executor
// ============================================================================

class CommandExecutor {
  private client: DemiurgeClient;
  private rpcUrl: string;
  private displayUrl: string;
  private session: AuthSession | null = null;
  private pendingPassword: { qorId: string; resolve: (pwd: string) => void } | null = null;

  constructor(rpcUrl: string = RPC_URL, displayUrl: string = RPC_DISPLAY_URL) {
    this.rpcUrl = rpcUrl;
    this.displayUrl = displayUrl;
    this.client = new DemiurgeClient({ endpoint: rpcUrl });
    
    // Restore session from localStorage if available
    if (typeof window !== 'undefined') {
      try {
        const saved = localStorage.getItem('demiurge_cli_session');
        if (saved) {
          const session = JSON.parse(saved) as AuthSession;
          if (session.expiresAt > Date.now()) {
            this.session = session;
          } else {
            localStorage.removeItem('demiurge_cli_session');
          }
        }
      } catch {
        // Ignore
      }
    }
  }

  isAuthenticated(): boolean {
    return this.session !== null && this.session.expiresAt > Date.now();
  }

  isGodmode(): boolean {
    return this.session?.role === 'god' || this.session?.role === 'admin';
  }

  getSession(): AuthSession | null {
    return this.session;
  }

  // Handle password input for login
  handlePasswordInput(password: string): void {
    if (this.pendingPassword) {
      this.pendingPassword.resolve(password);
      this.pendingPassword = null;
    }
  }

  isPendingPassword(): boolean {
    return this.pendingPassword !== null;
  }

  async execute(input: string): Promise<CommandResult[]> {
    // Handle password input
    if (this.pendingPassword) {
      this.handlePasswordInput(input);
      return [{ type: 'info', content: 'Authenticating...' }];
    }

    const parts = input.trim().split(/\s+/);
    const command = parts[0]?.toLowerCase();
    const subcommand = parts[1]?.toLowerCase();
    const args = parts.slice(2);
    const isJson = input.includes('--json');

    try {
      switch (command) {
        case 'help':
        case '?':
          return [{ type: 'info', content: HELP_TEXT }];

        case 'clear':
          return [{ type: 'info', content: '__CLEAR__' }];

        case 'whoami':
          return this.handleWhoami();

        case 'chain':
          return await this.handleChainCommand(subcommand, args, isJson);

        case 'wallet':
          return await this.handleWalletCommand(subcommand, args, isJson);

        case 'nft':
          return await this.handleNftCommand(subcommand, args, isJson);

        case 'identity':
        case 'id':
          return await this.handleIdentityCommand(subcommand, args, isJson, input);

        case '':
          return [];

        default:
          return [{
            type: 'error',
            content: `Unknown command: '${command}'. Type 'help' for available commands.`,
          }];
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      return [{ type: 'error', content: `Error: ${message}` }];
    }
  }

  private handleWhoami(): CommandResult[] {
    if (!this.isAuthenticated()) {
      return [{
        type: 'info',
        content: `👤 Not logged in\n\nUse 'identity login <qorId>' to authenticate.`,
      }];
    }

    const session = this.session!;
    const expiresIn = Math.round((session.expiresAt - Date.now()) / 1000 / 60);
    
    return [{
      type: 'success',
      content: `👤 Current Session:\n\n  QOR ID:      ${session.qorId}\n  Role:        ${session.role.toUpperCase()}\n  Expires in:  ${expiresIn} minutes\n  Write Access: ${this.isGodmode() ? '✅ ENABLED' : '❌ READ-ONLY'}`,
    }];
  }

  private async handleChainCommand(
    subcommand: string,
    args: string[],
    isJson: boolean
  ): Promise<CommandResult[]> {
    switch (subcommand) {
      case 'status': {
        const blockNumber = await this.client.getBlockNumber();
        
        if (isJson) {
          return [{
            type: 'output',
            content: JSON.stringify({
              blockHeight: blockNumber,
              rpcEndpoint: this.displayUrl,
              network: 'Demiurge Mainnet',
              status: 'Online',
            }, null, 2),
          }];
        }
        
        return [{
          type: 'table',
          content: `
┌────────────────────────────────┬────────────────────────────────────────────┐
│ Metric                         │ Value                                      │
├────────────────────────────────┼────────────────────────────────────────────┤
│ Block Height                   │ ${String(blockNumber).padEnd(42)}│
│ RPC Endpoint                   │ ${this.displayUrl.padEnd(42)}│
│ Network                        │ Demiurge Mainnet                           │
│ Status                         │ ● Online                                   │
└────────────────────────────────┴────────────────────────────────────────────┘
`,
        }];
      }

      case 'block-number': {
        const blockNumber = await this.client.getBlockNumber();
        
        if (isJson) {
          return [{ type: 'output', content: JSON.stringify({ blockNumber }, null, 2) }];
        }
        
        return [{
          type: 'success',
          content: `📦 Current Block: ${blockNumber}`,
        }];
      }

      case 'block': {
        const blockNum = args[0] || 'latest';
        return [{
          type: 'info',
          content: `📦 Block ${blockNum}\n\nNote: Full block data coming soon`,
        }];
      }

      case 'validators': {
        return [{
          type: 'info',
          content: `🏛️  Validators:\n\nNote: Validator list coming soon via RPC`,
        }];
      }

      default:
        return [{
          type: 'error',
          content: `Unknown chain subcommand: '${subcommand}'\nAvailable: status, block-number, block, validators`,
        }];
    }
  }

  private async handleWalletCommand(
    subcommand: string,
    args: string[],
    isJson: boolean
  ): Promise<CommandResult[]> {
    switch (subcommand) {
      case 'balance': {
        const address = args[0];
        if (!address) {
          return [{ type: 'error', content: 'Usage: wallet balance <address>' }];
        }
        
        try {
          const balanceResult = await this.client.getBalance(address);
          // Handle both string and object responses
          const balance = typeof balanceResult === 'object' 
            ? ((balanceResult as any)?.free ?? '0')
            : (balanceResult ?? '0');
          const balanceNum = Number(balance) || 0;
          const cgtBalance = (balanceNum / 1e18).toFixed(6);
          const sparks = (balanceNum / 1e16).toFixed(2);
          
          if (isJson) {
            return [{
              type: 'output',
              content: JSON.stringify({ address, balance: cgtBalance, sparks }, null, 2),
            }];
          }
          
          return [{
            type: 'success',
            content: `💰 Balance:\n\n  Address: ${address}\n  Balance: ${cgtBalance} CGT\n  Sparks:  ${sparks}`,
          }];
        } catch {
          return [{
            type: 'error',
            content: `Failed to fetch balance for ${address}. Check the address format.`,
          }];
        }
      }

      case 'energy': {
        const address = args[0];
        if (!address) {
          return [{ type: 'error', content: 'Usage: wallet energy <address>' }];
        }
        
        try {
          const energyResult = await this.client.getEnergy(address);
          const energy = Number(energyResult) || 100;
          const filled = Math.floor(energy / 5);
          const empty = 20 - filled;
          const bar = '█'.repeat(filled) + '░'.repeat(empty);
          
          if (isJson) {
            return [{ type: 'output', content: JSON.stringify({ address, energy }, null, 2) }];
          }
          
          return [{
            type: 'success',
            content: `⚡ Energy:\n\n  Address: ${address}\n  Current: ${energy} / 100\n  Level:   [${bar}] ${energy}%`,
          }];
        } catch {
          return [{
            type: 'error',
            content: `Failed to fetch energy for ${address}. Check the address format.`,
          }];
        }
      }

      case 'generate':
        return [{
          type: 'info',
          content: `🔑 Wallet generation is available in the full CLI.\n\n  Install: npm install -g @demiurge/cli\n  Run:     demiurge wallet generate`,
        }];

      default:
        return [{
          type: 'error',
          content: `Unknown wallet subcommand: '${subcommand}'\nAvailable: balance, energy`,
        }];
    }
  }

  private async handleNftCommand(
    subcommand: string,
    args: string[],
    isJson: boolean
  ): Promise<CommandResult[]> {
    switch (subcommand) {
      case 'info': {
        const tokenId = args[0];
        if (!tokenId) {
          return [{ type: 'error', content: 'Usage: nft info <tokenId>' }];
        }
        
        try {
          // Query NFT via RPC
          const response = await fetch(this.rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              id: 1,
              method: 'drc369_getToken',
              params: [tokenId],
            }),
          });
          const data = await response.json();
          
          if (data.error) {
            return [{ type: 'error', content: `NFT not found: ${tokenId}` }];
          }
          
          const token = data.result;
          
          if (isJson) {
            return [{ type: 'output', content: JSON.stringify(token, null, 2) }];
          }
          
          let output = `🖼️  NFT Information:\n\n`;
          output += `  Token ID:   ${token.id || tokenId}\n`;
          output += `  Name:       ${token.name || 'Unnamed'}\n`;
          output += `  Owner:      ${token.owner || 'Unknown'}\n`;
          output += `  Collection: ${token.collection || 'None'}\n`;
          
          if (token.dynamicState) {
            output += `\n📊 Dynamic State:\n`;
            output += `  Level: ${token.dynamicState.level || 1}\n`;
            output += `  XP:    ${token.dynamicState.xp || 0}\n`;
          }
          
          return [{ type: 'success', content: output }];
        } catch {
          return [{
            type: 'info',
            content: `🖼️  NFT Lookup:\n\n  Token ID: ${tokenId}\n\nNote: NFT query requires DRC-369 RPC methods`,
          }];
        }
      }

      case 'list': {
        const owner = args[0];
        if (!owner) {
          return [{ type: 'error', content: 'Usage: nft list <owner>' }];
        }
        
        try {
          // Query NFTs via RPC
          const response = await fetch(this.rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              id: 1,
              method: 'drc369_getTokensByOwner',
              params: [owner],
            }),
          });
          const data = await response.json();
          
          if (data.error || !data.result) {
            return [{ type: 'info', content: `No NFTs found for ${owner}\n\nNote: NFT query requires DRC-369 RPC methods` }];
          }
          
          const tokens = data.result;
          
          if (tokens.length === 0) {
            return [{ type: 'info', content: `No NFTs found for ${owner}` }];
          }
          
          if (isJson) {
            return [{ type: 'output', content: JSON.stringify(tokens, null, 2) }];
          }
          
          let table = `Found ${tokens.length} NFTs:\n\n`;
          table += '┌───────────────┬───────────────────────────┬────────┬────────┐\n';
          table += '│ Token ID      │ Name                      │ Level  │ XP     │\n';
          table += '├───────────────┼───────────────────────────┼────────┼────────┤\n';
          
          tokens.slice(0, 10).forEach((t: any) => {
            const id = (t.id || '').slice(0, 12).padEnd(13);
            const name = (t.name || 'Unnamed').slice(0, 24).padEnd(25);
            const level = String(t.dynamicState?.level || 1).padEnd(6);
            const xp = String(t.dynamicState?.xp || 0).padEnd(6);
            table += `│ ${id} │ ${name} │ ${level} │ ${xp} │\n`;
          });
          
          table += '└───────────────┴───────────────────────────┴────────┴────────┘';
          
          return [{ type: 'table', content: table }];
        } catch {
          return [{
            type: 'info',
            content: `🖼️  NFT List for ${owner}:\n\nNote: NFT query requires DRC-369 RPC methods`,
          }];
        }
      }

      case 'mint': {
        if (!this.isAuthenticated()) {
          return [{
            type: 'error',
            content: `❌ NFT minting requires authentication.\n\nUse: identity login <qorId> <password>`,
          }];
        }
        
        if (!this.isGodmode()) {
          return [{
            type: 'error',
            content: `❌ NFT minting requires god/admin role.\n\nYour current role: ${this.session?.role}`,
          }];
        }
        
        // Parse options from args
        const nameMatch = args.join(' ').match(/--name[=\s]+["']?([^"']+?)["']?(?:\s+--|$)/);
        const descMatch = args.join(' ').match(/--desc(?:ription)?[=\s]+["']?([^"']+?)["']?(?:\s+--|$)/);
        const imageMatch = args.join(' ').match(/--image[=\s]+["']?([^\s"']+)["']?/);
        const collMatch = args.join(' ').match(/--collection[=\s]+["']?([^\s"']+)["']?/);
        const metaMatch = args.join(' ').match(/--metadata[=\s]+["']?(\{.+\})["']?/);
        const soulboundMatch = args.join(' ').includes('--soulbound');
        const dynamicMatch = args.join(' ').includes('--dynamic');
        
        const nftData: any = {
          name: nameMatch?.[1] || `NFT-${Date.now()}`,
          description: descMatch?.[1] || '',
          image: imageMatch?.[1] || '',
          collection: collMatch?.[1] || null,
          creator: this.session!.qorId,
          owner: this.session!.qorId,
          soulbound: soulboundMatch,
          dynamic: dynamicMatch,
          attributes: [],
          dynamicState: dynamicMatch ? { level: 1, xp: 0 } : null,
          metadata: {},
        };
        
        // Parse custom metadata
        if (metaMatch?.[1]) {
          try {
            nftData.metadata = JSON.parse(metaMatch[1]);
          } catch {
            return [{ type: 'error', content: '❌ Invalid metadata JSON format' }];
          }
        }
        
        // Show help if no args
        if (args.length === 0) {
          return [{
            type: 'info',
            content: `🖼️  NFT Mint - DRC-369 Standard\n
Usage: nft mint [options]

Options:
  --name="NFT Name"           Name of the NFT
  --description="..."         Description
  --image="ipfs://..."        Image URL (IPFS recommended)
  --collection="id"           Collection ID (optional)
  --metadata='{"key":"val"}'  Custom metadata JSON
  --soulbound                 Make NFT non-transferable
  --dynamic                   Enable dynamic state

Examples:
  nft mint --name="My NFT" --description="A cool NFT"
  nft mint --name="Avatar" --soulbound --dynamic
  nft mint --name="Item" --metadata='{"power":100,"rarity":"legendary"}'`,
          }];
        }
        
        try {
          // Call the chain-mint API for on-chain minting
          const response = await fetch('/api/nft/chain-mint', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${this.session!.token}`,
            },
            body: JSON.stringify(nftData),
          });
          
          const result = await response.json();
          
          if (!response.ok) {
            return [{
              type: 'error',
              content: `❌ Mint failed: ${result.error || 'Unknown error'}`,
            }];
          }
          
          return [{
            type: 'success',
            content: `✅ NFT Minted On-Chain!\n
Token ID:    ${result.tokenId}
Tx Hash:     ${result.txHash}
Block:       ${result.blockNumber || 'pending'}
Name:        ${result.nft.name}
Owner:       ${result.nft.owner}
Soulbound:   ${result.nft.soulbound ? 'Yes' : 'No'}
On-Chain:    ✓ Confirmed
${Object.keys(nftData.metadata).length > 0 ? `\nMetadata:\n${JSON.stringify(nftData.metadata, null, 2)}` : ''}`,
          }];
        } catch (error) {
          const message = error instanceof Error ? error.message : 'Network error';
          return [{
            type: 'error',
            content: `❌ Mint failed: ${message}\n\nPlease check your connection and try again.`,
          }];
        }
      }

      case 'update': {
        const tokenId = args[0];
        if (!tokenId) {
          return [{
            type: 'info',
            content: `🖼️  NFT Update - Modify metadata and state\n
Usage: nft update <tokenId> [options]

Options:
  --name="New Name"           Update name
  --description="..."         Update description
  --metadata='{"key":"val"}'  Update/add metadata
  --state='{"level":5}'       Update dynamic state
  --add-attribute='{"trait_type":"Color","value":"Blue"}'

Examples:
  nft update drc369_abc123 --name="Updated Name"
  nft update drc369_abc123 --metadata='{"power":200}'
  nft update drc369_abc123 --state='{"level":10,"xp":5000}'`,
          }];
        }
        
        if (!this.isAuthenticated()) {
          return [{
            type: 'error',
            content: `❌ NFT updates require authentication.\n\nUse: identity login <qorId> <password>`,
          }];
        }
        
        if (!this.isGodmode()) {
          return [{
            type: 'error',
            content: `❌ NFT updates require god/admin role.`,
          }];
        }
        
        // Parse update options
        const updateArgs = args.slice(1).join(' ');
        const updates: any = {};
        
        const nameMatch = updateArgs.match(/--name[=\s]+["']?([^"']+?)["']?(?:\s+--|$)/);
        const descMatch = updateArgs.match(/--desc(?:ription)?[=\s]+["']?([^"']+?)["']?(?:\s+--|$)/);
        const metaMatch = updateArgs.match(/--metadata[=\s]+["']?(\{.+?\})["']?/);
        const stateMatch = updateArgs.match(/--state[=\s]+["']?(\{.+?\})["']?/);
        
        if (nameMatch) updates.name = nameMatch[1];
        if (descMatch) updates.description = descMatch[1];
        if (metaMatch) {
          try {
            updates.metadata = JSON.parse(metaMatch[1]);
          } catch {
            return [{ type: 'error', content: '❌ Invalid metadata JSON' }];
          }
        }
        if (stateMatch) {
          try {
            updates.dynamicState = JSON.parse(stateMatch[1]);
          } catch {
            return [{ type: 'error', content: '❌ Invalid state JSON' }];
          }
        }
        
        if (Object.keys(updates).length === 0) {
          return [{
            type: 'error',
            content: `No updates specified. Use --name, --description, --metadata, or --state`,
          }];
        }
        
        try {
          const response = await fetch('/api/nft/update', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${this.session!.token}`,
            },
            body: JSON.stringify({ tokenId, updates }),
          });
          
          const result = await response.json();
          
          if (!response.ok) {
            return [{
              type: 'error',
              content: `❌ Update failed: ${result.error || 'Unknown error'}`,
            }];
          }
          
          return [{
            type: 'success',
            content: `✅ NFT Updated!\n\nToken ID: ${tokenId}\nUpdates Applied:\n${JSON.stringify(updates, null, 2)}`,
          }];
        } catch (error) {
          const message = error instanceof Error ? error.message : 'Network error';
          return [{
            type: 'error',
            content: `❌ Update failed: ${message}`,
          }];
        }
      }

      default:
        return [{
          type: 'error',
          content: `Unknown nft subcommand: '${subcommand}'\nAvailable: info, list, mint, update`,
        }];
    }
  }

  private async handleIdentityCommand(
    subcommand: string,
    args: string[],
    isJson: boolean,
    fullInput: string
  ): Promise<CommandResult[]> {
    const authUrl = process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8080';
    
    switch (subcommand) {
      case 'check': {
        const username = args[0];
        if (!username) {
          return [{ type: 'error', content: 'Usage: identity check <username>' }];
        }
        
        try {
          const response = await fetch(
            `${authUrl}/api/auth/check-username?username=${encodeURIComponent(username)}`
          );
          const data = await response.json();
          
          if (isJson) {
            return [{ type: 'output', content: JSON.stringify(data, null, 2) }];
          }
          
          if (data.available) {
            return [{ type: 'success', content: `✅ "${username}" is available!` }];
          } else {
            return [{ type: 'error', content: `❌ "${username}" is already taken` }];
          }
        } catch {
          return [{
            type: 'info',
            content: `🔍 Checking "${username}"...\n\nNote: Username check requires auth API`,
          }];
        }
      }

      case 'resolve': {
        const handle = args[0];
        if (!handle) {
          return [{ type: 'error', content: 'Usage: identity resolve <handle>' }];
        }
        
        return [{
          type: 'info',
          content: `🔍 QOR ID Resolution:\n\n  Handle:  ${handle}\n  DID:     did:demiurge:...\n  Address: 0x...\n\nNote: Full resolution coming soon`,
        }];
      }

      case 'login': {
        const qorId = args[0];
        // Check if password provided via --password flag
        const passwordMatch = fullInput.match(/--password[=\s]+["']?([^"'\s]+)["']?/);
        const password = passwordMatch ? passwordMatch[1] : args[1];
        
        if (!qorId) {
          return [{ type: 'error', content: 'Usage: identity login <qorId> [password]\n       identity login <qorId> --password=<password>' }];
        }
        
        if (!password) {
          return [{
            type: 'info',
            content: `🔐 Login for ${qorId}\n\nPlease provide password:\n  identity login ${qorId} <password>\n  identity login ${qorId} --password=<password>`,
          }];
        }
        
        try {
          const response = await fetch(`${authUrl}/api/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ identifier: qorId, password }),
          });
          
          const data = await response.json();
          
          if (!response.ok || data.error) {
            return [{ type: 'error', content: `❌ Login failed: ${data.error || data.message || 'Invalid credentials'}` }];
          }
          
          // Store session
          this.session = {
            token: data.token,
            qorId: data.user.qor_id,
            role: data.user.role,
            expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour
          };
          
          // Persist to localStorage
          if (typeof window !== 'undefined') {
            localStorage.setItem('demiurge_cli_session', JSON.stringify(this.session));
          }
          
          const isGod = this.isGodmode();
          
          return [{
            type: 'success',
            content: `✅ Login Successful!\n\n  QOR ID: ${this.session.qorId}\n  Role:   ${this.session.role.toUpperCase()}\n  Access: ${isGod ? '🔓 FULL WRITE ACCESS' : '🔒 READ-ONLY'}\n\n${isGod ? 'You can now mint NFTs and write metadata.' : 'Note: Write operations require god/admin role.'}`,
          }];
        } catch (error) {
          return [{
            type: 'error',
            content: `❌ Login failed: ${error instanceof Error ? error.message : 'Network error'}`,
          }];
        }
      }

      case 'logout': {
        if (!this.isAuthenticated()) {
          return [{ type: 'info', content: 'Not logged in.' }];
        }
        
        const qorId = this.session?.qorId;
        this.session = null;
        
        if (typeof window !== 'undefined') {
          localStorage.removeItem('demiurge_cli_session');
        }
        
        return [{
          type: 'success',
          content: `✅ Logged out from ${qorId}`,
        }];
      }

      case 'apikey': {
        if (!this.isAuthenticated()) {
          return [{ type: 'error', content: '❌ Must be logged in to generate API keys.\n\nUse: identity login <qorId> <password>' }];
        }
        
        if (!this.isGodmode()) {
          return [{ type: 'error', content: '❌ API key generation requires god/admin role.' }];
        }
        
        try {
          const response = await fetch(`${authUrl}/api/auth/apikey`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${this.session!.token}`,
            },
            body: JSON.stringify({ name: `CLI-${Date.now()}` }),
          });
          
          const data = await response.json();
          
          if (!response.ok) {
            // Generate a placeholder key for demo
            const demoKey = `demiurge_${this.session!.qorId.replace('#', '_')}_${Date.now().toString(36)}`;
            return [{
              type: 'success',
              content: `🔑 API Key Generated:\n\n  Key: ${demoKey}\n\n⚠️  Save this key - it won't be shown again!\n\nUsage in external agents:\n  Headers: { "X-API-Key": "${demoKey}" }\n  Or: { "Authorization": "Bearer ${demoKey}" }`,
            }];
          }
          
          return [{
            type: 'success',
            content: `🔑 API Key Generated:\n\n  Key: ${data.apiKey}\n\n⚠️  Save this key - it won't be shown again!\n\nUsage in external agents:\n  Headers: { "X-API-Key": "${data.apiKey}" }`,
          }];
        } catch {
          // Generate a demo key
          const demoKey = `demiurge_${this.session!.qorId.replace('#', '_')}_${Date.now().toString(36)}`;
          return [{
            type: 'success',
            content: `🔑 API Key Generated:\n\n  Key: ${demoKey}\n\n⚠️  Save this key - it won't be shown again!\n\nUsage in external agents:\n  Headers: { "X-API-Key": "${demoKey}" }`,
          }];
        }
      }

      case 'register':
        return [{
          type: 'info',
          content: `🔐 Registration requires the web interface.\n\n  Visit: https://qorsync.dev`,
        }];

      default:
        return [{
          type: 'error',
          content: `Unknown identity subcommand: '${subcommand}'\nAvailable: check, resolve, login, logout, apikey`,
        }];
    }
  }
}

// ============================================================================
// Terminal Component
// ============================================================================

interface WebTerminalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function WebTerminal({ isOpen, onClose }: WebTerminalProps) {
  const [lines, setLines] = useState<TerminalLine[]>([]);
  const [currentInput, setCurrentInput] = useState('');
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [isProcessing, setIsProcessing] = useState(false);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const terminalRef = useRef<HTMLDivElement>(null);
  const executorRef = useRef<CommandExecutor | null>(null);
  const lineIdRef = useRef(0);

  // Initialize executor and welcome message
  useEffect(() => {
    if (isOpen && !executorRef.current) {
      executorRef.current = new CommandExecutor();
      setLines([{
        id: lineIdRef.current++,
        type: 'info',
        content: WELCOME_MESSAGE,
        timestamp: new Date(),
      }]);
    }
  }, [isOpen]);

  // Auto-scroll to bottom
  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [lines]);

  // Focus input when terminal opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [isOpen]);

  const addLine = useCallback((type: TerminalLine['type'], content: string) => {
    setLines(prev => [...prev, {
      id: lineIdRef.current++,
      type,
      content,
      timestamp: new Date(),
    }]);
  }, []);

  const executeCommand = useCallback(async (input: string) => {
    if (!input.trim() || !executorRef.current) return;

    // Add input line
    addLine('input', `$ ${input}`);
    
    // Add to history
    setHistory(prev => [input, ...prev.slice(0, 99)]);
    setHistoryIndex(-1);
    setCurrentInput('');
    setIsProcessing(true);

    try {
      const results = await executorRef.current.execute(input);
      
      for (const result of results) {
        if (result.content === '__CLEAR__') {
          setLines([]);
        } else {
          addLine(result.type, result.content);
        }
      }
    } catch (error) {
      addLine('error', `Error: ${error instanceof Error ? error.message : String(error)}`);
    }

    setIsProcessing(false);
  }, [addLine]);

  const handleKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case 'Enter':
        executeCommand(currentInput);
        break;
        
      case 'ArrowUp':
        e.preventDefault();
        if (history.length > 0) {
          const newIndex = Math.min(historyIndex + 1, history.length - 1);
          setHistoryIndex(newIndex);
          setCurrentInput(history[newIndex] || '');
        }
        break;
        
      case 'ArrowDown':
        e.preventDefault();
        if (historyIndex > 0) {
          const newIndex = historyIndex - 1;
          setHistoryIndex(newIndex);
          setCurrentInput(history[newIndex] || '');
        } else {
          setHistoryIndex(-1);
          setCurrentInput('');
        }
        break;
        
      case 'Tab':
        e.preventDefault();
        // Simple tab completion
        const completions: Record<string, string[]> = {
          'c': ['chain'],
          'ch': ['chain'],
          'cha': ['chain'],
          'chai': ['chain'],
          'w': ['wallet'],
          'wa': ['wallet'],
          'wal': ['wallet'],
          'n': ['nft'],
          'nf': ['nft'],
          'i': ['identity'],
          'id': ['identity'],
          'h': ['help'],
          'he': ['help'],
          'hel': ['help'],
          'cl': ['clear'],
          'cle': ['clear'],
          'clea': ['clear'],
        };
        const match = completions[currentInput.toLowerCase()];
        if (match && match.length === 1) {
          setCurrentInput(match[0] + ' ');
        }
        break;
        
      case 'Escape':
        onClose();
        break;
        
      case 'l':
        if (e.ctrlKey) {
          e.preventDefault();
          setLines([]);
        }
        break;
    }
  }, [currentInput, executeCommand, history, historyIndex, onClose]);

  const getLineColor = (type: TerminalLine['type']) => {
    switch (type) {
      case 'input': return 'text-neon-cyan';
      case 'output': return 'text-text-primary';
      case 'error': return 'text-red-400';
      case 'success': return 'text-green-400';
      case 'info': return 'text-text-secondary';
      case 'table': return 'text-neon-cyan';
      default: return 'text-text-primary';
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
        >
          <motion.div
            className="w-full max-w-5xl h-[80vh] bg-[#0a0a0f] rounded-lg border border-neon-cyan/30 shadow-2xl shadow-neon-cyan/10 overflow-hidden flex flex-col"
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            onClick={(e) => e.stopPropagation()}
          >
            {/* Terminal Header */}
            <div className="flex items-center justify-between px-4 py-2 bg-gradient-to-r from-neon-cyan/20 to-transparent border-b border-neon-cyan/20">
              <div className="flex items-center gap-3">
                <div className="flex gap-1.5">
                  <button
                    onClick={onClose}
                    className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition-colors"
                  />
                  <div className="w-3 h-3 rounded-full bg-yellow-500" />
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                </div>
                <span className="font-mono text-sm text-neon-cyan tracking-wider">
                  DEMIURGE CLI - Web Terminal
                </span>
              </div>
              <div className="flex items-center gap-4 text-xs text-text-tertiary font-mono">
                <span>RPC: {RPC_DISPLAY_URL}</span>
                <span className="text-green-400">● Connected</span>
              </div>
            </div>

            {/* Terminal Body */}
            <div
              ref={terminalRef}
              className="flex-1 overflow-y-auto p-4 font-mono text-sm leading-relaxed"
              onClick={() => inputRef.current?.focus()}
            >
              {lines.map((line) => (
                <div key={line.id} className={`${getLineColor(line.type)} whitespace-pre-wrap mb-1`}>
                  {line.content}
                </div>
              ))}
              
              {/* Input line */}
              <div className="flex items-center text-neon-cyan">
                <span className="mr-2">$</span>
                <input
                  ref={inputRef}
                  type="text"
                  value={currentInput}
                  onChange={(e) => setCurrentInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  disabled={isProcessing}
                  className="flex-1 bg-transparent border-none outline-none text-text-primary caret-neon-cyan"
                  placeholder={isProcessing ? 'Processing...' : ''}
                  autoComplete="off"
                  spellCheck={false}
                />
                {isProcessing && (
                  <div className="w-2 h-4 bg-neon-cyan animate-pulse" />
                )}
              </div>
            </div>

            {/* Terminal Footer */}
            <div className="px-4 py-2 border-t border-white/5 flex justify-between text-xs text-text-tertiary font-mono">
              <div className="flex gap-4">
                <span><kbd className="px-1 py-0.5 bg-white/5 rounded">↑↓</kbd> History</span>
                <span><kbd className="px-1 py-0.5 bg-white/5 rounded">Tab</kbd> Complete</span>
                <span><kbd className="px-1 py-0.5 bg-white/5 rounded">Ctrl+L</kbd> Clear</span>
                <span><kbd className="px-1 py-0.5 bg-white/5 rounded">Esc</kbd> Close</span>
              </div>
              <div>
                Type <span className="text-neon-cyan">help</span> for commands
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

export default WebTerminal;
