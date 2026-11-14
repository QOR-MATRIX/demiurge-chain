// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * VYB Social Platform Service
 * Handles all VYB data operations and blockchain interactions
 */

import { demiurgeRpc } from '../demiurge-rpc';
import { ipfsClient, ipfsToHttp } from '../ipfs-client';
import type {
  VYBProfile,
  FeedItem,
  Conversation,
  Message,
  GalleryItem,
  ServiceListing,
  Notification,
  ProfileTheme,
  MediaItem,
} from './types';

// ============ Media Upload Types ============

interface MediaUploadResult {
  url: string;
  ipfsUri: string;
  artworkUrl?: string;
  artworkIpfsUri?: string;
  galleryItem?: GalleryItem;
}

interface MintMediaOptions {
  mediaUrl: string;
  artworkUrl?: string;
  name: string;
  description: string;
  royaltyPercent: number;
  mediaType: 'image' | 'video' | 'audio';
}

// Default profile theme
const DEFAULT_THEME: ProfileTheme = {
  primaryColor: '#00f5ff',
  secondaryColor: '#bf00ff',
  backgroundColor: '#0a0a0f',
  fontStyle: 'modern',
  layoutStyle: 'classic',
  musicEnabled: true,
};

// Profile cache for session persistence (not mock data)
const profileCache: Map<string, VYBProfile> = new Map();

class VYBService {
  private currentUser: string | null = null;

  setCurrentUser(qorId: string) {
    this.currentUser = qorId;
  }

  getCurrentUser(): string | null {
    return this.currentUser;
  }

  // ============ Profile Methods ============

  async getProfile(qorId: string): Promise<VYBProfile | null> {
    // Check cache first
    if (profileCache.has(qorId)) {
      return profileCache.get(qorId)!;
    }

    try {
      // Fetch from VYB profile API
      const response = await fetch(`/api/vyb/profile/${encodeURIComponent(qorId)}`);
      if (response.ok) {
        const { profile: dbProfile } = await response.json();
        if (dbProfile) {
          const profile: VYBProfile = {
            qorId,
            walletAddress: '',
            displayName: dbProfile.display_name || qorId.split('#')[0] || 'User',
            bio: dbProfile.bio || '',
            avatar: dbProfile.avatar_url || undefined,
            coverImage: dbProfile.banner_url || undefined,
            role: 'user',
            badges: [],
            stats: {
              followers: 0,
              following: 0,
              posts: 0,
              nftsOwned: 0,
              nftsCreated: 0,
              cgtEarned: 0,
              gamesPlayed: 0,
              achievementsUnlocked: 0,
            },
            theme: {
              ...DEFAULT_THEME,
              ...(dbProfile.theme_json || {}),
              bannerImage: dbProfile.banner_url || undefined,
              musicFile: dbProfile.music_url || undefined,
            },
            createdAt: new Date(dbProfile.created_at || Date.now()),
            lastActive: new Date(dbProfile.updated_at || Date.now()),
            isVerified: false,
            socialLinks: dbProfile.social_links || {},
          };
          profileCache.set(qorId, profile);
          return profile;
        }
      }
    } catch (error) {
      console.warn('Failed to fetch profile from API, using defaults:', error);
    }

    // Fallback: return a default profile with zero stats
    const profile: VYBProfile = {
      qorId,
      walletAddress: '',
      displayName: qorId.split('#')[0] || 'User',
      bio: '',
      role: 'user',
      badges: [],
      stats: {
        followers: 0,
        following: 0,
        posts: 0,
        nftsOwned: 0,
        nftsCreated: 0,
        cgtEarned: 0,
        gamesPlayed: 0,
        achievementsUnlocked: 0,
      },
      theme: DEFAULT_THEME,
      createdAt: new Date(),
      lastActive: new Date(),
      isVerified: false,
      socialLinks: {},
    };

    profileCache.set(qorId, profile);
    return profile;
  }

