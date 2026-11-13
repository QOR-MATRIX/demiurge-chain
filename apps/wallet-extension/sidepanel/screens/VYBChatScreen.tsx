// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// VYB Chat Screen for Extension Side Panel
// Compact chat UI optimized for ~350px side panel width

import React, { useState, useEffect, useRef, useCallback } from 'react';

const HUB_URL = 'https://qorsync.dev';

interface ChatRoom {
  id: string;
  name: string;
  type: 'global' | 'public' | 'private';
  member_count: number;
  description: string;
}

interface ChatMessage {
  id: string;
  room_id: string;
  sender_qor_id: string;
  sender_display_name?: string;
  content: string;
  type: string;
  created_at: string;
}

interface DMConversation {
  qor_id: string;
  display_name: string;
  last_message: string;
  last_message_at: string;
  unread_count: number;
}

type View = 'rooms' | 'chat' | 'dm-list' | 'dm-chat';

function getAuthHeaders(qorId: string): HeadersInit {
  return {
    'Content-Type': 'application/json',
    'x-qor-id': qorId,
  };
}

export function VYBChatScreen() {
  const [qorId, setQorId] = useState<string>('');
  const [authToken, setAuthToken] = useState<string>('');
  const [view, setView] = useState<View>('rooms');
  const [rooms, setRooms] = useState<ChatRoom[]>([]);
  const [activeRoom, setActiveRoom] = useState<ChatRoom | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [conversations, setConversations] = useState<DMConversation[]>([]);
  const [dmPeer, setDmPeer] = useState<{ qorId: string; name: string } | null>(null);
  const [dmMessages, setDmMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const [loading, setLoading] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Get auth from background
  useEffect(() => {
    chrome.runtime.sendMessage({ type: 'AUTH_GET_SESSION' }, (response) => {
      if (response?.success && response.data?.isAuthenticated) {
        setQorId(response.data.user?.qorId || '');
        setAuthToken(response.data.token || '');
      }
    });
  }, []);

  // Load rooms
  useEffect(() => {
    if (!qorId) return;
    loadRooms();
    const interval = setInterval(loadRooms, 10000);
    return () => clearInterval(interval);
  }, [qorId]);

  // Auto-scroll on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, dmMessages]);

  const loadRooms = async () => {
    try {
      const res = await fetch(`${HUB_URL}/api/vyb/rooms`, { headers: getAuthHeaders(qorId) });
      const data = await res.json();
      setRooms(data.rooms || []);
    } catch {}
    setLoading(false);
  };

  const loadMessages = async (roomId: string) => {
    try {
      const res = await fetch(`${HUB_URL}/api/vyb/rooms/${roomId}/messages`, { headers: getAuthHeaders(qorId) });
      const data = await res.json();
      setMessages((data.messages || []).reverse());
    } catch {}
  };

  const loadDMConversations = async () => {
    try {
      const res = await fetch(`${HUB_URL}/api/vyb/dm`, { headers: getAuthHeaders(qorId) });
      const data = await res.json();
      setConversations(data.conversations || []);
    } catch {}
  };

  const loadDMHistory = async (peerQorId: string) => {
    try {
      const res = await fetch(`${HUB_URL}/api/vyb/dm/${encodeURIComponent(peerQorId)}`, { headers: getAuthHeaders(qorId) });
      const data = await res.json();
      setDmMessages((data.messages || []).reverse());
    } catch {}
  };

  const enterRoom = async (room: ChatRoom) => {
    // Auto-join
    try {
      await fetch(`${HUB_URL}/api/vyb/rooms/${room.id}/join`, {
        method: 'POST',
        headers: getAuthHeaders(qorId),
      });
    } catch {}

    setActiveRoom(room);
    setView('chat');
    await loadMessages(room.id);

    // Poll for new messages
    const interval = setInterval(() => loadMessages(room.id), 3000);
    return () => clearInterval(interval);
  };

  const enterDM = async (peer: DMConversation) => {
    setDmPeer({ qorId: peer.qor_id, name: peer.display_name });
    setView('dm-chat');
    await loadDMHistory(peer.qor_id);
  };

  const handleSendMessage = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!input.trim() || sending) return;

    const content = input.trim();
    setInput('');
    setSending(true);

    try {
      if (view === 'chat' && activeRoom) {
        const res = await fetch(`${HUB_URL}/api/vyb/rooms/${activeRoom.id}/messages`, {
          method: 'POST',
          headers: getAuthHeaders(qorId),
          body: JSON.stringify({ content }),
        });
        const data = await res.json();
        if (data.message) setMessages(prev => [...prev, data.message]);
      } else if (view === 'dm-chat' && dmPeer) {
        const res = await fetch(`${HUB_URL}/api/vyb/dm`, {
          method: 'POST',
          headers: getAuthHeaders(qorId),
          body: JSON.stringify({ receiverQorId: dmPeer.qorId, content }),
        });
        const data = await res.json();
        if (data.message) setDmMessages(prev => [...prev, data.message]);
      }
    } catch {
      setInput(content);
    }
    setSending(false);
  };

  const formatTime = (dateStr: string) => {
    const d = new Date(dateStr);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  // Not authenticated
  if (!qorId) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-4 text-center">
        <p className="text-2xl mb-2">💬</p>
        <p className="text-white text-sm mb-1">VYB Chat</p>
        <p className="text-gray-400 text-xs">Sign in with QOR ID to access chat</p>
      </div>
    );
  }

  // Room List View
  if (view === 'rooms') {
    return (
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700/50">
          <span className="text-white text-sm font-medium">💬 VYB Chat</span>
          <div className="flex gap-1">
            <button
              onClick={() => { setView('dm-list'); loadDMConversations(); }}
              className="text-xs px-2 py-1 rounded bg-gray-700/50 text-gray-300 hover:bg-gray-600/50"
            >
              ✉️ DMs
            </button>
            <button
              onClick={() => chrome.tabs.create({ url: `${HUB_URL}/social` })}
              className="text-xs px-2 py-1 rounded bg-demiurge-500/20 text-demiurge-400 hover:bg-demiurge-500/30"
            >
              🌐 Full VYB
            </button>
          </div>
        </div>

        {/* Room List */}
        <div className="flex-1 overflow-y-auto">
          {loading && (
            <div className="flex justify-center py-8">
              <div className="w-5 h-5 border-2 border-demiurge-400 border-t-transparent rounded-full animate-spin" />
            </div>
          )}

          {!loading && rooms.length === 0 && (
            <div className="text-center py-8 text-gray-500 text-xs">No rooms available</div>
          )}

          {rooms.map((room) => (
            <button
              key={room.id}
              onClick={() => enterRoom(room)}
              className="w-full flex items-center gap-2 px-3 py-2.5 hover:bg-white/5 transition-colors border-b border-gray-800/50 text-left"
            >
              <span className="text-gray-400 text-xs">
                {room.type === 'private' ? '🔒' : room.type === 'global' ? '🌐' : '#'}
              </span>
              <div className="flex-1 min-w-0">
                <p className="text-white text-xs truncate">{room.name}</p>
                {room.description && <p className="text-gray-600 text-[10px] truncate">{room.description}</p>}
              </div>
              <span className="text-gray-600 text-[10px]">{room.member_count}👤</span>
            </button>
          ))}
        </div>
      </div>
    );
  }

  // DM List View
  if (view === 'dm-list') {
    return (
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="flex items-center gap-2 px-3 py-2 border-b border-gray-700/50">
          <button onClick={() => setView('rooms')} className="text-gray-400 hover:text-white text-xs">←</button>
          <span className="text-white text-sm font-medium">Direct Messages</span>
        </div>

        <div className="flex-1 overflow-y-auto">
          {conversations.length === 0 && (
            <div className="text-center py-8 text-gray-500 text-xs">No conversations yet</div>
          )}

          {conversations.map((conv) => (
            <button
              key={conv.qor_id}
              onClick={() => enterDM(conv)}
              className="w-full flex items-center gap-2 px-3 py-2.5 hover:bg-white/5 transition-colors border-b border-gray-800/50 text-left"
            >
              <div className="w-7 h-7 rounded-full bg-gray-700 flex items-center justify-center text-xs flex-shrink-0">👤</div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <p className="text-white text-xs truncate">{conv.display_name}</p>
                  <span className="text-gray-600 text-[10px]">{formatTime(conv.last_message_at)}</span>
                </div>
                <p className="text-gray-500 text-[10px] truncate">{conv.last_message}</p>
              </div>
              {conv.unread_count > 0 && (
                <span className="w-4 h-4 rounded-full bg-red-500 text-white text-[9px] flex items-center justify-center">{conv.unread_count}</span>
              )}
            </button>
          ))}
        </div>
      </div>
    );
  }

  // Chat View (Room or DM)
  const isRoomChat = view === 'chat';
  const chatTitle = isRoomChat ? activeRoom?.name : dmPeer?.name;
  const currentMessages = isRoomChat ? messages : dmMessages;

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Chat Header */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-gray-700/50">
        <button
          onClick={() => setView(isRoomChat ? 'rooms' : 'dm-list')}
          className="text-gray-400 hover:text-white text-xs"
        >
          ←
        </button>
        <span className="text-white text-sm font-medium truncate">
          {isRoomChat ? '#' : '💬'} {chatTitle}
        </span>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-0.5">
        {currentMessages.length === 0 && (
          <div className="text-center py-8 text-gray-500 text-xs">No messages yet</div>
        )}

        {currentMessages.map((msg) => {
          const isOwn = msg.sender_qor_id === qorId;
          const isSystem = msg.type === 'system';

          if (isSystem) {
            return (
              <div key={msg.id} className="text-center py-0.5">
                <span className="text-gray-600 text-[10px] italic">{msg.content}</span>
              </div>
            );
          }

          return (
            <div key={msg.id} className="py-0.5">
              <span className={`text-[10px] font-medium ${isOwn ? 'text-demiurge-400' : 'text-gray-300'}`}>
                {msg.sender_display_name || msg.sender_qor_id.split('#')[0]}
              </span>
              <span className="text-gray-600 text-[9px] ml-1">{formatTime(msg.created_at)}</span>
              <p className="text-gray-300 text-xs break-words">{msg.content}</p>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <form onSubmit={handleSendMessage} className="px-3 py-2 border-t border-gray-700/50 flex gap-1.5">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={`Message ${chatTitle}...`}
          className="flex-1 bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white placeholder-gray-500 focus:border-demiurge-400/50 focus:outline-none"
          maxLength={4000}
          disabled={sending}
        />
        <button
          type="submit"
          disabled={!input.trim() || sending}
          className="px-3 py-1.5 bg-demiurge-500/20 text-demiurge-400 rounded text-xs hover:bg-demiurge-500/30 disabled:opacity-30 transition-colors"
        >
          Send
        </button>
      </form>
    </div>
  );
}
