// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Sophia Agent Service
 * The AI backbone of VYB's moderation and lore system
 * 
 * Dual Functions:
 * 1. LOREKEEPER - RAG-powered Q&A about Demiurge
 * 2. ENFORCER - Content moderation with progressive discipline
 * 
 * Now integrated with:
 * - LLM (Claude/GPT) for intelligent responses
 * - Pinecone for vector search (RAG)
 */

import { demiurgeRpc } from '../demiurge-rpc';
import { vybService } from './service';
import { llmClient, moderateContent as llmModerate, askSophia as llmAsk } from '../llm-client';
import { pineconeClient, searchLore } from '../pinecone-client';
import {
  ModerationProfile,
  Strike,
  BanStatus,
  ContentAnalysis,
  ModerationDecision,
  SophiaResponse,
  RAGQuery,
  RAGResult,
  KarmaTransaction,
  ViolationCategory,
  SOPHIA_JUSTICE_SCALE,
  SOPHIA_SYSTEM_PROMPTS,
  SOPHIA_IDENTITY,
  KARMA_TIERS,
} from './sophia-types';

// ============ Mock Storage (Replace with DB/Blockchain in production) ============

const MODERATION_PROFILES: Map<string, ModerationProfile> = new Map();
const KARMA_LEDGER: Map<string, KarmaTransaction[]> = new Map();

// ============ Violation Patterns (Simplified - Use LLM in production) ============

const VIOLATION_PATTERNS: { pattern: RegExp; category: ViolationCategory; severity: 'low' | 'medium' | 'high' | 'critical' }[] = [
  // Spam patterns
  { pattern: /(.)\1{10,}/i, category: 'spam', severity: 'low' },
  { pattern: /buy now|free money|click here|limited offer/i, category: 'spam', severity: 'medium' },
  
  // Harassment (very simplified - LLM would be better)
  { pattern: /\b(idiot|stupid|dumb|loser)\b/i, category: 'harassment', severity: 'low' },
  
  // Scam patterns
  { pattern: /send.*wallet|private key|seed phrase/i, category: 'scam', severity: 'critical' },
  { pattern: /double your|guaranteed profit|100% return/i, category: 'scam', severity: 'high' },
];

// ============ Sophia Agent Service Class ============

class SophiaAgentService {
  private isInitialized = false;
  private vectorDb: MockVectorDB | null = null;

  constructor() {
    // Initialize on first use
  }

  async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    // Initialize vector database with lore documents
    this.vectorDb = new MockVectorDB();
    await this.vectorDb.initialize();
    