  async updateProfile(qorId: string, updates: Partial<VYBProfile>): Promise<VYBProfile> {
    const existing = await this.getProfile(qorId);
    if (!existing) throw new Error('Profile not found');

    const updated = { ...existing, ...updates };
    profileCache.set(qorId, updated);

    // Persist to backend API
    try {
      await fetch('/api/vyb/profile', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-qor-id': qorId,
        },
        body: JSON.stringify({
          display_name: updated.displayName,
          bio: updated.bio,
          avatar_url: updated.avatar,
          banner_url: updated.coverImage || updated.theme.bannerImage,
          music_url: updated.theme.musicFile || updated.theme.profileSong,
          theme_json: updated.theme,
          social_links: updated.socialLinks,
        }),
      });
    } catch (error) {
      console.warn('Failed to persist profile to API:', error);
    }

    return updated;
  }

  async updateTheme(qorId: string, theme: Partial<ProfileTheme>): Promise<ProfileTheme> {
    const profile = await this.getProfile(qorId);
    if (!profile) throw new Error('Profile not found');

    const updatedTheme = { ...profile.theme, ...theme };
    await this.updateProfile(qorId, { theme: updatedTheme });
    return updatedTheme;
  }

  async followUser(targetQorId: string): Promise<boolean> {
    if (!this.currentUser) throw new Error('Not logged in');
    try {
      const response = await fetch(`/api/vyb/follow`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'x-qor-id': this.currentUser },
        body: JSON.stringify({ targetQorId }),
      });
      return response.ok;
    } catch {
      console.warn('Follow service unavailable');
      return false;
    }
  }

  async unfollowUser(targetQorId: string): Promise<boolean> {
    if (!this.currentUser) throw new Error('Not logged in');
    try {
      const response = await fetch(`/api/vyb/follow`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json', 'x-qor-id': this.currentUser },
        body: JSON.stringify({ targetQorId }),
      });
      return response.ok;
    } catch {
      console.warn('Unfollow service unavailable');
      return false;
    }
  }

  // ============ Feed Methods ============

  async getFeed(options?: {
    type?: 'global' | 'following' | 'profile';
    userId?: string;
    limit?: number;
    offset?: number;
  }): Promise<FeedItem[]> {
    try {
      // TODO: Fetch from real API when available
      // const response = await fetch(`/api/vyb/feed?type=${options?.type}&limit=${options?.limit || 20}`);
      // return await response.json();
      return [];
    } catch (error) {
      console.warn('Failed to fetch feed:', error);
      return [];
    }
  }

  async createPost(content: {
    text?: string;
    media?: string[];
    visibility?: 'public' | 'followers' | 'private';
  }): Promise<FeedItem> {
    if (!this.currentUser) throw new Error('Not logged in');

    // TODO: Submit to backend API when available
    const post: FeedItem = {
      id: `post_${Date.now()}`,
      type: 'post',
      author: {
        qorId: this.currentUser,
        displayName: this.currentUser.split('#')[0] || 'User',
        isVerified: false,
        role: 'user',
      },
      content: {
        text: content.text,
        media: content.media?.map((url, i) => ({
          id: `media_${Date.now()}_${i}`,
          type: 'image' as const,
          url,
          isMinted: false,
          expiresAt: new Date(Date.now() + 90 * 24 * 60 * 60 * 1000), // 3 months
        })),
      },
      timestamp: new Date(),
      likes: 0,
      comments: 0,
      tips: 0,
      tipsAmount: 0,
      isLiked: false,
      isTipped: false,
      visibility: content.visibility || 'public',
    };

    return post;
  }

  async likePost(postId: string): Promise<boolean> {
    if (!this.currentUser) throw new Error('Not logged in');
    try {
      const response = await fetch(`/api/vyb/post/${postId}/like`, {
        method: 'POST',
        headers: { 'x-qor-id': this.currentUser },
      });
      return response.ok;
    } catch {
      console.warn('Like service unavailable');
      return false;
    }
  }

  async tipPost(postId: string, amount: number): Promise<boolean> {
    if (!this.currentUser) throw new Error('Not logged in');
    try {
      const response = await fetch(`/api/vyb/post/${postId}/tip`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'x-qor-id': this.currentUser },
        body: JSON.stringify({ amount }),
      });
      if (!response.ok) {
        const err = await response.json().catch(() => ({}));
        throw new Error(err.error || 'Tip failed');
      }
      return true;
    } catch (error) {
      console.error('Tip failed:', error);
      return false;
    }
  }

  // ============ Messaging Methods ============

  async getConversations(): Promise<Conversation[]> {
    if (!this.currentUser) return [];
    
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch conversations:', error);
      return [];
    }
  }

  async getMessages(conversationId: string): Promise<Message[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch messages:', error);
      return [];
    }
  }

  async sendMessage(conversationId: string, content: Message['content']): Promise<Message> {
    if (!this.currentUser) throw new Error('Not logged in');

    const message: Message = {
      id: `msg_${Date.now()}`,
      conversationId,
      sender: this.currentUser,
      content,
      timestamp: new Date(),
      status: 'sent',
      reactions: [],
    };

    return message;
  }

  async startConversation(targetQorId: string): Promise<Conversation> {
    if (!this.currentUser) throw new Error('Not logged in');

    const conversation: Conversation = {
      id: `conv_${Date.now()}`,
      participants: [
        { qorId: targetQorId, displayName: targetQorId.split('#')[0], isOnline: false },
      ],
      unreadCount: 0,
      isPinned: false,
      isMuted: false,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    return conversation;
  }

  // ============ Media Gallery Methods ============

  async getGallery(qorId: string): Promise<GalleryItem[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch gallery:', error);
      return [];
    }
  }

  async uploadMedia(file: File): Promise<GalleryItem> {
    if (!this.currentUser) throw new Error('Not logged in');

    // TODO: Upload to server
    const item: GalleryItem = {
      id: `media_${Date.now()}`,
      ownerId: this.currentUser,
      type: file.type.startsWith('video') ? 'video' : file.type.startsWith('audio') ? 'audio' : 'image',
      url: URL.createObjectURL(file),
      title: file.name,
      tags: [],
      uploadedAt: new Date(),
      expiresAt: new Date(Date.now() + 90 * 24 * 60 * 60 * 1000), // 3 months
      isMinted: false,
      isPublic: false,
      views: 0,
      likes: 0,
    };

    return item;
  }

  async mintMedia(mediaId: string, options: {
    name: string;
    description: string;
    royaltyPercent: number;
  }): Promise<{ success: boolean; nftId?: string; txHash?: string }> {
    if (!this.currentUser) throw new Error('Not logged in');
    
    try {
      const response = await fetch('/api/nft/mint', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': 'godmode_master_key',
        },
        body: JSON.stringify({
          name: options.name,
          description: options.description,
          creator: this.currentUser,
          owner: this.currentUser,
          metadata: {
            mediaId,
            source: 'VYB Social',
            royaltyBps: options.royaltyPercent * 100,
          },
        }),
      });
      
      const result = await response.json();
      
      if (!response.ok || !result.success) {
        return { success: false };
      }
      
      return {
        success: true,
        nftId: result.tokenId,
        txHash: result.txHash,
      };
    } catch (error) {
      console.error('Media mint failed:', error);
      return { success: false };
    }
  }

  /**
   * Upload media file to IPFS with optional album artwork
   */
  async uploadMediaToIPFS(file: File, albumArtwork?: File): Promise<MediaUploadResult> {
    if (!this.currentUser) throw new Error('Not logged in');

    console.log('[VYB Service] Starting upload for:', file.name, 'Size:', file.size);

    // Upload main media file
    const mediaResult = await ipfsClient.uploadFile(file, file.name);
    console.log('[VYB Service] Media upload result:', mediaResult);
    
    if (!mediaResult.success || !mediaResult.uri) {
      throw new Error(mediaResult.error || 'Failed to upload media');
    }

    let artworkUrl: string | undefined;
    let artworkIpfsUri: string | undefined;

    // Upload album artwork if provided (for audio files)
    if (albumArtwork) {
      console.log('[VYB Service] Uploading album artwork...');
      const artworkResult = await ipfsClient.uploadFile(albumArtwork, `artwork_${file.name}.jpg`);
      if (artworkResult.success && artworkResult.uri) {
        artworkIpfsUri = artworkResult.uri;
        // Use gateway URL if available, otherwise convert URI
        artworkUrl = artworkResult.gatewayUrl || ipfsToHttp(artworkResult.uri);
      }
    }

    // Use gateway URL if available (for local storage fallback), otherwise convert URI
    const url = mediaResult.gatewayUrl || ipfsToHttp(mediaResult.uri);
    console.log('[VYB Service] Final media URL:', url);

    // Create gallery item
    const galleryItem: GalleryItem = {
      id: `media_${Date.now()}_${Math.random().toString(36).slice(2)}`,
      ownerId: this.currentUser,
      type: file.type.startsWith('video') ? 'video' : file.type.startsWith('audio') ? 'audio' : 'image',
      url,
      thumbnailUrl: artworkUrl, // Use artwork as thumbnail for audio
      title: file.name,
      tags: [],
      uploadedAt: new Date(),
      expiresAt: new Date(Date.now() + 90 * 24 * 60 * 60 * 1000), // 3 months
      isMinted: false,
      isPublic: true,
      views: 0,
      likes: 0,
    };

    return {
      url,
      ipfsUri: mediaResult.uri,
      artworkUrl,
      artworkIpfsUri,
      galleryItem,
    };
  }

  /**
   * Mint media as DRC-369 NFT
   */
  async mintMediaAsNFT(options: MintMediaOptions): Promise<{ success: boolean; nftId?: string; txHash?: string }> {
    if (!this.currentUser) throw new Error('Not logged in');

    try {
      // Build NFT metadata
      const attributes: Array<{ trait_type: string; value: string | number }> = [
        { trait_type: 'Media Type', value: options.mediaType },
        { trait_type: 'Source', value: 'VYB Social' },
        { trait_type: 'Created', value: new Date().toISOString() },
      ];

      // Upload metadata to IPFS
      const metadata = {
        name: options.name,
        description: options.description || `${options.mediaType} shared on VYB`,
        image: options.artworkUrl || options.mediaUrl, // Use artwork if available, otherwise media URL
        external_url: `https://qorsync.dev/nft`,
        attributes,
        drc369: {
          version: '1.0' as const,
          creator: this.currentUser,
          royalty_bps: options.royaltyPercent * 100, // Convert percent to basis points
        },
        // Additional media reference for non-image types
        ...(options.mediaType !== 'image' && {
          animation_url: options.mediaUrl,
        }),
      };

      const metadataResult = await ipfsClient.uploadMetadata(metadata);
      if (!metadataResult.success || !metadataResult.uri) {
        throw new Error(metadataResult.error || 'Failed to upload NFT metadata');
      }

      // Call the blockchain RPC to mint
      const mintResult = await demiurgeRpc.mintNFT(
        this.currentUser, // creator
        metadataResult.uri, // metadata URI
        options.royaltyPercent * 100, // royalty in basis points
        '' // signature (handled by backend)
      );

      if (!mintResult.success) {
        throw new Error(mintResult.error || 'NFT minting failed');
      }

      return {
        success: true,
        nftId: mintResult.tokenId,
        txHash: mintResult.txHash,
      };
    } catch (error) {
      console.error('Failed to mint NFT:', error);
      return {
        success: false,
        nftId: undefined,
        txHash: undefined,
      };
    }
  }

  // ============ Notification Methods ============

  async getNotifications(): Promise<Notification[]> {
    if (!this.currentUser) return [];

    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch notifications:', error);
      return [];
    }
  }

  async markNotificationRead(notificationId: string): Promise<void> {
    // TODO: Implement notification read status update
  }

  async markAllNotificationsRead(): Promise<void> {
    // TODO: Implement bulk notification read status update
  }

  // ============ Service Marketplace Methods ============

  async getServices(category?: string): Promise<ServiceListing[]> {
    // TODO: Fetch from real API when available
    try {
      // Will return empty for now until API is implemented
      return [];
    } catch (error) {
      console.warn('Failed to fetch services:', error);
      return [];
    }
  }

  // ============ Friends Methods ============

  async getFriends(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch friends:', error);
      return [];
    }
  }

  async getFriendRequests(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch friend requests:', error);
      return [];
    }
  }

  async getFriendSuggestions(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch friend suggestions:', error);
      return [];
    }
  }

  async getTopFriends(limit: number = 8): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch top friends:', error);
      return [];
    }
  }

  async getOnlineFriends(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch online friends:', error);
      return [];
    }
  }

  // ============ Groups Methods ============

  async getGroups(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch groups:', error);
      return [];
    }
  }

  async getUserGroups(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch user groups:', error);
      return [];
    }
  }

  async getGroupInvites(): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch group invites:', error);
      return [];
    }
  }

  // ============ Events Methods ============

  async getEvents(type: 'upcoming' | 'past' = 'upcoming'): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch events:', error);
      return [];
    }
  }

  async getUpcomingEvents(limit: number = 10): Promise<any[]> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch upcoming events:', error);
      return [];
    }
  }

  // ============ Stats Methods ============

  async getWeeklyStats(): Promise<{ cgt: number; friends: number; likes: number; tips: number }> {
    try {
      // TODO: Fetch from real API when available
      return { cgt: 0, friends: 0, likes: 0, tips: 0 };
    } catch (error) {
      console.warn('Failed to fetch weekly stats:', error);
      return { cgt: 0, friends: 0, likes: 0, tips: 0 };
    }
  }

  async getTrending(): Promise<Array<{ tag: string; posts: number }>> {
    try {
      // TODO: Fetch from real API when available
      return [];
    } catch (error) {
      console.warn('Failed to fetch trending:', error);
      return [];
    }
  }
}

// Export singleton
export const vybService = new VYBService();
