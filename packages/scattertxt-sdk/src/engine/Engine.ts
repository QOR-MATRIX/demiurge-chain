// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * ScatterTXT Engine Core
 * 
 * The main entry point for ScatterTXT games.
 */

import { EventEmitter } from 'eventemitter3';
import { BlockchainBridge } from '../blockchain/BlockchainBridge';
import { SessionKeyManager } from '../blockchain/SessionKeyManager';
import { DRC369Handler } from '../blockchain/DRC369Handler';
import { EnergyManager } from '../blockchain/EnergyManager';
import type { 
  ScatterConfig, 
  Player, 
  GameSession, 
  GameAction, 
  ActionResult,
  GameEvent,
} from '../types';

const DEFAULT_RPC_URL = 'http://127.0.0.1:9944';
const DEFAULT_WS_URL = 'ws://127.0.0.1:9944/ws';

interface EngineEvents {
  'connected': () => void;
  'disconnected': () => void;
  'session:started': (session: GameSession) => void;
  'session:ended': (session: GameSession) => void;
  'action:recorded': (result: ActionResult) => void;
  'event': (event: GameEvent) => void;
  'error': (error: Error) => void;
}

/**
 * ScatterTXT Game Engine
 * 
 * @example
 * ```typescript
 * const engine = await ScatterEngine.create({
 *   canvas: document.getElementById('game-canvas'),
 *   gameId: 'my-awesome-game',
 * });
 * 
 * const player = await engine.login('MyName#1234');
 * const session = await engine.startSession();
 * 
 * // Record game actions (no wallet popup!)
 * await engine.recordAction({
 *   type: 'COLLECT_ITEM',
 *   targetId: 'golden-key',
 *   timestamp: Date.now(),
 * });
 * ```
 */
export class ScatterEngine extends EventEmitter<EngineEvents> {
  private config: ScatterConfig;
  private blockchain: BlockchainBridge;
  private sessionKey: SessionKeyManager;
  private drc369: DRC369Handler;
  private energy: EnergyManager;
  
  private player: Player | null = null;
  private currentSession: GameSession | null = null;

  private constructor(config: ScatterConfig) {
    super();
    this.config = config;
    
    const rpcUrl = config.rpcUrl || DEFAULT_RPC_URL;
    const wsUrl = config.wsUrl || DEFAULT_WS_URL;
    
    this.blockchain = new BlockchainBridge(rpcUrl, wsUrl);
    this.sessionKey = new SessionKeyManager(this.blockchain);
    this.drc369 = new DRC369Handler(this.blockchain);
    this.energy = new EnergyManager(this.blockchain);
    
    // Forward blockchain events
    this.blockchain.on('event', (event) => this.emit('event', event));
    this.blockchain.on('error', (error) => this.emit('error', error));
  }

  /**
   * Create a new ScatterTXT engine instance
   */
  static async create(config: ScatterConfig = {}): Promise<ScatterEngine> {
    const engine = new ScatterEngine(config);
    await engine.blockchain.connect();
    engine.emit('connected');
    return engine;
  }

  /**
   * Authenticate player with QOR ID
   */
  async login(qorId: string): Promise<Player> {
    if (this.config.debug) {
      console.log(`[ScatterTXT] Logging in: ${qorId}`);
    }

    // Generate session key for seamless gameplay
    const sessionKey = await this.sessionKey.authenticate(qorId, [
      'game_actions',
      'asset_transfer',
      'state_update',
    ]);

    // Load player's assets and energy
    const [assets, energyInfo, balance] = await Promise.all([
      this.drc369.loadPlayerAssets(qorId),
      this.energy.getEnergy(qorId),
      this.blockchain.getBalance(qorId),
    ]);

    const address = await this.blockchain.getAddress(qorId);

    this.player = {
      qorId,
      address,
      sessionKey: sessionKey.id,
      assets,
      energy: energyInfo,
      balance,
    };

    if (this.config.debug) {
      console.log(`[ScatterTXT] Logged in as ${qorId} with ${assets.length} assets`);
    }

    return this.player;
  }

