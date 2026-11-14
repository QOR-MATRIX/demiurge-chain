// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * IPFS Upload API Route
 * Server-side uploads to protect API keys
 * Falls back to local storage if IPFS not configured
 */

import { NextRequest, NextResponse } from 'next/server';
import { writeFile, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import path from 'path';
import crypto from 'crypto';

// Local storage directory for media when IPFS isn't configured
const LOCAL_MEDIA_DIR = process.env.MEDIA_STORAGE_PATH || '/tmp/demiurge-media';

export async function POST(request: NextRequest) {
  try {
    const formData = await request.formData();
    const file = formData.get('file') as File | null;

    if (!file) {
      return NextResponse.json(
        { success: false, error: 'No file provided' },
        { status: 400 }
      );
    }

    // Check file size (1.5GB limit for music/video files)
    const maxSizeBytes = 1.5 * 1024 * 1024 * 1024; // 1.5 GB
    if (file.size > maxSizeBytes) {
      return NextResponse.json(
        { success: false, error: 'File too large (max 1.5GB)' },
        { status: 400 }
      );
    }

    // Try Pinata first
    const pinataApiKey = process.env.PINATA_API_KEY;
    const pinataSecretKey = process.env.PINATA_SECRET_KEY;

    if (pinataApiKey && pinataSecretKey) {
      try {
        const pinataFormData = new FormData();
        pinataFormData.append('file', file);

        const response = await fetch('https://api.pinata.cloud/pinning/pinFileToIPFS', {
          method: 'POST',
          headers: {
            'pinata_api_key': pinataApiKey,
            'pinata_secret_api_key': pinataSecretKey,
          },
          body: pinataFormData,
        });

        if (response.ok) {
          const data = await response.json();
          return NextResponse.json({
            success: true,
            cid: data.IpfsHash,
            uri: `ipfs://${data.IpfsHash}`,
            size: data.PinSize,
            gateway: `https://gateway.pinata.cloud/ipfs/${data.IpfsHash}`,
          });
        }
      } catch (pinataError) {
        console.warn('Pinata upload failed:', pinataError);
      }
    }

    // Try Infura as fallback
    const infuraProjectId = process.env.INFURA_IPFS_PROJECT_ID;
    const infuraProjectSecret = process.env.INFURA_IPFS_PROJECT_SECRET;

    if (infuraProjectId && infuraProjectSecret) {
      try {
        const infuraFormData = new FormData();
        infuraFormData.append('file', file);

        const auth = Buffer.from(`${infuraProjectId}:${infuraProjectSecret}`).toString('base64');

        const response = await fetch('https://ipfs.infura.io:5001/api/v0/add', {
          method: 'POST',
          headers: {
            'Authorization': `Basic ${auth}`,
          },
          body: infuraFormData,
        });

        if (response.ok) {
          const data = await response.json();
          return NextResponse.json({
            success: true,
            cid: data.Hash,
            uri: `ipfs://${data.Hash}`,
            size: parseInt(data.Size),
            gateway: `https://ipfs.infura.io/ipfs/${data.Hash}`,
          });
        }
      } catch (infuraError) {
        console.warn('Infura upload failed:', infuraError);
      }
    }

    // Try web3.storage
    const web3StorageToken = process.env.WEB3_STORAGE_TOKEN;

    if (web3StorageToken) {
      try {
        const response = await fetch('https://api.web3.storage/upload', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${web3StorageToken}`,
            'X-NAME': file.name,
          },
          body: file,
        });

        if (response.ok) {
          const data = await response.json();
          return NextResponse.json({
            success: true,
            cid: data.cid,
            uri: `ipfs://${data.cid}`,
            gateway: `https://w3s.link/ipfs/${data.cid}`,
          });
        }
      } catch (web3Error) {
        console.warn('Web3.storage upload failed:', web3Error);
      }
    }

    // Fallback: Local file storage
    // This is temporary storage until IPFS is configured
    console.log('[IPFS] No IPFS provider configured, using local storage fallback');
    
    try {
      // Ensure directory exists
      if (!existsSync(LOCAL_MEDIA_DIR)) {
        await mkdir(LOCAL_MEDIA_DIR, { recursive: true });
      }

      // Generate unique filename
      const ext = path.extname(file.name) || '';
      const hash = crypto.randomBytes(16).toString('hex');
      const filename = `${hash}${ext}`;
      const filepath = path.join(LOCAL_MEDIA_DIR, filename);

      // Write file
      const buffer = Buffer.from(await file.arrayBuffer());
      await writeFile(filepath, buffer);

      // Return local URL (served by static route or nginx)
      const baseUrl = process.env.NEXT_PUBLIC_API_URL || 'https://qorsync.dev';
      const localUrl = `${baseUrl}/api/media/${filename}`;

      return NextResponse.json({
        success: true,
        cid: hash, // Use hash as pseudo-CID
        uri: `local://${hash}`,
        size: file.size,
        gateway: localUrl,
        isLocal: true, // Flag to indicate this is local storage
        warning: 'Stored locally. File will be migrated to IPFS when configured.',
      });
    } catch (localError: any) {
      console.error('Local storage fallback failed:', localError);
      return NextResponse.json(
        { 
          success: false, 
          error: 'Media storage not available. Please contact support.',
          details: 'Neither IPFS nor local storage is working.'
        },
        { status: 503 }
      );
    }
  } catch (error: any) {
    console.error('IPFS upload error:', error);
    return NextResponse.json(
      { success: false, error: error.message || 'Upload failed' },
      { status: 500 }
    );
  }
}

// Also support pinning JSON metadata
export async function PUT(request: NextRequest) {
  try {
    const metadata = await request.json();

    const pinataApiKey = process.env.PINATA_API_KEY;
    const pinataSecretKey = process.env.PINATA_SECRET_KEY;

    if (!pinataApiKey || !pinataSecretKey) {
      return NextResponse.json(
        { success: false, error: 'Pinata not configured' },
        { status: 503 }
      );
    }

    const response = await fetch('https://api.pinata.cloud/pinning/pinJSONToIPFS', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'pinata_api_key': pinataApiKey,
        'pinata_secret_api_key': pinataSecretKey,
      },
      body: JSON.stringify({
        pinataContent: metadata,
        pinataMetadata: {
          name: metadata.name || 'nft-metadata',
        },
      }),
    });

    if (!response.ok) {
      throw new Error('Pinata JSON upload failed');
    }

    const data = await response.json();
    return NextResponse.json({
      success: true,
      cid: data.IpfsHash,
      uri: `ipfs://${data.IpfsHash}`,
      gateway: `https://gateway.pinata.cloud/ipfs/${data.IpfsHash}`,
    });
  } catch (error: any) {
    console.error('IPFS metadata upload error:', error);
    return NextResponse.json(
      { success: false, error: error.message || 'Metadata upload failed' },
      { status: 500 }
    );
  }
}
