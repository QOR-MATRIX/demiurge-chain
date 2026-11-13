// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Notification Service
 * Real-time notifications for transactions, alerts, and events
 */

// ============================================================================
// Types
// ============================================================================

export type NotificationType =
  | 'transaction'
  | 'reward'
  | 'alert'
  | 'achievement'
  | 'announcement'
  | 'error'
  | 'warning'
  | 'info';

export type NotificationPriority = 'low' | 'normal' | 'high' | 'critical';

export interface Notification {
  id: string;
  type: NotificationType;
  priority: NotificationPriority;
  title: string;
  message: string;
  icon?: string;
  action?: {
    label: string;
    url: string;
  };
  timestamp: Date;
  read: boolean;
  expiresAt?: Date;
}

export interface NotificationPreferences {
  email: {
    enabled: boolean;
    byType: Record<NotificationType, boolean>;
  };
  push: {
    enabled: boolean;
    byType: Record<NotificationType, boolean>;
  };
  inApp: {
    enabled: boolean;
    byType: Record<NotificationType, boolean>;
  };
  quietHours: {
    enabled: boolean;
    start: string; // HH:MM
    end: string; // HH:MM
  };
  summary: {
    enabled: boolean;
    frequency: 'realtime' | 'hourly' | 'daily';
  };
}

export interface NotificationListener {
  id: string;
  callback: (notification: Notification) => void;
}

// ============================================================================
// Notification Service
// ============================================================================

class NotificationService {
  private notifications: Notification[] = [];
  private preferences: NotificationPreferences;
  private listeners: Map<string, NotificationListener> = new Map();
  private listenerIdCounter = 0;
  private storageKey = 'sophia_notifications';
  private maxStoredNotifications = 100;

  constructor() {
    this.preferences = this.loadPreferences();
    this.loadNotifications();
  }

  /**
   * Send a notification
   */
  notify(
    type: NotificationType,
    title: string,
    message: string,
    options?: {
      priority?: NotificationPriority;
      icon?: string;
      action?: { label: string; url: string };
      expiresIn?: number; // milliseconds
    }
  ): Notification {
    const notification: Notification = {
      id: `notif_${Date.now()}_${Math.random().toString(36).slice(2)}`,
      type,
      priority: options?.priority || 'normal',
      title,
      message,
      icon: options?.icon,
      action: options?.action,
      timestamp: new Date(),
      read: false,
      expiresAt: options?.expiresIn ? new Date(Date.now() + options.expiresIn) : undefined,
    };

    // Check if notification should be shown based on preferences and quiet hours
    if (this.shouldShowNotification(notification)) {
      this.notifications.unshift(notification);

      // Trim old notifications
      if (this.notifications.length > this.maxStoredNotifications) {
        this.notifications = this.notifications.slice(0, this.maxStoredNotifications);
      }

      this.saveNotifications();
      this.notifyListeners(notification);

      // Dispatch to different channels
      this.dispatchNotification(notification);
    }

    return notification;
  }

  /**
   * Get all notifications
   */
  getNotifications(filter?: { type?: NotificationType; read?: boolean }): Notification[] {
    let result = this.notifications;

    if (filter?.type) {
      result = result.filter((n) => n.type === filter.type);
    }

    if (filter?.read !== undefined) {
      result = result.filter((n) => n.read === filter.read);
    }

    return result;
  }

  /**
   * Get unread notifications count
   */
  getUnreadCount(): number {
    return this.notifications.filter((n) => !n.read).length;
  }

  /**
   * Mark notification as read
   */
  markAsRead(notificationId: string): boolean {
    const notification = this.notifications.find((n) => n.id === notificationId);

    if (!notification) return false;

    notification.read = true;
    this.saveNotifications();
    return true;
  }

  /**
   * Mark all notifications as read
   */
  markAllAsRead(): void {
    for (const notification of this.notifications) {
      notification.read = true;
    }
    this.saveNotifications();
  }

  /**
   * Delete a notification
   */
  deleteNotification(notificationId: string): boolean {
    const index = this.notifications.findIndex((n) => n.id === notificationId);

    if (index === -1) return false;

    this.notifications.splice(index, 1);
    this.saveNotifications();
    return true;
  }

  /**
   * Clear all notifications
   */
  clearAll(): void {
    this.notifications = [];
    this.saveNotifications();
  }

