// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import { useState, useEffect, useRef } from 'react';
import { demiurgeRpc } from '@/lib/demiurge-rpc';

interface ChainEvent {
  id: string;
  type: 'block' | 'transaction' | 'governance' | 'reward' | 'nft_mint' | 'announcement';
  title: string;
  description: string;
  timestamp: Date;
  link?: string;
  highlight?: boolean;
}

export function OnChainFeedWidget() {
  const [events, setEvents] = useState<ChainEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [blockHeight, setBlockHeight] = useState<number | null>(null);
  const [era, setEra] = useState<number | null>(null);
  const feedRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadChainData();
    const interval = setInterval(loadChainData, 10000); // Refresh every 10s
    return () => clearInterval(interval);
  }, []);

  const loadChainData = async () => {
    try {
      // Get real chain data
      const chainInfo = await demiurgeRpc.getChainInfo();
      if (chainInfo && chainInfo.connected) {
        setBlockHeight(chainInfo.blockHeight);
        setEra(chainInfo.currentEra);
        
        // Try to get real events
        try {
          const recentEvents = await demiurgeRpc.getRecentEvents(5);
          if (recentEvents.length > 0) {
            const formattedEvents: ChainEvent[] = recentEvents.map(e => ({
              id: `${e.type}-${e.blockNumber}`,
              type: e.type as any,
              title: formatEventTitle(e),
              description: formatEventDescription(e),
              timestamp: new Date(e.timestamp * 1000),
            }));
            setEvents(prev => [...formattedEvents, ...prev.filter(p => p.type === 'announcement')].slice(0, 10));
            setLoading(false);
            return;
          }
        } catch (eventError) {
          // Events endpoint might not be available yet
        }
      }
    } catch (error) {
      console.warn('Using mock chain data');
    }

    // Show real chain activity - just current block for now
    const realEvents: ChainEvent[] = [];
    
    // Add current block as most recent event
    if (blockHeight) {
      realEvents.push({
        id: `block-${blockHeight}`,
        type: 'block',
        title: `Block #${blockHeight}`,
        description: 'Latest block produced',
        timestamp: new Date(),
      });
    }
    
    // Add era info if available
    if (era) {
      realEvents.push({
        id: `era-${era}`,
        type: 'reward',
        title: `Era ${era}`,
        description: 'Current staking era',
        timestamp: new Date(Date.now() - 60000),
      });
    }
    
    // Add platform announcement
    realEvents.push({
      id: 'announce-platform',
      type: 'announcement',
      title: 'Demiurge-Cloud Live!',
      description: 'Next-gen blockchain for gaming and AI',
      timestamp: new Date(Date.now() - 3600000),
      highlight: true,
    });

    setEvents(realEvents);
    setLoading(false);
  };
  
  const formatEventTitle = (event: any): string => {
    switch (event.type) {
      case 'block': return `Block #${event.blockNumber}`;
      case 'transaction': return `Transfer`;
      case 'nft_mint': return `NFT Minted`;
      case 'stake': return `Stake Update`;
      case 'reward': return `Rewards Distributed`;
      default: return event.type;
    }
  };
  
  const formatEventDescription = (event: any): string => {
    switch (event.type) {
      case 'block': return 'New block produced';
      case 'transaction': return `${(event.data?.amount || 0) / 100} CGT transferred`;
      case 'nft_mint': return `New DRC-369 asset created`;
      case 'stake': return `${(event.data?.amount || 0) / 100} CGT staked`;
      case 'reward': return `Era rewards distributed`;
      default: return '';
    }
  };

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'block': return '⛓️';
      case 'transaction': return '💸';
      case 'governance': return '🗳️';
      case 'reward': return '🎁';
      case 'nft_mint': return '🎨';
      case 'announcement': return '📢';
      default: return '📌';
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case 'block': return 'text-neon-cyan';
      case 'transaction': return 'text-neon-green';
      case 'governance': return 'text-demiurge-violet';
      case 'reward': return 'text-demiurge-gold';
      case 'nft_mint': return 'text-neon-magenta';
      case 'announcement': return 'text-white';
      default: return 'text-gray-400';
    }
  };

  const formatTime = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);

    if (seconds < 60) return `${seconds}s ago`;
    if (minutes < 60) return `${minutes}m ago`;
    return `${hours}h ago`;
  };

  return (
    <div className="glass-panel rounded-xl p-6 border border-demiurge-violet/20 relative overflow-hidden">
      {/* Glow effect */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-demiurge-violet/10 rounded-full blur-3xl" />
      
      <div className="relative z-10">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-grunge text-demiurge-violet">⛓️ Chain Activity</h3>
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
            <span className="text-xs text-gray-400">Live</span>
          </div>
        </div>

        {/* Chain Stats Bar */}
        <div className="flex items-center justify-between mb-4 p-3 bg-black/30 rounded-lg">
          <div className="text-center">
            <div className="text-xs text-gray-400">Block</div>
            <div className="font-mono text-neon-cyan text-sm">
              #{blockHeight?.toLocaleString() || '---'}
            </div>
          </div>
          <div className="h-6 w-px bg-white/10" />
          <div className="text-center">
            <div className="text-xs text-gray-400">Era</div>
            <div className="font-mono text-demiurge-gold text-sm">
              {era || '---'}
            </div>
          </div>
          <div className="h-6 w-px bg-white/10" />
          <div className="text-center">
            <div className="text-xs text-gray-400">Status</div>
            <div className="font-mono text-neon-green text-sm">
              {blockHeight ? 'Live' : 'Connecting...'}
            </div>
          </div>
        </div>

        {/* Event Feed */}
        <div 
          ref={feedRef}
          className="space-y-2 max-h-64 overflow-y-auto scrollbar-thin scrollbar-thumb-white/10"
        >
          {loading ? (
            <div className="space-y-2 animate-pulse">
              {[1, 2, 3].map(i => (
                <div key={i} className="h-12 bg-white/5 rounded-lg" />
              ))}
            </div>
          ) : (
            events.map((event) => (
              <div
                key={event.id}
                className={`flex items-start gap-3 p-2 rounded-lg transition-all ${
                  event.highlight 
                    ? 'bg-demiurge-violet/10 border border-demiurge-violet/30' 
                    : 'hover:bg-white/5'
                }`}
              >
                <span className="text-lg mt-0.5">{getEventIcon(event.type)}</span>
                <div className="flex-1 min-w-0">
                  <div className={`text-sm font-medium ${getEventColor(event.type)}`}>
                    {event.title}
                  </div>
                  <div className="text-xs text-gray-400 truncate">
                    {event.description}
                  </div>
                </div>
                <div className="text-xs text-gray-500 whitespace-nowrap">
                  {formatTime(event.timestamp)}
                </div>
              </div>
            ))
          )}
        </div>

        {/* View More */}
        <a
          href="https://qorsync.dev"
          target="_blank"
          rel="noopener noreferrer"
          className="block mt-4 text-center text-xs text-demiurge-violet hover:underline"
        >
          View Block Explorer →
        </a>
      </div>
    </div>
  );
}