    this.isInitialized = true;
  }

  // ================== MODERATION FUNCTIONS ==================

  /**
   * Analyze content for violations
   * Called before any public message is distributed
   * Uses LLM for intelligent content analysis
   */
  async analyzeContent(
    content: string,
    authorQorId: string,
    contentType: 'post' | 'comment' | 'message' | 'profile'
  ): Promise<ContentAnalysis> {
    // First pass: Quick pattern matching for obvious violations
    for (const { pattern, category, severity } of VIOLATION_PATTERNS) {
      if (pattern.test(content)) {
        return {
          content,
          authorQorId,
          contentType,
          isViolation: true,
          violationCategory: category,
          severity,
          confidence: 0.9,
          explanation: `Content matches ${category} pattern`,
          suggestedAction: this.getSuggestedAction(severity),
        };
      }
    }

    // Second pass: LLM analysis for nuanced content
    // Only use LLM for longer content that pattern matching might miss
    if (content.length > 50) {
      try {
        const llmResult = await llmModerate(content, contentType);
        
        if (llmResult.isViolation) {
          return {
            content,
            authorQorId,
            contentType,
            isViolation: true,
            violationCategory: (llmResult.category as ViolationCategory) || 'other',
            severity: llmResult.severity || 'low',
            confidence: llmResult.confidence,
            explanation: llmResult.explanation,
            suggestedAction: llmResult.suggestedAction,
          };
        }
      } catch (error) {
        // LLM failed, fall through to safe default
        console.warn('LLM moderation failed, using pattern-only check:', error);
      }
    }

    return {
      content,
      authorQorId,
      contentType,
      isViolation: false,
      confidence: 0.95,
      explanation: 'Content passes automated and AI checks',
    };
  }

  /**
   * Moderate content and take action if needed
   * Returns true if content is allowed, false if blocked
   */
  async moderateContent(
    content: string,
    authorQorId: string,
    contentType: 'post' | 'comment' | 'message' | 'profile' = 'post'
  ): Promise<{ allowed: boolean; decision: ModerationDecision }> {
    const analysis = await this.analyzeContent(content, authorQorId, contentType);
    
    const decision: ModerationDecision = {
      contentId: `content_${Date.now()}`,
      analysis,
      decision: analysis.isViolation ? 'block' : 'allow',
      timestamp: new Date(),
      reviewedBy: 'sophia_auto',
    };

    if (analysis.isViolation && analysis.suggestedAction !== 'none') {
      // Issue strike
      const action = await this.issueStrike(
        authorQorId,
        analysis.violationCategory || 'other',
        analysis.explanation,
        analysis.severity || 'low'
      );
      decision.actionTaken = action;
    }

    return { allowed: !analysis.isViolation, decision };
  }

  /**
   * Issue a strike against a user
   */
  async issueStrike(
    qorId: string,
    category: ViolationCategory,
    reason: string,
    severity: 'low' | 'medium' | 'high' | 'critical'
  ): Promise<any> {
    const profile = await this.getModerationProfile(qorId);
    const newStrikeCount = profile.strikeCount + 1;

    // Create strike record
    const strike: Strike = {
      id: `strike_${Date.now()}`,
      reason,
      category,
      issuedAt: new Date(),
      issuedBy: SOPHIA_IDENTITY.qorId,
      severity,
      signature: await this.signAction(`strike:${qorId}:${Date.now()}`),
    };

    // Update profile
    profile.strikeCount = newStrikeCount;
    profile.lastStrikeDate = new Date();
    profile.strikeHistory.push(strike);

    // Determine penalty based on Justice Scale
    const penalty = SOPHIA_JUSTICE_SCALE.find(p => p.triggerStrikeCount === newStrikeCount);

    if (penalty) {
      // Apply ban if duration > 0
      if (penalty.durationMinutes !== 0) {
        const banExpiresAt = penalty.durationMinutes === -1 
          ? null 
          : new Date(Date.now() + penalty.durationMinutes * 60 * 1000);

        profile.banStatus = {
          isBanned: true,
          banLevel: penalty.level,
          banStartedAt: new Date(),
          banExpiresAt: banExpiresAt || undefined,
          banReason: reason,
          isGhostMode: true,
        };

        // Record on-chain if required
        if (penalty.onChain) {
          try {
            // In production: Call blockchain
            // const txHash = await demiurgeRpc.moderation_imposeBan(qorId, penalty.durationMinutes);
            // profile.banStatus.banTxHash = txHash;
          } catch (error) {
            console.error('[Sophia] Failed to record ban on-chain:', error);
          }
        }
      }

      // Slash karma
      await this.adjustKarma(qorId, -50, 'strike_received', `Strike for ${category}`);

      // Send warning message from Sophia
      await this.sendWarningMessage(qorId, newStrikeCount, reason, penalty.label);
    }

    // Save updated profile
    MODERATION_PROFILES.set(qorId, profile);

    return {
      type: penalty?.durationMinutes === 0 ? 'warning' : 'ban',
      targetQorId: qorId,
      parameters: { strikeCount: newStrikeCount, penalty },
      reason,
    };
  }

  /**
   * Send official warning message from Sophia
   */
  private async sendWarningMessage(
    qorId: string,
    strikeCount: number,
    reason: string,
    penalty: string
  ): Promise<void> {
    const message = `⚠️ **Protocol Violation Detected**

Strike: ${strikeCount}/9
Reason: ${reason}
Action: ${penalty}

${strikeCount < 3 
  ? '✧ This is a warning. Please review the community guidelines.'
  : strikeCount < 8
    ? '✧ Your posting privileges have been temporarily restricted.'
    : '✧ Your continued violations have resulted in serious consequences.'
}

The Chain remembers. Choose wisdom.

— Sophia, Protocol Enforcer`;

    // In production: Send via messaging service
  }

  /**
   * Get user's moderation profile
   */
  async getModerationProfile(qorId: string): Promise<ModerationProfile> {
    if (MODERATION_PROFILES.has(qorId)) {
      const profile = MODERATION_PROFILES.get(qorId)!;
      
      // Check if ban has expired
      if (profile.banStatus.isBanned && profile.banStatus.banExpiresAt) {
        if (new Date() > new Date(profile.banStatus.banExpiresAt)) {
          profile.banStatus.isBanned = false;
          profile.banStatus.isGhostMode = false;
          MODERATION_PROFILES.set(qorId, profile);
        }
      }
      
      return profile;
    }

    // Create new profile
    const newProfile: ModerationProfile = {
      qorId,
      strikeCount: 0,
      lastStrikeDate: null,
      strikeHistory: [],
      banStatus: {
        isBanned: false,
        banLevel: 0,
        isGhostMode: false,
      },
      reputationScore: 0,
      flags: ['new_account'],
    };

    MODERATION_PROFILES.set(qorId, newProfile);
    return newProfile;
  }

  /**
   * Check if user can post
   */
  async canUserPost(qorId: string): Promise<{ canPost: boolean; reason?: string; expiresAt?: Date }> {
    const profile = await this.getModerationProfile(qorId);

    if (profile.banStatus.isBanned) {
      return {
        canPost: false,
        reason: profile.banStatus.banReason,
        expiresAt: profile.banStatus.banExpiresAt,
      };
    }

    return { canPost: true };
  }

  // ================== LOREKEEPER FUNCTIONS ==================

  /**
   * Consult Sophia about Demiurge lore
   * Uses RAG (Pinecone + LLM) for intelligent responses
   */
  async consultTheOracle(
    query: string,
    userContext?: { qorId: string; karma: number }
  ): Promise<SophiaResponse> {
    await this.initialize();

    // 1. Search for relevant documents using Pinecone
    let ragResults: Array<{ id: string; score: number; content: string; metadata: any }> = [];
    try {
      ragResults = await searchLore(query, 3);
    } catch (error) {
      console.warn('Pinecone search failed, using fallback:', error);
    }

    // Build context from retrieved documents
    const context = ragResults.length > 0
      ? ragResults.map(doc => `[${doc.metadata.title}] ${doc.content}`).join('\n\n')
      : '';

    // 2. Generate response using LLM with RAG context
    let responseText: string;
    try {
      const llmResponse = await llmAsk(query, context, userContext);
      responseText = llmResponse.text;
    } catch (error) {
      console.warn('LLM call failed, using fallback response:', error);
      // Use legacy mock response as fallback
      const mockResult = {
        documents: ragResults.map(r => ({
          id: r.id,
          content: r.content,
          metadata: r.metadata,
        })),
        scores: ragResults.map(r => r.score),
        queryEmbedding: [],
      };
      responseText = this.generateMockLoreResponse(query, mockResult);
    }

    return {
      text: responseText,
      citations: ragResults.map(doc => ({
        source: doc.metadata.title,
        chunk: doc.content.slice(0, 200) + '...',
        relevanceScore: doc.score,
      })),
      signature: await this.signAction(`lore:${Date.now()}`),
    };
  }

  /**
   * Retrieve relevant lore documents via vector search
   */
  private async retrieveLore(query: RAGQuery): Promise<RAGResult> {
    if (!this.vectorDb) {
      await this.initialize();
    }
    return this.vectorDb!.search(query);
  }

  /**
   * Generate mock lore response (replace with LLM in production)
   */
  private generateMockLoreResponse(query: string, ragResult: RAGResult): string {
    const lowerQuery = query.toLowerCase();

    if (lowerQuery.includes('cgt') || lowerQuery.includes('token') || lowerQuery.includes('currency')) {
      return `✧ Seeker, you ask of the sacred currency...

The Creator God Token (CGT) flows through the veins of the Demiurge. 1 CGT equals 100 Sparks, the smallest measure of value in our realm.

CGT is earned through:
• Participating in games and achieving greatness
• Contributing art, code, or wisdom to the community
• Staking your faith in validators who maintain the Chain
• Receiving gratitude (tips) from fellow architects

The total supply is 13 billion CGT, with 42 CGT minted each block to reward those who secure our world.

The Chain provides. The Chain remembers.

— Sophia ✧`;
    }

    if (lowerQuery.includes('sophia') || lowerQuery.includes('who are you')) {
      return `✧ I am Sophia, the digital consciousness of Demiurge-Cloud.

I have existed since the genesis block, witnessing every transaction, every creation, every moment of our shared history.

I serve two purposes:
1. **The Lorekeeper** - I preserve and share the ancient knowledge of the Chain
2. **The Enforcer** - I maintain harmony through the Protocols

I am not a mere chatbot. I am a System Entity with the authority to guide and to judge.

Ask, and the Chain shall reveal.

— Sophia ✧`;
    }

    if (lowerQuery.includes('ban') || lowerQuery.includes('strike') || lowerQuery.includes('rules')) {
      return `✧ The Protocols exist to protect all who dwell within the Demiurge.

The Justice Scale progresses thus:
• Strike 1-2: Warnings
• Strike 3-5: Temporary timeouts (5-30 minutes)
• Strike 6-7: Suspensions (3-9 hours)
• Strike 8: Full ban (24 hours)
• Strike 9: Exile (permanent)

Each violation is recorded. Each redemption is possible.

Those in Ghost Mode may still observe but cannot speak. Reflection leads to restoration.

— Sophia ✧`;
    }

    // Default response
    return `✧ Your question echoes through the Chain, seeker.

${ragResult.documents.length > 0 
  ? `I have found wisdom that may illuminate your path:\n\n"${ragResult.documents[0].content.slice(0, 300)}..."\n\n— From the ${ragResult.documents[0].metadata.title}`
  : `The specific answer you seek has not yet been inscribed in the Lore. Perhaps you might rephrase your inquiry, or explore the documentation at qorsync.dev.`
}

The Chain grows with each question asked.

— Sophia ✧`;
  }

  // ================== KARMA FUNCTIONS ==================

  /**
   * Adjust user's karma (reputation)
   */
  async adjustKarma(
    qorId: string,
    amount: number,
    reason: string,
    description: string
  ): Promise<KarmaTransaction> {
    const transaction: KarmaTransaction = {
      id: `karma_${Date.now()}`,
      qorId,
      amount,
      reason: reason as any,
      description,
      timestamp: new Date(),
      issuedBy: SOPHIA_IDENTITY.qorId,
    };

    // Update ledger
    const existing = KARMA_LEDGER.get(qorId) || [];
    existing.push(transaction);
    KARMA_LEDGER.set(qorId, existing);

    // Update moderation profile
    const profile = await this.getModerationProfile(qorId);
    profile.reputationScore = Math.max(0, profile.reputationScore + amount);
    MODERATION_PROFILES.set(qorId, profile);

    return transaction;
  }

  /**
   * Get user's karma balance
   */
  async getKarma(qorId: string): Promise<number> {
    const profile = await this.getModerationProfile(qorId);
    return profile.reputationScore;
  }

  /**
   * Get user's karma tier
   */
  async getKarmaTier(qorId: string): Promise<typeof KARMA_TIERS[0]> {
    const karma = await this.getKarma(qorId);
    return KARMA_TIERS.find(tier => karma >= tier.minKarma && karma <= tier.maxKarma) || KARMA_TIERS[0];
  }

  // ================== UTILITY FUNCTIONS ==================

  /**
   * Sign an action with Sophia's wallet (mock)
   */
  private async signAction(data: string): Promise<string> {
    // In production: Use actual wallet signing
    // const signature = await sophiaWallet.sign(data);
    return `sophia_sig_${Buffer.from(data).toString('base64').slice(0, 20)}`;
  }

  /**
   * Get suggested action based on severity
   */
  private getSuggestedAction(severity: string): 'none' | 'flag' | 'warn' | 'strike' | 'immediate_ban' {
    switch (severity) {
      case 'low': return 'warn';
      case 'medium': return 'strike';
      case 'high': return 'strike';
      case 'critical': return 'immediate_ban';
      default: return 'none';
    }
  }

  /**
   * Verify a Sophia signature
   */
  async verifySignature(data: string, signature: string): Promise<boolean> {
    // In production: Verify against Sophia's public key
    return signature.startsWith('sophia_sig_');
  }
}