  /**
   * Subscribe to notification events
   */
  subscribe(callback: (notification: Notification) => void): string {
    const id = `listener_${++this.listenerIdCounter}`;
    this.listeners.set(id, { id, callback });
    return id;
  }

  /**
   * Unsubscribe from notifications
   */
  unsubscribe(listenerId: string): boolean {
    return this.listeners.delete(listenerId);
  }

  /**
   * Update notification preferences
   */
  setPreferences(preferences: Partial<NotificationPreferences>): void {
    this.preferences = {
      ...this.preferences,
      ...preferences,
    };
    localStorage.setItem('sophia_notification_preferences', JSON.stringify(this.preferences));
  }

  /**
   * Get current preferences
   */
  getPreferences(): NotificationPreferences {
    return { ...this.preferences };
  }

  /**
   * Request browser notification permission
   */
  async requestPermission(): Promise<NotificationPermission> {
    if (!('Notification' in window)) {
      console.warn('Notifications not supported');
      return 'denied';
    }

    if (Notification.permission === 'granted') {
      return 'granted';
    }

    const permission = await Notification.requestPermission();
    return permission;
  }

  // ============================================================================
  // Notification Helpers
  // ============================================================================

  /**
   * Notify transaction confirmed
   */
  notifyTransactionConfirmed(txHash: string, amount: string, type: 'sent' | 'received'): void {
    const title = type === 'sent' ? 'Sent' : 'Received';
    const message =
      type === 'sent'
        ? `You sent ${amount} CGT (${txHash.slice(0, 10)}...)`
        : `You received ${amount} CGT (${txHash.slice(0, 10)}...)`;

    this.notify('transaction', title, message, {
      priority: 'high',
      icon: type === 'sent' ? '📤' : '📥',
      action: {
        label: 'View',
        url: `https://qorsync.dev/tx/${txHash}`,
      },
    });
  }

  /**
   * Notify transaction failed
   */
  notifyTransactionFailed(reason: string): void {
    this.notify('error', 'Transaction Failed', reason, {
      priority: 'critical',
      icon: '❌',
      expiresIn: 5 * 60 * 1000, // 5 minutes
    });
  }

  /**
   * Notify staking reward
   */
  notifyStakingReward(amount: string, era: number): void {
    this.notify('reward', 'Staking Reward', `You earned ${amount} CGT in era ${era}`, {
      priority: 'high',
      icon: '🎁',
    });
  }

  /**
   * Notify achievement unlocked
   */
  notifyAchievementUnlocked(achievement: string, description: string): void {
    this.notify('achievement', '🏆 Achievement Unlocked!', `${achievement}\n${description}`, {
      priority: 'high',
      icon: '🏆',
    });
  }

  /**
   * Notify price alert
   */
  notifyPriceAlert(token: string, price: number, targetPrice: number): void {
    this.notify(
      'alert',
      'Price Alert',
      `${token} reached $${price} (your target: $${targetPrice})`,
      {
        priority: 'high',
        icon: '📊',
      }
    );
  }

  /**
   * Notify system maintenance
   */
  notifyMaintenance(duration: string, startTime: string): void {
    this.notify(
      'announcement',
      'Scheduled Maintenance',
      `RPC maintenance for ${duration} starting at ${startTime}`,
      {
        priority: 'normal',
        icon: '🔧',
      }
    );
  }

  /**
   * Notify security alert
   */
  notifySecurityAlert(type: string, details: string): void {
    this.notify('warning', 'Security Alert', `${type}: ${details}`, {
      priority: 'critical',
      icon: '🔒',
      expiresIn: 24 * 60 * 60 * 1000, // 24 hours
    });
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private shouldShowNotification(notification: Notification): boolean {
    // Check if notification type is enabled
    if (!this.preferences.inApp.byType[notification.type]) {
      return false;
    }

    // Check quiet hours
    if (this.preferences.quietHours.enabled) {
      if (this.isInQuietHours()) {
        // Only show critical notifications during quiet hours
        if (notification.priority !== 'critical') {
          return false;
        }
      }
    }

    return true;
  }

  private isInQuietHours(): boolean {
    const now = new Date();
    const currentTime = `${String(now.getHours()).padStart(2, '0')}:${String(
      now.getMinutes()
    ).padStart(2, '0')}`;

    const { start, end } = this.preferences.quietHours;

    if (start < end) {
      return currentTime >= start && currentTime < end;
    } else {
      // Quiet hours wrap around midnight
      return currentTime >= start || currentTime < end;
    }
  }

  private dispatchNotification(notification: Notification): void {
    // Browser notification
    if (this.preferences.push.enabled && this.preferences.push.byType[notification.type]) {
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(notification.title, {
          body: notification.message,
          icon: notification.icon,
          badge: '/favicon.ico',
          tag: notification.id,
          requireInteraction: notification.priority === 'critical',
        });
      }
    }

    // Email notification (mock - would call API)
    if (this.preferences.email.enabled && this.preferences.email.byType[notification.type]) {
      this.sendEmailNotification(notification);
    }
  }

