// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { qorAuth } from '@demiurge/qor-sdk';
import { useToast } from '@/components/notifications';

interface User {
  id: string;
  qor_id: string;
  email: string;
  role: string;
  status: string;
  created_at: string;
}

interface Stats {
  total_users: number;
  active_sessions: number;
  registrations_24h: number;
  logins_24h: number;
  users_by_role: Array<{ role: string; count: number }>;
}

interface Transaction {
  hash: string;
  from: string;
  to: string | null;
  amount: string | null;
  nonce: number;
  status: string;
}

interface MintResult {
  tx_hash: string;
  to: string;
  amount: string;
  new_balance: string;
  reason: string;
  success: boolean;
}

export default function AdminPage() {
  const router = useRouter();
  const toast = useToast();
  const [isGod, setIsGod] = useState(false);
  const [loading, setLoading] = useState(true);
  const [usersLoading, setUsersLoading] = useState(false);
  const [users, setUsers] = useState<User[]>([]);
  const [stats, setStats] = useState<Stats | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [transactionsLoading, setTransactionsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'users' | 'tokens' | 'transactions' | 'stats'>('users');
  const [mintLoading, setMintLoading] = useState(false);
  const [mintResult, setMintResult] = useState<MintResult | null>(null);

  useEffect(() => {
    async function checkAccess() {
      try {
        const hasAccess = await qorAuth.isGod();
        setIsGod(hasAccess);
        setLoading(false);
        
        if (!hasAccess) {
          router.push('/');
          return;
        }

        // Load initial data
        await loadStats();
        await loadUsers();
      } catch (error) {
        console.error('Failed to check access:', error);
        setLoading(false);
        router.push('/');
      }
    }
    checkAccess();
  }, [router]);

  const loadUsers = async () => {
    setUsersLoading(true);
    try {
      const token = qorAuth.getToken();
      const response = await fetch('http://localhost:8080/api/v1/admin/users', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });
      const data = await response.json();
      setUsers(data.users || []);
    } catch (error) {
      console.error('Failed to load users:', error);
      toast.error('Failed to load users', 'Check server connection');
    } finally {
      setUsersLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const token = qorAuth.getToken();
      const response = await fetch('http://localhost:8080/api/v1/admin/stats', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });
      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Failed to load stats:', error);
      toast.error('Failed to load stats');
    }
  };

  const handleBanUser = async (userId: string) => {
    if (!confirm('Are you sure you want to ban this user?')) return;

    try {
      const token = qorAuth.getToken();
      const response = await fetch(`http://localhost:8080/api/v1/admin/users/${userId}/ban`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ reason: 'Admin action' }),
      });
      
      if (response.ok) {
        await loadUsers();
        toast.success('User banned successfully');
      } else {
        toast.error('Failed to ban user', 'Server returned an error');
      }
    } catch (error) {
      console.error('Failed to ban user:', error);
      toast.error('Failed to ban user', 'Network error');
    }
  };

  const handleTransferTokens = async (toUserId: string, amount: string) => {
    try {
      const token = qorAuth.getToken();
      const response = await fetch('http://localhost:8080/api/v1/admin/tokens/transfer', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          to_user_id: toUserId,
          amount: amount,
          reason: 'Customer support',
        }),
      });
      
      if (response.ok) {
        toast.success('Token transfer initiated', `Sent ${amount} CGT to ${toUserId}`);
      } else {
        toast.error('Transfer failed', 'Server returned an error');
      }
    } catch (error) {
      console.error('Failed to transfer tokens:', error);
      toast.error('Transfer failed', 'Network error');
    }
  };

  const loadTransactions = async () => {
    setTransactionsLoading(true);
    try {
      const response = await fetch(
        (process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944'),
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'admin_getRecentTransactions',
            params: [50],
          }),
        }
      );
      const data = await response.json();
      if (data.result) {
        setTransactions(data.result);
      }
    } catch (error) {
      console.error('Failed to load transactions:', error);
      toast.error('Failed to load transactions');
    } finally {
      setTransactionsLoading(false);
    }
  };

  const handleMintCGT = async (toAddress: string, amount: string, reason: string) => {
    setMintLoading(true);
    setMintResult(null);
    
    try {
      const token = qorAuth.getToken();
      const response = await fetch(
        (process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944'),
        {
          method: 'POST',
          headers: { 
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
          },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'admin_mintCgt',
            params: [toAddress, amount, reason, token],
          }),
        }
      );
      
      const data = await response.json();
      
      if (data.result) {
        setMintResult(data.result);
        toast.success('CGT Minted Successfully', `Minted to ${toAddress.slice(0, 10)}...`);
      } else if (data.error) {
        toast.error('Mint failed', data.error.message);
      }
    } catch (error) {
      console.error('Failed to mint CGT:', error);
      toast.error('Failed to mint CGT', 'Network error');
    } finally {
      setMintLoading(false);
    }
  };

  if (loading) {
    return (
      <main className="min-h-screen flex items-center justify-center">
        <div className="glass-panel p-8 rounded-lg text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-demiurge-gold mx-auto mb-4" />
          <div className="text-demiurge-cyan">Loading admin portal...</div>
        </div>
      </main>
    );
  }

  if (!isGod) {
    return null;
  }

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-5xl font-bold mb-4 bg-gradient-to-r from-demiurge-gold via-demiurge-violet to-demiurge-cyan bg-clip-text text-transparent">
          Admin Portal
        </h1>
        <p className="text-xl text-gray-300 mb-8">
          God-Level System Control
        </p>

        {/* Tabs */}
        <div className="flex gap-4 mb-8">
          {(['users', 'tokens', 'transactions', 'stats'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`glass-panel px-6 py-2 rounded-lg transition-all ${
                activeTab === tab
                  ? 'chroma-glow border border-demiurge-gold'
                  : 'hover:chroma-glow'
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        {/* Users Tab */}
        {activeTab === 'users' && (
          <div className="glass-panel p-6 rounded-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-2xl font-bold text-demiurge-cyan">User Management</h2>
              <button
                onClick={loadUsers}
                disabled={usersLoading}
                className="glass-panel px-4 py-2 rounded-lg hover:chroma-glow transition-all text-sm disabled:opacity-50"
              >
                {usersLoading ? 'Loading...' : 'Refresh'}
              </button>
            </div>
            
            {usersLoading ? (
              <div className="space-y-3">
                {[1, 2, 3, 4, 5].map((i) => (
                  <div key={i} className="h-12 bg-white/5 rounded-lg animate-pulse" />
                ))}
              </div>
            ) : users.length === 0 ? (
              <div className="text-center py-12">
                <div className="text-5xl mb-4">👤</div>
                <h3 className="text-xl font-bold text-white mb-2">No Users Found</h3>
                <p className="text-gray-400 mb-4">User data will appear here once users register.</p>
                <button
                  onClick={loadUsers}
                  className="text-demiurge-cyan hover:underline"
                >
                  Try loading again
                </button>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-demiurge-cyan/20">
                      <th className="text-left p-2">QOR ID</th>
                      <th className="text-left p-2">Email</th>
                      <th className="text-left p-2">Role</th>
                      <th className="text-left p-2">Status</th>
                      <th className="text-left p-2">Created</th>
                      <th className="text-left p-2">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {users.map((user) => (
                      <tr key={user.id} className="border-b border-demiurge-cyan/10 hover:bg-white/5">
                        <td className="p-2">{user.qor_id}</td>
                        <td className="p-2 text-gray-400">{user.email}</td>
                        <td className="p-2">
                          <span className={`px-2 py-1 rounded text-xs ${
                            user.role === 'god' ? 'bg-demiurge-gold/20 text-demiurge-gold' :
                            user.role === 'admin' ? 'bg-demiurge-violet/20 text-demiurge-violet' :
                            'bg-demiurge-cyan/20 text-demiurge-cyan'
                          }`}>
                            {user.role}
                          </span>
                        </td>
                        <td className="p-2">
                          <span className={`px-2 py-1 rounded text-xs ${
                            user.status === 'active' ? 'text-green-400' : 'text-red-400'
                          }`}>
                            {user.status}
                          </span>
                        </td>
                        <td className="p-2 text-gray-400 text-sm">
                          {new Date(user.created_at).toLocaleDateString()}
                        </td>
                        <td className="p-2">
                          <button
                            onClick={() => handleBanUser(user.id)}
                            className="text-red-400 hover:text-red-300 text-sm"
                          >
                            Ban
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* Tokens Tab */}
        {activeTab === 'tokens' && (
          <div className="space-y-6">
            {/* Mint CGT Section */}
            <div className="glass-panel p-6 rounded-lg">
              <h2 className="text-2xl font-bold mb-4 text-demiurge-gold">Mint CGT (Godmode)</h2>
              <p className="text-gray-400 mb-4">
                Issue CGT directly to an address. Use for error compensation or testing.
              </p>
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  const formData = new FormData(e.target as HTMLFormElement);
                  handleMintCGT(
                    formData.get('mint_address') as string,
                    formData.get('mint_amount') as string,
                    formData.get('mint_reason') as string
                  );
                }}
                className="space-y-4"
              >
                <div>
                  <label className="block text-sm mb-2">Recipient Address (hex)</label>
                  <input
                    type="text"
                    name="mint_address"
                    placeholder="0x..."
                    required
                    className="w-full glass-panel px-4 py-2 rounded-lg bg-black/20"
                  />
                </div>
                <div>
                  <label className="block text-sm mb-2">Amount (in smallest unit)</label>
                  <input
                    type="text"
                    name="mint_amount"
                    placeholder="100000000000000000000"
                    required
                    className="w-full glass-panel px-4 py-2 rounded-lg bg-black/20"
                  />
                  <p className="text-xs text-gray-500 mt-1">1 CGT = 100 Sparks. Enter raw amount.</p>
                </div>
                <div>
                  <label className="block text-sm mb-2">Reason</label>
                  <input
                    type="text"
                    name="mint_reason"
                    placeholder="Error compensation for user..."
                    required
                    className="w-full glass-panel px-4 py-2 rounded-lg bg-black/20"
                  />
                </div>
                <button
                  type="submit"
                  disabled={mintLoading}
                  className="glass-panel px-6 py-2 rounded-lg bg-demiurge-gold/20 border border-demiurge-gold hover:bg-demiurge-gold/30 transition-all disabled:opacity-50"
                >
                  {mintLoading ? 'Minting...' : 'Mint CGT'}
                </button>
              </form>
              
              {/* Mint Result */}
              {mintResult && (
                <div className={`mt-4 p-4 rounded-lg ${mintResult.success ? 'bg-green-900/20 border border-green-500' : 'bg-red-900/20 border border-red-500'}`}>
                  <h4 className="font-bold mb-2">{mintResult.success ? 'Mint Successful' : 'Mint Failed'}</h4>
                  <p className="text-sm text-gray-300">TX Hash: <code className="text-xs">{mintResult.tx_hash}</code></p>
                  <p className="text-sm text-gray-300">Amount: {mintResult.amount} → New Balance: {mintResult.new_balance}</p>
                  <p className="text-sm text-gray-300">Reason: {mintResult.reason}</p>
                </div>
              )}
            </div>

            {/* Transfer Section */}
            <div className="glass-panel p-6 rounded-lg">
              <h2 className="text-2xl font-bold mb-4 text-demiurge-cyan">Transfer CGT</h2>
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  const formData = new FormData(e.target as HTMLFormElement);
                  handleTransferTokens(
                    formData.get('to_user_id') as string,
                    formData.get('amount') as string
                  );
                }}
                className="space-y-4"
              >
                <div>
                  <label className="block text-sm mb-2">To User ID</label>
                  <input
                    type="text"
                    name="to_user_id"
                    required
                    className="w-full glass-panel px-4 py-2 rounded-lg bg-black/20"
                  />
                </div>
                <div>
                  <label className="block text-sm mb-2">Amount (CGT)</label>
                  <input
                    type="text"
                    name="amount"
                    required
                    className="w-full glass-panel px-4 py-2 rounded-lg bg-black/20"
                  />
                </div>
                <button
                  type="submit"
                  className="glass-panel px-6 py-2 rounded-lg hover:chroma-glow transition-all"
                >
                  Transfer
                </button>
              </form>
            </div>
          </div>
        )}

        {/* Transactions Tab */}
        {activeTab === 'transactions' && (
          <div className="glass-panel p-6 rounded-lg">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-2xl font-bold text-demiurge-cyan">Transaction Viewer</h2>
              <button
                onClick={loadTransactions}
                disabled={transactionsLoading}
                className="glass-panel px-4 py-2 rounded-lg hover:chroma-glow transition-all text-sm disabled:opacity-50"
              >
                {transactionsLoading ? 'Loading...' : 'Refresh'}
              </button>
            </div>
            
            {transactionsLoading ? (
              <div className="space-y-3">
                {[1, 2, 3, 4, 5].map((i) => (
                  <div key={i} className="h-10 bg-white/5 rounded-lg animate-pulse" />
                ))}
              </div>
            ) : transactions.length === 0 ? (
              <div className="text-center py-12">
                <div className="text-5xl mb-4">📋</div>
                <h3 className="text-xl font-bold text-white mb-2">No Transactions</h3>
                <p className="text-gray-400 mb-4">Click below to load recent blockchain transactions.</p>
                <button
                  onClick={loadTransactions}
                  className="glass-panel px-4 py-2 rounded-lg text-demiurge-cyan hover:chroma-glow transition-all"
                >
                  Load Transactions
                </button>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-demiurge-cyan/20">
                      <th className="text-left p-2 text-sm">Hash</th>
                      <th className="text-left p-2 text-sm">From</th>
                      <th className="text-left p-2 text-sm">To</th>
                      <th className="text-left p-2 text-sm">Amount</th>
                      <th className="text-left p-2 text-sm">Nonce</th>
                      <th className="text-left p-2 text-sm">Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {transactions.map((tx) => (
                      <tr key={tx.hash} className="border-b border-demiurge-cyan/10 hover:bg-white/5">
                        <td className="p-2 font-mono text-xs">
                          {tx.hash.slice(0, 10)}...{tx.hash.slice(-6)}
                        </td>
                        <td className="p-2 font-mono text-xs text-gray-400">
                          {tx.from.slice(0, 10)}...
                        </td>
                        <td className="p-2 font-mono text-xs text-gray-400">
                          {tx.to ? `${tx.to.slice(0, 10)}...` : '-'}
                        </td>
                        <td className="p-2 text-sm">
                          {tx.amount || '-'}
                        </td>
                        <td className="p-2 text-sm text-gray-400">
                          {tx.nonce}
                        </td>
                        <td className="p-2">
                          <span className={`px-2 py-1 rounded text-xs ${
                            tx.status === 'finalized' ? 'bg-green-900/30 text-green-400' :
                            tx.status === 'pending' ? 'bg-yellow-900/30 text-yellow-400' :
                            'bg-gray-900/30 text-gray-400'
                          }`}>
                            {tx.status}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* Stats Tab */}
        {activeTab === 'stats' && stats && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="glass-panel p-6 rounded-lg">
              <h3 className="text-sm text-gray-400 mb-2">Total Users</h3>
              <p className="text-3xl font-bold text-demiurge-cyan">{stats.total_users}</p>
            </div>
            <div className="glass-panel p-6 rounded-lg">
              <h3 className="text-sm text-gray-400 mb-2">Active Sessions</h3>
              <p className="text-3xl font-bold text-demiurge-violet">{stats.active_sessions}</p>
            </div>
            <div className="glass-panel p-6 rounded-lg">
              <h3 className="text-sm text-gray-400 mb-2">Registrations (24h)</h3>
              <p className="text-3xl font-bold text-demiurge-gold">{stats.registrations_24h}</p>
            </div>
            <div className="glass-panel p-6 rounded-lg">
              <h3 className="text-sm text-gray-400 mb-2">Logins (24h)</h3>
              <p className="text-3xl font-bold text-demiurge-cyan">{stats.logins_24h}</p>
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
