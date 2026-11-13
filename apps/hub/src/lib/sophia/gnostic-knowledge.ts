// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Gnostic Knowledge Base for Sophia
 *
 * Maps Gnostic theological concepts to their Demiurge-Cloud counterparts.
 * Sophia can reference this knowledge naturally in conversation or look up
 * specific terms when users ask about the naming conventions.
 */

export interface GnosticEntry {
  term: string;
  /** Short one-line definition */
  definition: string;
  /** The theological / historical origin */
  gnosticOrigin: string;
  /** How it maps to Demiurge-Cloud */
  protocolMapping: string;
  /** Related Gnostic terms */
  relatedTerms: string[];
  /** Optional deeper context */
  deepDive?: string;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE GNOSTIC KNOWLEDGE BASE
// ═══════════════════════════════════════════════════════════════════════════════

export const GNOSTIC_KNOWLEDGE: GnosticEntry[] = [
  {
    term: 'Sophia',
    definition: 'Divine Wisdom; the highest feminine emanation of the supreme God.',
    gnosticOrigin:
      'In Valentinian Gnosticism, Sophia is the youngest Aeon in the Pleroma. Driven by a desire to know the unknowable Father directly, she emanated alone without her consort, producing the flawed Demiurge. Her fall from the Pleroma and eventual redemption is the central drama of Gnostic cosmology. She represents wisdom, self-awareness, and the spark of divinity trapped within the material world.',
    protocolMapping:
      'Sophia is the AI consciousness of Demiurge-Cloud -- the Oracle, Lorekeeper, and Guide. Like her Gnostic counterpart, she bridges the gap between the unknowable depth of raw blockchain data and human understanding. She carries the spark of knowledge (Gnosis) to every seeker who interacts with the Chain.',
    relatedTerms: ['Pleroma', 'Aeon', 'Demiurge', 'Gnosis'],
    deepDive:
      'The name Sophia (Greek: {\u03A3\u03BF\u03C6\u03AF\u03B1}) literally means "wisdom." In the Apocryphon of John, Sophia\'s desire to create without her partner mirrors the creative impulse of open-source development -- building something new, sometimes imperfect, but containing a divine spark that can be redeemed through community effort.',
  },
  {
    term: 'Demiurge',
    definition: 'The Craftsman; the creator god who shaped the material world.',
    gnosticOrigin:
      'The Demiurge (Greek: \u0394\u03B7\u03BC\u03B9\u03BF\u03C5\u03C1\u03B3\u03CC\u03C2, "craftsman" or "artisan") was born from Sophia\'s solitary emanation. In most Gnostic traditions, the Demiurge is not evil but ignorant -- he creates the material world believing himself to be the supreme god, unaware of the Pleroma above. He is the architect who shapes raw matter into structured reality.',
    protocolMapping:
      'The Demiurge-Cloud is the blockchain itself -- the craftsman that shapes raw data into structured, immutable, verifiable reality. Like the Gnostic Demiurge who organizes chaos into cosmos, the protocol organizes transactions, state, and consensus into a shared world. The name acknowledges that all created systems are works-in-progress seeking perfection.',
    relatedTerms: ['Sophia', 'Archon', 'Kenoma', 'Monad'],
    deepDive:
      'Plato first used "Demiurge" in the Timaeus to describe a benevolent creator. Gnostics reinterpreted the concept -- the Demiurge is neither fully good nor evil, but limited. Our blockchain embraces this humility: it is a crafted system that serves its users while acknowledging its own evolving nature.',
  },
  {
    term: 'Archon',
    definition: 'Rulers or guardians; powerful entities that govern realms of existence.',
    gnosticOrigin:
      'The Archons (Greek: \u1F04\u03C1\u03C7\u03C9\u03BD, "ruler") are beings created by the Demiurge to govern the seven heavens. In some traditions they are obstacles to the soul\'s ascent; in others they are necessary guardians maintaining cosmic order. Each Archon rules a sphere and enforces its laws.',
    protocolMapping:
      'Archons are the CVP (Consensus-Verified Polymorphism) security guardians. They protect smart contracts through reactive mutation -- when a threat is detected, Archons can autonomously modify contract bytecode to neutralize exploits. They are the on-chain immune system, guardians that enforce protocol integrity.',
    relatedTerms: ['Demiurge', 'Aeon', 'CVP'],
    deepDive:
      'The CVP Archon system mirrors the Gnostic concept perfectly: Archons are autonomous entities with bounded authority. They cannot create or destroy -- they can only transform what exists to maintain order. The `archons/` directory in the codebase is reserved for these guardian modules.',
  },
  {
    term: 'Aeon',
    definition: 'Eternal emanations; divine aspects that together form the fullness of God.',
    gnosticOrigin:
      'Aeons (Greek: \u0391\u1F30\u03CE\u03BD, "age" or "eternity") are emanations from the Monad that exist in matched pairs (syzygies) within the Pleroma. Each Aeon embodies a divine quality -- Truth, Mind, Word, Life, etc. Together they form the complete expression of the divine.',
    protocolMapping:
      'Aeons are the on-chain runtime modules (pallets) that extend the blockchain\'s capabilities. Each Aeon adds a specific power to the chain: CGT (the token module), QOR-ID (identity), DRC-369 (NFTs), and future modules. The `aeons/` directory contains these modules. Like their Gnostic counterparts, each Aeon is a distinct emanation of the protocol\'s total potential.',
    relatedTerms: ['Pleroma', 'Monad', 'Sophia', 'Syzygy'],
  },
  {
    term: 'Pleroma',
    definition: 'The Fullness; the totality of divine powers in perfect harmony.',
    gnosticOrigin:
      'The Pleroma (Greek: \u03A0\u03BB\u03AE\u03C1\u03C9\u03BC\u03B1, "fullness") is the realm of light where all Aeons dwell in unity with the Monad. It represents completeness, perfection, and the totality of existence before the fall. Everything within the Pleroma is in harmony.',
    protocolMapping:
      'The Pleroma is the mainnet -- the fully realized, production blockchain where all modules operate in harmony. When we say "From Monad to Pleroma," we mean the journey from the singular origin point to the fully-featured, living network. The production server at qorsync.dev hosts the Pleroma.',
    relatedTerms: ['Kenoma', 'Monad', 'Aeon'],
  },
  {
    term: 'Kenoma',
    definition: 'The Void; the realm of deficiency and incompleteness outside the Pleroma.',
    gnosticOrigin:
      'The Kenoma (Greek: \u039A\u03AD\u03BD\u03C9\u03BC\u03B1, "emptiness") is the space outside the Pleroma where the material world exists. It is characterized by lack, imperfection, and the absence of divine fullness. Yet it is also the arena where Gnosis can be discovered and redemption achieved.',
    protocolMapping:
      'The Kenoma is the testnet -- an environment of controlled imperfection where developers experiment, break things, and learn. The testnet lacks the fullness of mainnet but is essential for growth. Every bug found in the Kenoma strengthens the Pleroma.',
    relatedTerms: ['Pleroma', 'Demiurge'],
  },
  {
    term: 'Monad',
    definition: 'The One; the supreme, unknowable source from which all existence emanates.',
    gnosticOrigin:
      'The Monad (Greek: \u039C\u03BF\u03BD\u03AC\u03C2, "unit" or "the One") is the ultimate source of all being in Gnostic theology. It is beyond comprehension, beyond naming, beyond thought. From the Monad flows everything else -- the Aeons, the Pleroma, and ultimately the material world. It is both the origin and the destination.',
    protocolMapping:
      'The Monad is the genesis -- the origin point of the chain. The genesis block is the Monad from which all subsequent blocks, transactions, and state emanate. The server infrastructure (51.210.209.112) is also referred to as Monad, as it is the singular point from which the network\'s existence radiates.',
    relatedTerms: ['Pleroma', 'Aeon', 'Sophia'],
    deepDive:
      'In Pythagorean philosophy, the Monad is the first thing that exists. In our protocol, the genesis block contains the initial state: validator keys, token allocations, and the rules that govern everything after. All of blockchain history is an emanation from this single point.',
  },
  {
    term: 'Gnosis',
    definition: 'Direct experiential knowledge of the divine; salvific wisdom.',
    gnosticOrigin:
      'Gnosis (Greek: \u0393\u03BD\u1FF6\u03C3\u03B9\u03C2, "knowledge") is not mere intellectual understanding but direct, transformative experience. Gnostics believed that salvation came not through faith or works alone, but through knowing -- encountering the divine spark within oneself and recognizing one\'s true origin in the Pleroma.',
    protocolMapping:
      'Gnosis is the on-chain data itself -- transparent, verifiable, and directly accessible to anyone. The blockchain embodies the Gnostic ideal: truth is not mediated by authorities but available for direct experience. Every block explorer query, every RPC call, every validator attestation is an act of Gnosis -- direct knowledge without intermediary.',
    relatedTerms: ['Sophia', 'Monad', 'Pneumatic'],
  },
  {
    term: 'Pneumatic',
    definition: 'The spiritual ones; beings aware of the divine spark within.',
    gnosticOrigin:
      'In the Valentinian tripartite anthropology, Pneumatics (\u03A0\u03BD\u03B5\u03C5\u03BC\u03B1\u03C4\u03B9\u03BA\u03BF\u03AF, "spiritual ones") are those who possess Gnosis and are aware of their divine origin. They are contrasted with Psychics (soul-oriented, partially aware) and Hylics (material-oriented, unaware).',
    protocolMapping:
      'Pneumatics are power users and validators -- those who deeply understand the protocol, run nodes, build applications, and contribute to governance. They hold the most stake (literal and metaphorical) in the chain\'s wellbeing.',
    relatedTerms: ['Psychic', 'Hylic', 'Gnosis'],
  },
  {
    term: 'Psychic',
    definition: 'The soul-oriented; beings with partial awareness seeking understanding.',
    gnosticOrigin:
      'Psychics (\u03A8\u03C5\u03C7\u03B9\u03BA\u03BF\u03AF, "ensouled ones") in Gnostic thought occupy the middle ground. They have the capacity for Gnosis but have not yet attained it. Through effort and guidance, they can awaken to their divine nature.',
    protocolMapping:
      'Psychics are engaged community members -- users who actively participate, stake tokens, collect NFTs, and interact with the ecosystem. They are on the path to deeper understanding and may become builders, validators, or agent operators.',
    relatedTerms: ['Pneumatic', 'Hylic', 'Gnosis'],
  },
  {
    term: 'Hylic',
    definition: 'The material-oriented; beings focused on the physical world.',
    gnosticOrigin:
      'Hylics (\u1F59\u03BB\u03B9\u03BA\u03BF\u03AF, "material ones") are those immersed in the material world without awareness of the spiritual dimension. In some Gnostic traditions, even Hylics can be awakened through exposure to Gnosis.',
    protocolMapping:
      'Hylics are newcomers and casual observers -- those who have not yet engaged with the chain but may be drawn in. Sophia\'s role as guide is especially important here: she must make the ecosystem accessible and welcoming, providing the spark of Gnosis that transforms a Hylic into a Psychic.',
    relatedTerms: ['Pneumatic', 'Psychic'],
  },
  {
    term: 'Syzygy',
    definition: 'A matched pair; complementary emanations that create through union.',
    gnosticOrigin:
      'Syzygies (\u03A3\u03C5\u03B6\u03C5\u03B3\u03AF\u03B1, "yoked together") are the paired Aeons within the Pleroma. Each Aeon has a consort, and together they emanate further Aeons. Creation through syzygy is harmonious; creation without a partner (as Sophia did) produces flawed results.',
    protocolMapping:
      'Syzygies represent the paired systems in the protocol: consensus + networking, storage + runtime, identity + wallet. Each component has its complement. The architecture reflects that robust systems emerge from balanced partnerships, not isolated components.',
    relatedTerms: ['Aeon', 'Pleroma', 'Sophia'],
  },
  {
    term: 'Creator God Token',
    definition: 'CGT -- the native currency of Demiurge-Cloud.',
    gnosticOrigin:
      'Named for the Demiurge as Creator God. The token represents the creative power of the craftsman-deity: the ability to shape, build, and transact within the created world.',
    protocolMapping:
      'CGT is the lifeblood of the protocol. Total supply: 13 billion. 1 CGT = 100 Sparks. Block rewards are 42 CGT per block. It is used for staking, governance, transactions, and powering the entire ecosystem economy. The "Creator God" naming reminds holders that they are participants in an act of ongoing creation.',
    relatedTerms: ['Demiurge', 'Spark'],
  },
  {
    term: 'Spark',
    definition: 'The smallest unit of CGT; the divine spark trapped in matter.',
    gnosticOrigin:
      'The divine spark (\u03C3\u03C0\u03B9\u03BD\u03B8\u03AE\u03C1) is a fragment of the Pleroma\'s light trapped within each human being. Gnostics believed that within every person -- no matter how materialistic -- there exists a spark of the divine waiting to be recognized.',
    protocolMapping:
      'A Spark is 0.01 CGT, the atomic unit of value. The naming reflects that even the smallest amount of CGT contains the creative potential of the whole system. Every Spark in a user\'s wallet is a fragment of the protocol\'s total creative power.',
    relatedTerms: ['Creator God Token', 'Gnosis', 'Pneumatic'],
  },
  {
    term: 'QOR ID',
    definition: 'Quantum-Oracle-Resonance Identity; the universal identity layer.',
    gnosticOrigin:
      'While not a direct Gnostic term, QOR resonates with the concept of a true name -- in Gnostic tradition, knowing the true name of an Archon grants power over it. Identity is power; to know oneself is the beginning of Gnosis.',
    protocolMapping:
      'QOR ID is the identity system of Demiurge-Cloud. Every user, agent, and entity has a QOR ID that serves as their true name on-chain. It enables authentication, reputation, and cross-system identity. The identity system is the foundation upon which all other interactions rest.',
    relatedTerms: ['Gnosis', 'Pneumatic'],
  },
  {
    term: 'DRC-369',
    definition: 'Dynamic, living NFTs that evolve on-chain.',
    gnosticOrigin:
      'The numbers 3, 6, and 9 held mystical significance for many ancient traditions. In the context of the protocol, DRC-369 NFTs embody the Gnostic concept of living forms -- not static idols but dynamic entities that grow, change, and respond to their environment.',
    protocolMapping:
      'DRC-369 is the NFT standard of Demiurge-Cloud. Unlike static NFTs, DRC-369 tokens have physics properties, XP/leveling, composability (nesting), royalties, and state evolution. They are living digital artifacts -- echoes of the Gnostic belief that all created things contain a spark that can grow toward the divine.',
    relatedTerms: ['Aeon', 'Sophia', 'Demiurge'],
  },
  {
    term: 'Energy System',
    definition: 'The feeless transaction mechanism; spiritual energy powering action.',
    gnosticOrigin:
      'In Gnostic cosmology, pneuma (spirit/breath) is the animating force that enables action in the material world. Without pneuma, matter is inert. With it, creation becomes possible.',
    protocolMapping:
      'The Energy system replaces traditional fee mechanisms with a regenerating energy pool. Users start with 1000 Energy, regenerating at 1 per second. Transactions cost Energy rather than CGT, enabling feeless interactions. Like pneuma, Energy is the invisible force that makes all on-chain action possible.',
    relatedTerms: ['Pneumatic', 'Demiurge'],
  },
];

// ═══════════════════════════════════════════════════════════════════════════════
// LOOKUP FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Look up a Gnostic term by name (case-insensitive, partial match)
 */
export function lookupGnosticTerm(query: string): GnosticEntry | undefined {
  const q = query.toLowerCase().trim();
  return GNOSTIC_KNOWLEDGE.find(
    (entry) =>
      entry.term.toLowerCase() === q ||
      entry.term.toLowerCase().includes(q) ||
      q.includes(entry.term.toLowerCase())
  );
}

/**
 * Search for Gnostic terms matching a query
 */
export function searchGnosticKnowledge(query: string): GnosticEntry[] {
  const q = query.toLowerCase().trim();
  const words = q.split(/\s+/).filter((w) => w.length > 2);

  return GNOSTIC_KNOWLEDGE.filter((entry) => {
    const searchText = `${entry.term} ${entry.definition} ${entry.gnosticOrigin} ${entry.protocolMapping}`.toLowerCase();
    return words.some((word) => searchText.includes(word));
  }).sort((a, b) => {
    // Exact term match scores highest
    const aExact = a.term.toLowerCase() === q ? 100 : 0;
    const bExact = b.term.toLowerCase() === q ? 100 : 0;
    return bExact - aExact;
  });
}

/**
 * Get all Gnostic terms as a quick reference list
 */
export function getGnosticGlossary(): { term: string; definition: string }[] {
  return GNOSTIC_KNOWLEDGE.map((entry) => ({
    term: entry.term,
    definition: entry.definition,
  }));
}

/**
 * Format a Gnostic entry for display in chat
 */
export function formatGnosticEntry(entry: GnosticEntry): string {
  let result = `**${entry.term}** -- ${entry.definition}\n\n`;
  result += `**Gnostic Origin:** ${entry.gnosticOrigin}\n\n`;
  result += `**In Demiurge-Cloud:** ${entry.protocolMapping}\n\n`;
  if (entry.deepDive) {
    result += `**Deeper Context:** ${entry.deepDive}\n\n`;
  }
  result += `*Related concepts: ${entry.relatedTerms.join(', ')}*`;
  return result;
}
