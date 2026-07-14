import { useApi } from '../hooks/useApi';
import { api } from '../api/client';
import { FormEvent, useState } from 'react';

export default function Settings() {
  const workspaces = useApi(() => api.listWorkspaces());
  const ws = workspaces.data?.[0];
  const users = useApi(
    () => (ws ? api.listUsers(ws.id) : Promise.resolve([])),
    [ws?.id],
  );
  const apiKeys = useApi(() => api.listApiKeys());

  const [showWs, setShowWs] = useState(false);
  const [wsName, setWsName] = useState('');
  const [wsSlug, setWsSlug] = useState('');

  const [showUser, setShowUser] = useState(false);
  const [userEmail, setUserEmail] = useState('');
  const [userRole, setUserRole] = useState('member');

  const [newKeyLabel, setNewKeyLabel] = useState('');
  const [newKeyValue, setNewKeyValue] = useState<string | null>(null);

  const handleCreateWs = async (e: FormEvent) => {
    e.preventDefault();
    try {
      await api.createWorkspace(wsName, wsSlug);
      setShowWs(false);
      setWsName('');
      setWsSlug('');
      workspaces.refetch();
    } catch (err: unknown) {
      console.error('Create workspace failed', err);
    }
  };

  const handleInviteUser = async (e: FormEvent) => {
    e.preventDefault();
    if (!ws) return;
    try {
      await api.createUser(ws.id, userEmail, userRole);
      setShowUser(false);
      setUserEmail('');
      users.refetch();
    } catch (err: unknown) {
      console.error('Invite user failed', err);
    }
  };

  const handleCreateKey = async () => {
    try {
      const res = await api.createApiKey(newKeyLabel || undefined);
      setNewKeyValue(res.key);
      setNewKeyLabel('');
      apiKeys.refetch();
    } catch (err: unknown) {
      console.error('Create API key failed', err);
    }
  };

  const handleRevokeKey = async (id: string) => {
    try {
      await api.revokeApiKey(id);
      apiKeys.refetch();
    } catch (err: unknown) {
      console.error('Revoke key failed', err);
    }
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Settings</h1>

      <div className="space-y-6">
        {/* Workspaces */}
        <section className="bg-white border rounded-lg p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="font-semibold">Workspaces</h2>
            <button
              onClick={() => setShowWs(!showWs)}
              className="text-sm text-brand-600 hover:text-brand-800"
            >
              {showWs ? 'Cancel' : 'New'}
            </button>
          </div>
          {showWs && (
            <form onSubmit={handleCreateWs} className="flex gap-2 mb-3">
              <input
                required
                placeholder="Name"
                value={wsName}
                onChange={(e) => setWsName(e.target.value)}
                className="flex-1 border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
              <input
                required
                placeholder="slug"
                value={wsSlug}
                onChange={(e) => setWsSlug(e.target.value)}
                className="flex-1 border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 font-mono"
              />
              <button
                type="submit"
                className="bg-brand-600 text-white px-3 py-1.5 rounded text-sm"
              >
                Create
              </button>
            </form>
          )}
          <ul className="space-y-1 text-sm">
            {workspaces.data?.map((w) => (
              <li key={w.id} className="flex justify-between">
                <span>{w.name}</span>
                <span className="text-gray-400">{w.slug}</span>
              </li>
            ))}
          </ul>
        </section>

        {/* Users */}
        <section className="bg-white border rounded-lg p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="font-semibold">Users</h2>
            <button
              onClick={() => setShowUser(!showUser)}
              className="text-sm text-brand-600 hover:text-brand-800"
            >
              {showUser ? 'Cancel' : 'Invite'}
            </button>
          </div>
          {showUser && ws && (
            <form onSubmit={handleInviteUser} className="flex gap-2 mb-3">
              <input
                required
                type="email"
                placeholder="Email"
                value={userEmail}
                onChange={(e) => setUserEmail(e.target.value)}
                className="flex-1 border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
              <select
                value={userRole}
                onChange={(e) => setUserRole(e.target.value)}
                className="border rounded px-3 py-1.5 text-sm"
              >
                <option value="admin">Admin</option>
                <option value="member">Member</option>
                <option value="viewer">Viewer</option>
              </select>
              <button
                type="submit"
                className="bg-brand-600 text-white px-3 py-1.5 rounded text-sm"
              >
                Invite
              </button>
            </form>
          )}
          <ul className="space-y-1 text-sm">
            {users.data?.map((u) => (
              <li key={u.id} className="flex justify-between">
                <span>
                  {u.name || u.email}
                </span>
                <span className="text-gray-400">{u.role}</span>
              </li>
            ))}
          </ul>
        </section>

        {/* API Keys */}
        <section className="bg-white border rounded-lg p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="font-semibold">API Keys</h2>
            <div className="flex gap-2">
              <input
                placeholder="Label (optional)"
                value={newKeyLabel}
                onChange={(e) => setNewKeyLabel(e.target.value)}
                className="border rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
              <button
                onClick={handleCreateKey}
                className="bg-brand-600 text-white px-3 py-1.5 rounded text-sm"
              >
                Generate
              </button>
            </div>
          </div>
          {newKeyValue && (
            <div className="bg-yellow-50 border border-yellow-200 rounded p-3 mb-3 text-sm">
              <strong className="block mb-1">Key generated — copy it now:</strong>
              <code className="bg-white px-2 py-1 rounded text-xs break-all">
                {newKeyValue}
              </code>
              <button
                onClick={() => setNewKeyValue(null)}
                className="block mt-1 text-xs text-gray-500 hover:text-gray-700"
              >
                Dismiss
              </button>
            </div>
          )}
          <ul className="space-y-1 text-sm">
            {apiKeys.data?.map((k) => (
              <li key={k.id} className="flex justify-between items-center">
                <span>
                  <code className="bg-gray-100 px-1 py-0.5 rounded text-xs">
                    {k.prefix}...
                  </code>
                  <span className="text-gray-400 ml-2">
                    {new Date(k.created_at).toLocaleDateString()}
                  </span>
                </span>
                <button
                  onClick={() => handleRevokeKey(k.id)}
                  className="text-xs text-red-500 hover:text-red-700"
                >
                  Revoke
                </button>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