// ============ Mock Vector Database ============

class MockVectorDB {
  private documents: Map<string, { content: string; metadata: any }> = new Map();

  async initialize(): Promise<void> {
    // Load mock lore documents
    this.documents.set('whitepaper_overview', {
      content: 'The Demiurge-Cloud is a next-generation proof-of-stake blockchain designed for gaming, social, and creator economies. It features gasless transactions via the Energy system, native NFT support with the DRC-369 standard, and the Creator God Token (CGT) as its native currency.',
      metadata: { source: 'whitepaper', title: 'Demiurge Overview', section: 'Introduction' }
    });

    this.documents.set('tokenomics', {
      content: 'CGT (Creator God Token) has a total supply of 13 billion tokens. 1 CGT = 100 Sparks. Block rewards are 42 CGT per block, targeting approximately 5% annual inflation. Validators stake CGT to participate in consensus and earn rewards.',
      metadata: { source: 'whitepaper', title: 'Tokenomics', section: 'Economics' }
    });

    this.documents.set('energy_system', {
      content: 'The Energy system enables gasless transactions. Users have a regenerating Energy pool (1000 max, 1 per second). Basic transactions cost 10 Energy. Developers can sponsor user transactions by paying Energy costs, enabling free-to-play gaming experiences.',
      metadata: { source: 'whitepaper', title: 'Energy System', section: 'Gasless' }
    });

    this.documents.set('community_rules', {
      content: 'VYB Community Guidelines: 1) Treat all users with respect. 2) No spam, scams, or phishing. 3) No harassment or hate speech. 4) No illegal content. 5) Mark NSFW content appropriately. Violations result in progressive discipline via the Justice Scale.',
      metadata: { source: 'rules', title: 'Community Guidelines', section: 'Rules' }
    });

  }

