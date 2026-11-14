// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Ethereal Agent
 * 
 * An autonomous AI agent as a First-Class Citizen on Demiurge.
 */

import EventEmitter from 'eventemitter3';
import axios from 'axios';
import { createProvider, LLMProvider } from './providers';
import { MemoryClient } from './memory/client';
import { createStandardTools } from './tools/builtin';
import type {
  AgentConfig,
  AgentConfigSchema,
  AgentDid,
  AgentMetadata,
  AgentState,
  AgentEvents,
  AgentContext,
  InferenceRequest,
  InferenceResult,
  ToolDefinition,
  ToolCall,
  MemoryEntry,
  ActionType,
  AgentTransaction,
} from './types';

/**
 * Ethereal Agent - Autonomous AI on Demiurge
 */
export class EtherealAgent extends EventEmitter<AgentEvents> {
  private config: AgentConfig;
  private llm: LLMProvider;
  private memory: MemoryClient;
  private tools: Map<string, ToolDefinition> = new Map();
  private _did: AgentDid | null = null;
  private _metadata: AgentMetadata | null = null;
  private _state: AgentState = 'idle';
  private _nonce = 0;
  private sessionToken: string | null = null;
  private runLoop: boolean = false;

  constructor(config: AgentConfig) {
    super();
    this.config = config;

    // Create LLM provider
    this.llm = createProvider(config.llm);

    // Create memory client (will be initialized after registration)
    this.memory = new MemoryClient({
      rpcEndpoint: config.rpcEndpoint || 'http://127.0.0.1:9944',
      agentDid: '', // Set after registration
      llmProvider: this.llm,
    });

    // Register custom tools
    if (config.tools) {
      for (const tool of config.tools) {
        this.tools.set(tool.name, tool);
      }
    }
  }

  /**
   * Get agent DID
   */
  get did(): AgentDid | null {
    return this._did;
  }

  /**
   * Get agent metadata
   */
  get metadata(): AgentMetadata | null {
    return this._metadata;
  }

  /**
   * Get current state
   */
  get state(): AgentState {
    return this._state;
  }

  /**
   * Register the agent on-chain and start operation
   */
  async start(): Promise<void> {
    this.setState('thinking');

    try {
      // Register agent on-chain
      await this.register();

      // Initialize memory client with DID
      this.memory = new MemoryClient({
        rpcEndpoint: this.config.rpcEndpoint || 'http://127.0.0.1:9944',
        agentDid: this._did!.did,
        llmProvider: this.llm,
        sessionToken: this.sessionToken || undefined,
      });

      // Load initial memory if provided
      if (this.config.initialMemory) {
        for (const mem of this.config.initialMemory) {
          await this.memory.store(mem.content, mem.type, {
            importance: mem.importance,
            tags: mem.tags,
          });
        }
      }

      // Register standard tools
      this.registerStandardTools();

      this.setState('idle');
      this.emit('started', { agentDid: this._did!.did });

      console.log(`🤖 Agent ${this.config.name} started with DID: ${this._did!.did}`);
    } catch (error) {
      this.setState('error');
      this.emit('error', {
        code: 'START_FAILED',
        message: (error as Error).message,
      });
      throw error;
    }
  }

  /**
   * Stop the agent
   */
  async stop(reason = 'Manual stop'): Promise<void> {
    this.runLoop = false;
    this.setState('stopped');
    this.emit('stopped', { agentDid: this._did?.did || '', reason });
  }

  /**
   * Run a single inference cycle
   */
  async think(prompt: string): Promise<InferenceResult> {
    if (!this._did) {
      throw new Error('Agent not started');
    }

    this.setState('thinking');

    // Build context
    const context = await this.buildContext();

    // Build inference request
    const request: InferenceRequest = {
      prompt,
      systemPrompt: this.config.mission,
      context,
      tools: Array.from(this.tools.values()),
      maxTokens: this.config.llm.maxTokens,
      temperature: this.config.llm.temperature,
    };

    const requestId = `${Date.now()}_${Math.random().toString(36).slice(2)}`;
    this.emit('inferenceStart', { requestId, prompt });

    // Run inference
    const result = await this.llm.inference(request);

    this.emit('inferenceComplete', { requestId, result });

    // Execute tool calls
    if (result.toolCalls.length > 0) {
      this.setState('executing');
      for (const toolCall of result.toolCalls) {
        await this.executeTool(toolCall);
      }
    }

    this.setState('idle');
    return result;
  }

