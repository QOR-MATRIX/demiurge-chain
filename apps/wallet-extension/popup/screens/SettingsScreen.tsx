// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Demiurge Wallet Extension - Settings Screen
import React, { useState } from 'react';
import { useStore } from '../store';
import { Button } from '../components/Button';

type SettingsTab = 'general' | 'security' | 'about';

export function SettingsScreen() {
  const { setView, lock, activeAccount, network, detachWallet, authLogout, isAuthenticated, authUser } = useStore();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [autoLockTime, setAutoLockTime] = useState('15');
  const [showExportModal, setShowExportModal] = useState(false);
  const [exportPassword, setExportPassword] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [exportError, setExportError] = useState('');
  const [showDetachConfirm, setShowDetachConfirm] = useState(false);

  const handleDetachWallet = async () => {
    await detachWallet();
    setShowDetachConfirm(false);
  };

  const handleExportPrivateKey = async () => {
    setExportError('');
    
    try {
      const response = await chrome.runtime.sendMessage({
        type: 'EXPORT_PRIVATE_KEY',
        payload: { password: exportPassword },
      });

      if (response.success && response.data) {
        setPrivateKey(response.data.privateKey);
      } else {
        setExportError(response.error || 'Failed to export private key');
      }
    } catch (error) {
      setExportError('Failed to export private key');
    }
  };

  const handleClearData = async () => {
    if (confirm('Are you sure you want to clear all wallet data? This action cannot be undone.')) {
      if (confirm('This will delete all accounts and settings. Have you backed up your recovery phrase?')) {
        await chrome.storage.local.clear();
        window.location.reload();
      }
    }
  };

  const tabs = [
    { id: 'general', label: 'General' },
    { id: 'security', label: 'Security' },
    { id: 'about', label: 'About' },
  ] as const;

  return (
    <div className="flex-1 flex flex-col p-4">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <button
          onClick={() => setView('main')}
          className="p-2 text-gray-400 hover:text-white transition-colors"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <h1 className="text-xl font-bold text-white">Settings</h1>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-800/50 rounded-lg p-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'bg-gray-700 text-white'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'general' && (
          <div className="space-y-4">
            <div className="card">
              <h3 className="text-white font-medium mb-3">Auto-Lock Timer</h3>
              <select
                value={autoLockTime}
                onChange={(e) => setAutoLockTime(e.target.value)}
                className="input"
              >
                <option value="5">5 minutes</option>
                <option value="15">15 minutes</option>
                <option value="30">30 minutes</option>
                <option value="60">1 hour</option>
                <option value="0">Never</option>
              </select>
            </div>

            <div className="card">
              <h3 className="text-white font-medium mb-3">Connected Sites</h3>
              <p className="text-gray-400 text-sm">
                Manage sites connected to your wallet
              </p>
              <Button variant="ghost" className="mt-3" fullWidth>
                View Connected Sites
              </Button>
            </div>
          </div>
        )}

        {activeTab === 'security' && (
          <div className="space-y-4">
            {isAuthenticated && authUser && (
              <div className="card">
                <h3 className="text-white font-medium mb-3">Signed In</h3>
                <p className="text-gray-400 text-sm mb-1">
                  QOR ID: <span className="text-white font-mono">{authUser.qorId}</span>
                </p>
                <p className="text-gray-400 text-sm mb-3">
                  {authUser.displayName && `Display name: ${authUser.displayName}`}
                </p>
                <Button variant="secondary" fullWidth onClick={authLogout}>
                  Sign Out
                </Button>
              </div>
            )}

            <div className="card">
              <h3 className="text-white font-medium mb-3">Export Private Key</h3>
              <p className="text-gray-400 text-sm mb-3">
                Warning: Never share your private key with anyone.
              </p>
              <Button 
                variant="secondary" 
                fullWidth
                onClick={() => setShowExportModal(true)}
              >
                Export Private Key
              </Button>
            </div>

            <div className="card border-yellow-500/30">
              <h3 className="text-yellow-400 font-medium mb-3">Detach Wallet</h3>
              <p className="text-gray-400 text-sm mb-3">
                Sign out and clear all local wallet data. You can sign back in with your QOR ID at any time.
              </p>
              <Button variant="secondary" fullWidth onClick={() => setShowDetachConfirm(true)}>
                Detach Wallet
              </Button>
            </div>

            <div className="card border-red-500/30">
              <h3 className="text-red-400 font-medium mb-3">Danger Zone</h3>
              <p className="text-gray-400 text-sm mb-3">
                Clear all wallet data including cached sessions. This cannot be undone.
              </p>
              <Button variant="danger" fullWidth onClick={handleClearData}>
                Clear All Data
              </Button>
            </div>
          </div>
        )}

        {activeTab === 'about' && (
          <div className="space-y-4">
            <div className="card text-center">
              <div className="w-16 h-16 bg-gradient-to-br from-demiurge-400 to-demiurge-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <span className="text-white font-bold text-2xl">D</span>
              </div>
              <h3 className="text-white font-bold text-lg">Demiurge Wallet</h3>
              <p className="text-gray-400 text-sm">Version 1.0.0</p>
            </div>

            <div className="card">
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">Network</span>
                  <span className="text-white">{network}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Active Account</span>
                  <span className="text-white font-mono">
                    {activeAccount?.slice(0, 8)}...{activeAccount?.slice(-6)}
                  </span>
                </div>
              </div>
            </div>

            <div className="card">
              <h3 className="text-white font-medium mb-3">Links</h3>
              <div className="space-y-2">
                <a
                  href="https://qorsync.dev"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-between text-gray-300 hover:text-white transition-colors"
                >
                  <span>Website</span>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
                <a
                  href="https://qorsync.dev"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-between text-gray-300 hover:text-white transition-colors"
                >
                  <span>Documentation</span>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
                <a
                  href="https://github.com/ALaustrup/demiurge-cloud"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-between text-gray-300 hover:text-white transition-colors"
                >
                  <span>GitHub</span>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Export Modal */}
      {showExportModal && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
          <div className="bg-gray-800 rounded-xl p-6 w-full max-w-sm">
            <h2 className="text-xl font-bold text-white mb-4">Export Private Key</h2>
            
            {!privateKey ? (
              <>
                <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4">
                  <p className="text-red-400 text-sm">
                    Warning: Your private key gives full access to your wallet. Never share it with anyone.
                  </p>
                </div>

                <input
                  type="password"
                  value={exportPassword}
                  onChange={(e) => setExportPassword(e.target.value)}
                  placeholder="Enter your password"
                  className="input mb-4"
                />

                {exportError && (
                  <p className="text-red-500 text-sm mb-4">{exportError}</p>
                )}

                <div className="flex gap-3">
                  <Button
                    variant="secondary"
                    fullWidth
                    onClick={() => {
                      setShowExportModal(false);
                      setExportPassword('');
                      setExportError('');
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    variant="danger"
                    fullWidth
                    onClick={handleExportPrivateKey}
                    disabled={!exportPassword}
                  >
                    Export
                  </Button>
                </div>
              </>
            ) : (
              <>
                <div className="bg-gray-900 rounded-lg p-3 mb-4">
                  <p className="text-white text-xs font-mono break-all select-all">
                    {privateKey}
                  </p>
                </div>

                <Button
                  fullWidth
                  onClick={() => {
                    setShowExportModal(false);
                    setExportPassword('');
                    setPrivateKey('');
                  }}
                >
                  Done
                </Button>
              </>
            )}
          </div>
        </div>
      )}

      {/* Detach Confirmation Modal */}
      {showDetachConfirm && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
          <div className="bg-gray-800 rounded-xl p-6 w-full max-w-sm">
            <h2 className="text-xl font-bold text-white mb-3">Detach Wallet</h2>
            <p className="text-gray-400 text-sm mb-2">
              This will:
            </p>
            <ul className="text-gray-400 text-sm mb-4 list-disc list-inside space-y-1">
              <li>Sign you out of your QOR ID</li>
              <li>Clear all local wallet data</li>
              <li>Remove cached notes and media</li>
            </ul>
            <p className="text-gray-300 text-sm mb-4">
              You can sign back in with your QOR ID at any time. Your on-chain data remains safe.
            </p>
            <div className="flex gap-3">
              <Button
                variant="secondary"
                fullWidth
                onClick={() => setShowDetachConfirm(false)}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                fullWidth
                onClick={handleDetachWallet}
              >
                Detach
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