  /**
   * Start a new game session
   */
  async startSession(): Promise<GameSession> {
    if (!this.player) {
      throw new Error('Must be logged in to start a session');
    }

    const gameId = this.config.gameId || 'default';

    // Check energy
    const hasEnergy = await this.energy.hasEnough(this.player.qorId, 10);
    if (!hasEnergy) {
      throw new Error('Not enough energy to start a game session');
    }

    // Start session on-chain
    const session = await this.blockchain.call<GameSession>('gameEngine_startSession', {
      qorId: this.player.qorId,
      gameId,
      sessionKeyId: this.player.sessionKey,
    });

    this.currentSession = session;
    this.emit('session:started', session);

    if (this.config.debug) {
      console.log(`[ScatterTXT] Session started: ${session.id}`);
    }

    return session;
  }

  /**
   * Record a game action on-chain
   * Uses session key - no wallet popup!
   */
  async recordAction(action: GameAction): Promise<ActionResult> {
    if (!this.currentSession) {
      throw new Error('No active session');
    }

    // Sign action with session key
    const signedAction = await this.sessionKey.signAction(action);

    // Submit to blockchain
    const result = await this.blockchain.call<ActionResult>('gameEngine_recordAction', {
      sessionId: this.currentSession.id,
      action: signedAction,
    });

    // Update local state if needed
    if (result.state) {
      this.currentSession.state = {
        ...this.currentSession.state,
        ...result.state,
      };
    }

    this.emit('action:recorded', result);

    if (this.config.debug) {
      console.log(`[ScatterTXT] Action recorded: ${action.type}`, result);
    }

    return result;
  }

  /**
   * End the current game session
   */
  async endSession(): Promise<{ session: GameSession; rewards: import('../types').GameRewards }> {
    if (!this.currentSession) {
      throw new Error('No active session');
    }

    const result = await this.blockchain.call<{ session: GameSession; rewards: import('../types').GameRewards }>(
      'gameEngine_endSession',
      { sessionId: this.currentSession.id }
    );

    this.emit('session:ended', result.session);
    this.currentSession = null;

    if (this.config.debug) {
      console.log(`[ScatterTXT] Session ended. Rewards:`, result.rewards);
    }

    return result;
  }

  /**
   * Get current player info
   */
  getPlayer(): Player | null {
    return this.player;
  }

  /**
   * Get current session info
   */
  getSession(): GameSession | null {
    return this.currentSession;
  }

  /**
   * Get player's DRC-369 assets
   */
  async getAssets(): Promise<import('../blockchain/DRC369Handler').GameAsset[]> {
    if (!this.player) {
      return [];
    }
    return this.drc369.loadPlayerAssets(this.player.qorId);
  }

  /**
   * Update DRC-369 asset state (e.g., equip item, level up)
   */
  async updateAssetState(
    tokenId: string,
    changes: Record<string, unknown>
  ): Promise<void> {
    await this.drc369.updateAssetState(tokenId, changes, this.sessionKey);
  }

  /**
   * Evolve a DRC-369 asset
   */
  async evolveAsset(tokenId: string): Promise<import('../blockchain/DRC369Handler').EvolutionResult> {
    return this.drc369.evolveAsset(tokenId, this.sessionKey);
  }

  /**
   * Get current energy info
   */
  async getEnergy(): Promise<import('../blockchain/EnergyManager').EnergyInfo> {
    if (!this.player) {
      throw new Error('Must be logged in');
    }
    return this.energy.getEnergy(this.player.qorId);
  }

  /**
   * Subscribe to real-time game events
   */
  subscribeToEvents(callback: (event: GameEvent) => void): () => void {
    this.on('event', callback);
    return () => this.off('event', callback);
  }

  /**
   * Disconnect and cleanup
   */
  async disconnect(): Promise<void> {
    if (this.currentSession) {
      await this.endSession();
    }
    await this.blockchain.disconnect();
    this.emit('disconnected');
  }
}