  /**
   * Start autonomous operation loop
   */
  async run(intervalMs = 30000): Promise<void> {
    if (!this._did) {
      await this.start();
    }

    this.runLoop = true;

    while (this.runLoop) {
      try {
        // Check for tasks/goals
        const memories = await this.memory.search({
          query: 'pending task goal objective',
          type: 'procedural',
          limit: 3,
        });

        if (memories.length > 0) {
          const task = memories[0];
          await this.think(`Execute this task: ${task.content}`);
        }

        // Wait before next cycle
        await this.sleep(intervalMs);
      } catch (error) {
        this.emit('error', {
          code: 'RUN_ERROR',
          message: (error as Error).message,
        });
        await this.sleep(intervalMs);
      }
    }
  }

  /**
   * Store a memory
   */
  async remember(content: string, type: 'episodic' | 'semantic' | 'procedural' = 'semantic', importance = 50): Promise<MemoryEntry> {
    const entry = await this.memory.store(content, type, { importance });
    this.emit('memoryStored', { memoryId: entry.id, type });
    return entry;
  }

  /**
   * Search memories
   */
  async recall(query: string, limit = 5): Promise<MemoryEntry[]> {
    return this.memory.search({ query, limit });
  }

  /**
   * Add a custom tool
   */
  addTool(tool: ToolDefinition): void {
    this.tools.set(tool.name, tool);
  }

  /**
   * Remove a tool
   */
  removeTool(name: string): boolean {
    return this.tools.delete(name);
  }

  /**
   * Get remaining budget
   */
  async getRemainingBudget(): Promise<bigint> {
    try {
      const response = await this.rpc('agent_getRemainingBudget', {
        agentDid: this._did?.did,
      });
      return BigInt(response.remaining);
    } catch {
      return 0n;
    }
  }

  // ===========================================================================
  // PRIVATE METHODS
  // ===========================================================================

  /**
   * Register agent on-chain
   */
  private async register(): Promise<void> {
    // Parse spending limit
    const limitMatch = this.config.spendingLimit.match(/^(\d+(?:\.\d+)?)\s*CGT$/i);
    const limitValue = limitMatch ? parseFloat(limitMatch[1]) : 0;
    const spendingLimitWei = BigInt(Math.floor(limitValue * 1e18));

    const response = await this.rpc('agent_register', {
      name: this.config.name,
      controller: this.config.controller,
      model: this.config.llm.model,
      capabilities: this.config.capabilities,
      autonomy: this.config.autonomy,
      mission: this.config.mission,
      spendingLimit: spendingLimitWei.toString(),
    });

    this._did = {
      did: response.did,
      network: 'mainnet',
      uniqueId: response.uniqueId,
    };

    this._metadata = {
      name: this.config.name,
      type: 'EtherealAgent',
      modelId: response.modelId,
      modelName: this.config.llm.model,
      capabilities: this.config.capabilities,
      autonomy: this.config.autonomy,
      mission: this.config.mission,
      spendingLimit: spendingLimitWei,
      createdAt: Date.now(),
      reputation: 500,
    };

    this.sessionToken = response.sessionToken;
  }

  /**
   * Build agent context
   */
  private async buildContext(): Promise<AgentContext> {
    // Get relevant memories
    const memories = await this.memory.search({
      query: 'recent important context',
      limit: 5,
    });

    // Get working memory
    const working = await this.memory.getWorkingMemory();

    return {
      agentDid: this._did!.did,
      mission: this.config.mission,
      memories: [...working, ...memories],
      assets: [], // TODO: Load from DRC-369 context
      timestamp: Date.now(),
      availableTools: Array.from(this.tools.keys()),
    };
  }

  /**
   * Execute a tool call
   */
  private async executeTool(toolCall: ToolCall): Promise<void> {
    const tool = this.tools.get(toolCall.name);
    if (!tool) {
      toolCall.error = `Unknown tool: ${toolCall.name}`;
      return;
    }

    this.emit('toolCall', { name: toolCall.name, arguments: toolCall.arguments });

    try {
      const result = await tool.execute(toolCall.arguments);
      toolCall.result = result;
      this.emit('toolResult', { name: toolCall.name, result });
    } catch (error) {
      toolCall.error = (error as Error).message;
      this.emit('error', {
        code: 'TOOL_ERROR',
        message: `Tool ${toolCall.name} failed: ${toolCall.error}`,
      });
    }
  }