  private sendEmailNotification(notification: Notification): void {
    // Mock implementation - would call backend API
    console.log('📧 Email notification:', notification.title);
  }

  private notifyListeners(notification: Notification): void {
    for (const listener of this.listeners.values()) {
      try {
        listener.callback(notification);
      } catch (error) {
        console.error('Error in notification listener:', error);
      }
    }
  }

  private saveNotifications(): void {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(this.notifications));
    } catch (error) {
      console.error('Failed to save notifications:', error);
    }
  }

  private loadNotifications(): void {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (stored) {
        const notifications = JSON.parse(stored) as Notification[];
        // Parse timestamps back to Date objects
        this.notifications = notifications.map((n) => ({
          ...n,
          timestamp: new Date(n.timestamp),
          expiresAt: n.expiresAt ? new Date(n.expiresAt) : undefined,
        }));

        // Remove expired notifications
        this.notifications = this.notifications.filter((n) => {
          if (n.expiresAt && n.expiresAt < new Date()) {
            return false;
          }
          return true;
        });

        this.saveNotifications();
      }
    } catch (error) {
      console.error('Failed to load notifications:', error);
    }
  }

  private loadPreferences(): NotificationPreferences {
    try {
      const stored = localStorage.getItem('sophia_notification_preferences');
      if (stored) {
        return JSON.parse(stored);
      }
    } catch (error) {
      console.error('Failed to load notification preferences:', error);
    }

    // Default preferences
    return {
      email: {
        enabled: false,
        byType: {
          transaction: true,
          reward: true,
          alert: true,
          achievement: true,
          announcement: false,
          error: true,
          warning: true,
          info: false,
        },
      },
      push: {
        enabled: false,
        byType: {
          transaction: true,
          reward: true,
          alert: true,
          achievement: true,
          announcement: false,
          error: true,
          warning: true,
          info: false,
        },
      },
      inApp: {
        enabled: true,
        byType: {
          transaction: true,
          reward: true,
          alert: true,
          achievement: true,
          announcement: true,
          error: true,
          warning: true,
          info: true,
        },
      },
      quietHours: {
        enabled: false,
        start: '22:00',
        end: '08:00',
      },
      summary: {
        enabled: false,
        frequency: 'daily',
      },
    };
  }
}

// Export singleton instance
export const notificationService = new NotificationService();

// ============================================================================
// React Hook
// ============================================================================

import { useEffect, useState, useCallback } from 'react';

export function useNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);

  useEffect(() => {
    // Load initial notifications
    setNotifications(notificationService.getNotifications());
    setUnreadCount(notificationService.getUnreadCount());

    // Subscribe to new notifications
    const listenerId = notificationService.subscribe((notification) => {
      setNotifications((prev) => [notification, ...prev]);
      setUnreadCount((prev) => prev + 1);
    });

    return () => {
      notificationService.unsubscribe(listenerId);
    };
  }, []);

  const markAsRead = useCallback((notificationId: string) => {
    notificationService.markAsRead(notificationId);
    setUnreadCount((prev) => Math.max(0, prev - 1));
  }, []);

  const markAllAsRead = useCallback(() => {
    notificationService.markAllAsRead();
    setUnreadCount(0);
  }, []);

  const deleteNotification = useCallback((notificationId: string) => {
    notificationService.deleteNotification(notificationId);
    setNotifications((prev) => prev.filter((n) => n.id !== notificationId));
  }, []);

  const clearAll = useCallback(() => {
    notificationService.clearAll();
    setNotifications([]);
    setUnreadCount(0);
  }, []);

  return {
    notifications,
    unreadCount,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    clearAll,
    preferences: notificationService.getPreferences(),
    setPreferences: (prefs) => notificationService.setPreferences(prefs),
  };
}
