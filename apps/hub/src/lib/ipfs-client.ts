// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * IPFS Client for Demiurge
 * 
 * Handles media uploads for NFT minting and decentralized storage.
 * Supports multiple IPFS gateways with fallback.
 */

export interface IPFSUploadResult {
  success: boolean;
  cid?: string;
  uri?: string;
  size?: number;
  error?: string;
  gatewayUrl?: string; // Direct HTTP URL for the uploaded file
}

export interface NFTMetadata {
  name: string;
  description: string;
  image: string;  // IPFS URI
  external_url?: string;
  attributes: Array<{
    trait_type: string;
    value: string | number;
    display_type?: 'number' | 'date' | 'boost_percentage';
  }>;
  // DRC-369 specific
  drc369?: {
    version: '1.0';
    creator: string;
    royalty_bps: number;
    collection?: string;
  };
}

// IPFS Gateway configurations
const IPFS_GATEWAYS = {
  // Primary: Pinata (requires API key)
  pinata: {
    api: 'https://api.pinata.cloud',
    gateway: 'https://gateway.pinata.cloud/ipfs/',
  },
  // Fallback: Infura (requires project ID)
  infura: {
    api: 'https://ipfs.infura.io:5001/api/v0',
    gateway: 'https://ipfs.infura.io/ipfs/',
  },
  // Public fallback (for reading only)
  public: {
    gateway: 'https://ipfs.io/ipfs/',
  },
  // Demiurge hosted gateway (if available)
  demiurge: {
    api: 'http://127.0.0.1:5001/api/v0',
    gateway: 'http://127.0.0.1:5001/ipfs/',
  },
};

class IPFSClient {
  private pinataApiKey: string | null = null;
  private pinataSecretKey: string | null = null;
  private infuraProjectId: string | null = null;
  private infuraProjectSecret: string | null = null;
  private preferredGateway: keyof typeof IPFS_GATEWAYS = 'pinata';

  constructor() {
    // Load from environment
    if (typeof window === 'undefined') {
      // Server-side
      this.pinataApiKey = process.env.PINATA_API_KEY || null;
      this.pinataSecretKey = process.env.PINATA_SECRET_KEY || null;
      this.infuraProjectId = process.env.INFURA_IPFS_PROJECT_ID || null;
      this.infuraProjectSecret = process.env.INFURA_IPFS_PROJECT_SECRET || null;
    } else {
      // Client-side: pinning credentials are deliberately NOT read here.
      // Anything under NEXT_PUBLIC_* is inlined into the browser bundle and
      // would hand every visitor write access to the IPFS account.
      // Browser uploads must go through the server-side /api/ipfs route.
      this.pinataApiKey = null;
      this.pinataSecretKey = null;
    }

    // Determine preferred gateway based on available credentials
    if (this.pinataApiKey) {
      this.preferredGateway = 'pinata';
    } else if (this.infuraProjectId) {
      this.preferredGateway = 'infura';
    } else {
      this.preferredGateway = 'demiurge';
    }
  }

  /**
   * Upload a file to IPFS
   */
  async uploadFile(file: File | Blob, filename?: string): Promise<IPFSUploadResult> {
    const formData = new FormData();
    formData.append('file', file, filename || 'upload');

    // Try Pinata first
    if (this.pinataApiKey) {
      try {
        const result = await this.uploadToPinata(formData);
        if (result.success) return result;
      } catch (error) {
        console.warn('Pinata upload failed, trying fallback:', error);
      }
    }

    // Try Demiurge gateway
    try {
      const result = await this.uploadToDemiurge(formData);
      if (result.success) return result;
    } catch (error) {
      console.warn('Demiurge IPFS upload failed:', error);
    }

    // Try server-side API route as last resort
    try {
      const result = await this.uploadViaApi(file, filename);
      if (result.success) return result;
    } catch (error) {
      console.error('All IPFS uploads failed:', error);
    }

    return {
      success: false,
      error: 'Failed to upload to IPFS. Please try again.',
    };
  }

  /**
   * Upload to Pinata
   */
  private async uploadToPinata(formData: FormData): Promise<IPFSUploadResult> {
    const response = await fetch(`${IPFS_GATEWAYS.pinata.api}/pinning/pinFileToIPFS`, {
      method: 'POST',
      headers: {
        'pinata_api_key': this.pinataApiKey!,
        'pinata_secret_api_key': this.pinataSecretKey || '',
      },
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`Pinata upload failed: ${response.status}`);
    }

    const data = await response.json();
    return {
      success: true,
      cid: data.IpfsHash,
      uri: `ipfs://${data.IpfsHash}`,
      size: data.PinSize,
    };
  }