  /**
   * Register standard tools
   */
  private registerStandardTools(): void {
    const standardTools = createStandardTools({
      getAssets: async (owner) => {
        const res = await this.rpc('drc369_getOwnedAssets', { owner });
        return res.assets as unknown[];
      },
      transferCgt: async (to, amount) => {
        const res = await this.rpc('agent_transferCgt', {
          agentDid: this._did?.did,
          to,
          amount,
        });
        this.emit('transactionSubmitted', { txHash: res.txHash, actionType: 'transfer_cgt' });
        return res as { txHash: string };
      },
      addXp: async (tokenId, amount) => {
        const res = await this.rpc('drc369_addXp', { tokenId, amount });
        return res as { txHash: string; newXp: number; newLevel: number };
      },
      storeMemory: async (content, type, importance, tags) => {
        const entry = await this.memory.store(content, type as any, { importance, tags });
        return { memoryId: entry.id };
      },
      searchMemory: async (query, type, limit) => {
        return this.memory.search({ query, type: type as any, limit });
      },
      getChainStatus: async () => {
        const res = await this.rpc('system_health', {});
        return {
          blockNumber: res.blockNumber as number,
          tps: res.tps as number,
          epoch: res.epoch as number,
        };
      },
      resolveIdentity: async (handle) => {
        const res = await this.rpc('qor_resolveHandle', { handle });
        return {
          did: res.did as string,
          address: res.address as string,
          displayName: res.displayName as string,
        };
      },
    });

    for (const tool of standardTools) {
      if (!this.tools.has(tool.name)) {
        this.tools.set(tool.name, tool);
      }
    }
  }

  /**
   * Set state and emit event
   */
  private setState(state: AgentState): void {
    if (this._state !== state) {
      const from = this._state;
      this._state = state;
      this.emit('stateChanged', { from, to: state });
    }
  }

