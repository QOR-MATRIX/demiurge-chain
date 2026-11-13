// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * @demiurge/drc369-sdk
 * 
 * TypeScript SDK for DRC-369 Evolving NFTs with CVP Protection
 * 
 * @example
 * ```typescript
 * import { Drc369Client } from '@demiurge/drc369-sdk';
 * 
 * const client = new Drc369Client({
 *   baseUrl: 'http://127.0.0.1:3000/api/v1/drc369',
 *   token: 'your-auth-token',
 * });
 * 
 * // Mint an NFT
 * const result = await client.mint({
 *   name: 'My Evolving NFT',
 *   description: 'Protected by CVP',
 *   resources: [{
 *     type: 'image',
 *     uri: 'ipfs://Qm...',
 *     priority: 1,
 *     contextTags: ['profile'],
 *   }],
 * });
 * 
 * // Add XP
 * await client.addXp({
 *   nftId: result.data!.id,
 *   amount: 100,
 *   reason: 'Quest completed',
 * });
 * 
 * // Check CVP status
 * const cvpStatus = await client.getCvpStatus(result.data!.id);
 * console.log('Protected:', cvpStatus.isProtected);
 * console.log('Current epoch:', cvpStatus.currentEpoch);
 * ```
 * 
 * @example React Hooks
 * ```tsx
 * import { Drc369Provider, useNft, useMint, useCvpStatus } from '@demiurge/drc369-sdk/react';
 * 
 * function App() {
 *   return (
 *     <Drc369Provider config={{ baseUrl: '...' }}>
 *       <MyComponent />
 *     </Drc369Provider>
 *   );
 * }
 * 
 * function MyComponent() {
 *   const { nft, loading } = useNft('nft-id');
 *   const { status } = useCvpStatus('nft-id');
 *   const { mint, loading: minting } = useMint();
 *   
 *   if (loading) return <div>Loading...</div>;
 *   
 *   return (
 *     <div>
 *       <h1>{nft?.name}</h1>
 *       <p>Level: {nft?.level}</p>
 *       <p>CVP Protected: {status?.isProtected ? 'Yes' : 'No'}</p>
 *     </div>
 *   );
 * }
 * ```
 * 
 * @packageDocumentation
 */

// Core client
export { Drc369Client, drc369 } from './client';
export type { Drc369ClientConfig, Drc369ClientEvents } from './client';

// Types
export * from './types';

// Default export
export { drc369 as default } from './client';