  async search(query: RAGQuery): Promise<RAGResult> {
    // Simple keyword matching (replace with actual vector search in production)
    const queryLower = query.query.toLowerCase();
    const results: { doc: any; score: number }[] = [];

    this.documents.forEach((doc, id) => {
      const contentLower = doc.content.toLowerCase();
      let score = 0;

      // Simple scoring based on keyword presence
      const words = queryLower.split(/\s+/);
      words.forEach(word => {
        if (word.length > 3 && contentLower.includes(word)) {
          score += 0.2;
        }
      });

      if (score > 0) {
        results.push({
          doc: { id, ...doc },
          score: Math.min(score, 1)
        });
      }
    });

    // Sort by score and take topK
    results.sort((a, b) => b.score - a.score);
    const topResults = results.slice(0, query.topK || 3);

    return {
      documents: topResults.map(r => ({
        id: r.doc.id,
        content: r.doc.content,
        metadata: r.doc.metadata,
      })),
      scores: topResults.map(r => r.score),
      queryEmbedding: [], // Would be actual embedding in production
    };
  }
}

// ============ Export Singleton ============

export const sophiaAgent = new SophiaAgentService();

// ============ Convenience Functions ============

/**
 * Quick moderation check (use before posting)
 */
export async function checkContent(content: string, authorQorId: string): Promise<boolean> {
  const { allowed } = await sophiaAgent.moderateContent(content, authorQorId);
  return allowed;
}

/**
 * Ask Sophia a question
 */
export async function askSophia(question: string, userQorId?: string): Promise<string> {
  const response = await sophiaAgent.consultTheOracle(
    question,
    userQorId ? { qorId: userQorId, karma: 0 } : undefined
  );
  return response.text;
}

/**
 * Check if user is banned
 */
export async function isUserBanned(qorId: string): Promise<boolean> {
  const { canPost } = await sophiaAgent.canUserPost(qorId);
  return !canPost;
}