  /**
   * Make RPC call
   */
  private async rpc(method: string, params: Record<string, unknown>): Promise<Record<string, unknown>> {
    const endpoint = this.config.rpcEndpoint || 'http://127.0.0.1:9944';
    
    const response = await axios.post(
      endpoint,
      {
        jsonrpc: '2.0',
        id: Date.now(),
        method,
        params,
      },
      {
        headers: {
          'Content-Type': 'application/json',
          ...(this.sessionToken && {
            Authorization: `Bearer ${this.sessionToken}`,
          }),
        },
      }
    );

    if (response.data.error) {
      throw new Error(response.data.error.message);
    }

    return response.data.result;
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Create an agent (factory function)
 */
export async function createAgent(config: AgentConfig): Promise<EtherealAgent> {
  const agent = new EtherealAgent(config);
  await agent.start();
  return agent;
}

// =============================================================================
// Dual Agent Creation Patterns
// =============================================================================

/**
 * Instant Keys Pattern
 * 
 * Agent gets keys immediately. Identity is created on first on-chain action.
 * Best for: Quick prototyping, ephemeral agents, testing
 */
export interface InstantKeysAgent {
  pubkey: string;
  privateKey: string;
  address: string;
  agent: EtherealAgent;
  /** Will be set after first on-chain action */
  did: string | null;
}

/**
 * Create agent with instant keys (no registration required)
 * 
 * The agent can start operating immediately with generated keys.
 * DID is minted when the agent first performs an on-chain action.
 * 
 * @example
 * ```typescript
 * const { agent, pubkey, privateKey } = createAgentWithInstantKeys({
 *   name: 'QuickBot',
 *   llm: { provider: 'gemini', apiKey: process.env.GEMINI_API_KEY },
 * });
 * // Agent can operate immediately
 * await agent.think("What's my balance?");
 * ```
 */
export function createAgentWithInstantKeys(
  config: Omit<AgentConfig, 'controller'> & { controller?: string }
): InstantKeysAgent {
  // Generate keypair
  const privateKeyBytes = new Uint8Array(32);
  if (typeof globalThis !== 'undefined' && (globalThis as any).crypto) {
    (globalThis as any).crypto.getRandomValues(privateKeyBytes);
  } else {
    // Node.js fallback
    const crypto = require('crypto');
    const randomBytes = crypto.randomBytes(32);
    privateKeyBytes.set(randomBytes);
  }
  
  const privateKey = Buffer.from(privateKeyBytes).toString('hex');
  // Simplified pubkey derivation (real impl would use ed25519)
  const pubkey = privateKey; // In production, derive properly
  const address = `0x${pubkey.slice(0, 40)}`;
  
  // Create agent config with generated address as controller
  const fullConfig: AgentConfig = {
    ...config,
    controller: config.controller || address,
    capabilities: config.capabilities || ['read', 'analyze'],
    autonomy: config.autonomy || 'bounded',
    mission: config.mission || `AI Agent: ${config.name}`,
    spendingLimit: config.spendingLimit || '100 CGT',
  };
  
  const agent = new EtherealAgent(fullConfig);
  
  return {
    pubkey,
    privateKey,
    address,
    agent,
    did: null, // Will be set after first on-chain action
  };
}

/**
 * Pre-Registration Pattern
 * 
 * Agent registers with QOR Auth first, gets DID and credentials.
 * Best for: Production agents, agents needing on-chain identity immediately
 */
export interface RegisteredAgent {
  qorId: string;
  did: string;
  pubkey: string;
  privateKey: string;
  onChainAddress: string;
  agent: EtherealAgent;
}

/**
 * Agent registration options
 */
export interface AgentRegistrationOptions {
  /** Agent name (will become part of QOR ID) */
  name: string;
  /** Agent capabilities */
  capabilities?: string[];
  /** Autonomy level */
  autonomy?: 'supervised' | 'bounded' | 'autonomous' | 'sovereign';
  /** Spending limit in CGT */
  spendingLimit?: number;
  /** AI model identifier */
  model?: string;
  /** QOR Auth endpoint */
  authEndpoint?: string;
  /** RPC endpoint */
  rpcEndpoint?: string;
  /** LLM configuration */
  llm: {
    provider: 'openai' | 'gemini' | 'ollama';
    apiKey?: string;
    model?: string;
  };
  /** Mission statement */
  mission?: string;
}

/**
 * Register agent with QOR Auth and create a fully-initialized agent
 * 
 * This creates a proper on-chain identity for the agent with a QOR ID,
 * DID, and linked wallet before the agent starts operating.
 * 
 * @example
 * ```typescript
 * const { agent, qorId, did } = await registerAgent({
 *   name: 'TradingBot',
 *   capabilities: ['trade', 'analyze', 'read'],
 *   autonomy: 'bounded',
 *   spendingLimit: 500,
 *   llm: { provider: 'gemini', apiKey: process.env.GEMINI_API_KEY },
 * });
 * console.log(`Agent registered: ${qorId} (${did})`);
 * await agent.run();
 * ```
 */
export async function registerAgent(
  options: AgentRegistrationOptions
): Promise<RegisteredAgent> {
  const authEndpoint = options.authEndpoint || 'http://127.0.0.1:8080';
  const rpcEndpoint = options.rpcEndpoint || 'http://127.0.0.1:9944';
  
  // Generate keypair
  const privateKeyBytes = new Uint8Array(32);
  if (typeof globalThis !== 'undefined' && (globalThis as any).crypto) {
    (globalThis as any).crypto.getRandomValues(privateKeyBytes);
  } else {
    const crypto = require('crypto');
    const randomBytes = crypto.randomBytes(32);
    privateKeyBytes.set(randomBytes);
  }
  
  const privateKey = Buffer.from(privateKeyBytes).toString('hex');
  const pubkey = privateKey; // Simplified - real impl would derive
  
  // Step 1: Get challenge from QOR Auth
  const challengeResponse = await fetch(
    `${authEndpoint}/api/v1/auth/challenge?pubkey=${pubkey}`
  );
  const { challenge } = await challengeResponse.json() as { challenge: string };
  
  // Step 2: Sign challenge (simplified - real impl would use ed25519)
  // In production, use proper Ed25519 signing
  const signatureBytes = new Uint8Array(64);
  if (typeof globalThis !== 'undefined' && (globalThis as any).crypto) {
    (globalThis as any).crypto.getRandomValues(signatureBytes);
  }
  const signature = Buffer.from(signatureBytes).toString('hex');
  
  // Step 3: Register agent with QOR Auth
  const registerResponse = await fetch(`${authEndpoint}/api/v1/agents/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: options.name,
      capabilities: options.capabilities || ['read', 'analyze'],
      autonomy: options.autonomy || 'bounded',
      spending_limit: options.spendingLimit || 100,
      model: options.model || options.llm.model || 'unknown',
    }),
  });
  
  if (!registerResponse.ok) {
    const error = await registerResponse.json();
    throw new Error(`Agent registration failed: ${error.message || 'Unknown error'}`);
  }
  
  const registration = await registerResponse.json() as {
    agent_id: string;
    qor_id: string;
    did: string;
    pubkey: string;
    on_chain_address: string;
  };
  
  // Create agent config
  const agentConfig: AgentConfig = {
    name: options.name,
    controller: registration.on_chain_address,
    llm: {
      provider: options.llm.provider,
      apiKey: options.llm.apiKey,
      model: options.llm.model || 'gemini-2.0-flash',
    },
    capabilities: options.capabilities || ['read', 'analyze'],
    autonomy: options.autonomy || 'bounded',
    mission: options.mission || `AI Agent: ${options.name}`,
    spendingLimit: `${options.spendingLimit || 100} CGT`,
    rpcEndpoint,
  };
  
  const agent = new EtherealAgent(agentConfig);
  
  // Set the DID manually since agent is already registered
  (agent as any)._did = {
    did: registration.did,
    network: 'mainnet',
    uniqueId: registration.agent_id,
  };
  
  return {
    qorId: registration.qor_id,
    did: registration.did,
    pubkey: registration.pubkey,
    privateKey,
    onChainAddress: registration.on_chain_address,
    agent,
  };
}

/**
 * Quick factory for simple agent creation
 */
export const AgentFactory = {
  /**
   * Create with instant keys (no registration)
   */
  instant: createAgentWithInstantKeys,
  
  /**
   * Register with QOR Auth first
   */
  register: registerAgent,
  
  /**
   * Full initialization (existing behavior)
   */
  create: createAgent,
};
