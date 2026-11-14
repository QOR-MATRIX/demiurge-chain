// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Demiurge Wallet Extension - QOR ID Login Screen
import React, { useState } from 'react';
import { useStore } from '../store';
import { Button } from '../components/Button';

export function LoginScreen() {
  const { authLogin, isLoading } = useStore();
  const [showForm, setShowForm] = useState(false);
  const [identifier, setIdentifier] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  const handleQorIdLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!identifier.trim()) {
      setError('Please enter your QOR ID');
      return;
    }
    if (!password) {
      setError('Please enter your password');
      return;
    }

    try {
      await authLogin(identifier.trim(), password);
    } catch (e) {
      setError((e as Error).message || 'Login failed');
    }
  };

  // Landing / splash screen
  if (!showForm) {
    return (
      <div className="flex-1 flex flex-col p-6">
        <div className="flex-1 flex flex-col items-center justify-center">
          <div className="w-20 h-20 bg-gradient-to-br from-demiurge-400 to-demiurge-600 rounded-2xl flex items-center justify-center mb-6">
            <span className="text-white font-bold text-3xl">D</span>
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Demiurge</h1>
          <p className="text-gray-400 text-center text-sm mb-8">
            Your portal to the Demiurge ecosystem
          </p>
        </div>

        <div className="space-y-3">
          <Button fullWidth onClick={() => setShowForm(true)}>
            <span className="flex items-center justify-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
              Sign In with QOR ID
            </span>
          </Button>
        </div>

        <p className="text-center text-gray-500 text-xs mt-4">
          Don&apos;t have a QOR ID?{' '}
          <a
            href="https://qorsync.dev/auth"
            target="_blank"
            rel="noopener noreferrer"
            className="text-demiurge-400 hover:text-demiurge-300"
          >
            Create one
          </a>
        </p>
      </div>
    );
  }

  // QOR ID Login form
  return (
    <div className="flex-1 flex flex-col p-6">
      <button
        onClick={() => setShowForm(false)}
        className="flex items-center gap-2 text-gray-400 hover:text-white mb-6 transition-colors"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
        </svg>
        Back
      </button>

      <div className="flex items-center justify-center mb-6">
        <div className="w-14 h-14 bg-gradient-to-br from-demiurge-400 to-demiurge-600 rounded-2xl flex items-center justify-center">
          <svg className="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        </div>
      </div>

      <h1 className="text-xl font-bold text-white mb-1 text-center">QOR ID Login</h1>
      <p className="text-gray-400 text-sm mb-6 text-center">
        Sign in with your QOR ID to access the full ecosystem
      </p>

      <form onSubmit={handleQorIdLogin} className="space-y-4 flex-1">
        <div>
          <label className="block text-gray-300 text-sm mb-2">QOR ID or Email</label>
          <input
            type="text"
            value={identifier}
            onChange={(e) => setIdentifier(e.target.value)}
            placeholder="username#1234 or email"
            className="input"
            autoFocus
          />
        </div>

        <div>
          <label className="block text-gray-300 text-sm mb-2">Password</label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Enter your password"
            className="input"
          />
        </div>

        {error && (
          <p className="text-red-500 text-sm text-center">{error}</p>
        )}

        <Button
          fullWidth
          type="submit"
          loading={isLoading}
          disabled={!identifier || !password}
        >
          Sign In
        </Button>
      </form>

      <p className="text-center text-gray-500 text-xs mt-4">
        <a
          href="https://qorsync.dev/auth"
          target="_blank"
          rel="noopener noreferrer"
          className="text-demiurge-400 hover:text-demiurge-300"
        >
          Create a QOR ID
        </a>
        {' · '}
        <a
          href="https://qorsync.dev/auth/reset"
          target="_blank"
          rel="noopener noreferrer"
          className="text-gray-400 hover:text-gray-300"
        >
          Forgot password?
        </a>
      </p>
    </div>
  );
}