  /**
   * Upload to Demiurge IPFS gateway
   */
  private async uploadToDemiurge(formData: FormData): Promise<IPFSUploadResult> {
    const response = await fetch(`${IPFS_GATEWAYS.demiurge.api}/add`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`Demiurge IPFS upload failed: ${response.status}`);
    }

    const data = await response.json();
    return {
      success: true,
      cid: data.Hash,
      uri: `ipfs://${data.Hash}`,
      size: data.Size,
    };
  }

  /**
   * Upload via our API route (server-side upload)
   */
  private async uploadViaApi(file: File | Blob, filename?: string): Promise<IPFSUploadResult> {
    const formData = new FormData();
    formData.append('file', file, filename || 'upload');

    const response = await fetch('/api/ipfs/upload', {
      method: 'POST',
      body: formData,
    });

    const data = await response.json();
    
    if (!response.ok || !data.success) {
      throw new Error(data.error || 'API upload failed');
    }

    // If gateway URL is provided, use it directly
    if (data.gateway) {
      return {
        success: true,
        cid: data.cid,
        uri: data.uri,
        size: data.size,
        // Store gateway URL for direct access
        gatewayUrl: data.gateway,
      };
    }

    return data;
  }

  /**
   * Upload JSON metadata to IPFS
   */
  async uploadMetadata(metadata: NFTMetadata): Promise<IPFSUploadResult> {
    const jsonBlob = new Blob([JSON.stringify(metadata, null, 2)], {
      type: 'application/json',
    });
    return this.uploadFile(jsonBlob, 'metadata.json');
  }

  /**
   * Upload media and create NFT metadata in one operation
   */
  async uploadNFT(
    mediaFile: File,
    metadata: Omit<NFTMetadata, 'image'>
  ): Promise<{
    success: boolean;
    mediaUri?: string;
    metadataUri?: string;
    error?: string;
  }> {
    // 1. Upload media file
    const mediaResult = await this.uploadFile(mediaFile, mediaFile.name);
    if (!mediaResult.success) {
      return {
        success: false,
        error: `Media upload failed: ${mediaResult.error}`,
      };
    }

    // 2. Create and upload metadata with media URI
    const fullMetadata: NFTMetadata = {
      ...metadata,
      image: mediaResult.uri!,
    };

    const metadataResult = await this.uploadMetadata(fullMetadata);
    if (!metadataResult.success) {
      return {
        success: false,
        error: `Metadata upload failed: ${metadataResult.error}`,
      };
    }

    return {
      success: true,
      mediaUri: mediaResult.uri,
      metadataUri: metadataResult.uri,
    };
  }

  /**
   * Convert IPFS URI to HTTP gateway URL
   */
  toGatewayUrl(ipfsUri: string, gateway: keyof typeof IPFS_GATEWAYS = 'public'): string {
    // Handle local:// URIs (fallback storage)
    if (ipfsUri.startsWith('local://')) {
      const hash = ipfsUri.replace('local://', '');
      const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'https://qorsync.dev';
      return `${baseUrl}/api/media/${hash}`;
    }
    
    if (!ipfsUri.startsWith('ipfs://')) {
      return ipfsUri; // Already a regular URL
    }
    const cid = ipfsUri.replace('ipfs://', '');
    return `${IPFS_GATEWAYS[gateway].gateway}${cid}`;
  }

  /**
   * Check if a CID exists on IPFS
   */
  async exists(cid: string): Promise<boolean> {
    try {
      const response = await fetch(this.toGatewayUrl(`ipfs://${cid}`), {
        method: 'HEAD',
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Fetch content from IPFS
   */
  async fetch(ipfsUri: string): Promise<Response> {
    const url = this.toGatewayUrl(ipfsUri);
    return fetch(url);
  }

  /**
   * Fetch and parse JSON from IPFS
   */
  async fetchJson<T = any>(ipfsUri: string): Promise<T> {
    const response = await this.fetch(ipfsUri);
    if (!response.ok) {
      throw new Error(`Failed to fetch from IPFS: ${response.status}`);
    }
    return response.json();
  }
}

// Export singleton instance
export const ipfsClient = new IPFSClient();

// Export convenience functions
export const uploadToIPFS = (file: File | Blob, filename?: string) =>
  ipfsClient.uploadFile(file, filename);

export const uploadNFTToIPFS = (
  mediaFile: File,
  metadata: Omit<NFTMetadata, 'image'>
) => ipfsClient.uploadNFT(mediaFile, metadata);

export const ipfsToHttp = (uri: string) => ipfsClient.toGatewayUrl(uri);
