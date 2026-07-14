import { FormEvent, useState } from 'react';
import { useApi } from '../hooks/useApi';
import { api } from '../api/client';

const CONNECTOR_TYPES = ['postgres', 'snowflake', 'bigquery'];

export default function DataSources() {
  const workspaces = useApi(() => api.listWorkspaces());
  const ws = workspaces.data?.[0];
  const sources = useApi(
    () => (ws ? api.listDataSources(ws.id) : Promise.resolve([])),
    [ws?.id],
  );

  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState('');
  const [connType, setConnType] = useState('postgres');
  const [connStr, setConnStr] = useState('');
  const [creating, setCreating] = useState(false);

  const handleCreate = async (e: FormEvent) => {
    e.preventDefault();
    if (!ws) return;
    setCreating(true);
    try {
      await api.createDataSource(ws.id, name, connType, {
        connection_string: connStr,
      });
      setShowCreate(false);
      setName('');
      setConnType('postgres');
      setConnStr('');
      sources.refetch();
    } catch (err: unknown) {
      console.error('Create failed', err);
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteDataSource(id);
      sources.refetch();
    } catch (err: unknown) {
      console.error('Delete failed', err);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Data Sources</h1>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 transition-colors"
        >
          {showCreate ? 'Cancel' : 'New Data Source'}
        </button>
      </div>

      {showCreate && ws && (
        <form
          onSubmit={handleCreate}
          className="bg-white border rounded-lg p-5 mb-6 space-y-3"
        >
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium mb-1">Name</label>
              <input
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Type</label>
              <select
                value={connType}
                onChange={(e) => setConnType(e.target.value)}
                className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              >
                {CONNECTOR_TYPES.map((t) => (
                  <option key={t} value={t}>
                    {t}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">
              Connection String
            </label>
            <input
              required
              value={connStr}
              onChange={(e) => setConnStr(e.target.value)}
              placeholder="postgresql://user:pass@host:5432/db"
              className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 font-mono"
            />
          </div>
          <button
            type="submit"
            disabled={creating}
            className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
          >
            {creating ? 'Creating...' : 'Create Data Source'}
          </button>
        </form>
      )}

      {sources.loading ? (
        <p className="text-gray-400">Loading...</p>
      ) : (
        <div className="space-y-3">
          {sources.data?.map((ds) => (
            <div
              key={ds.id}
              className="bg-white border rounded-lg p-4 flex items-center gap-4"
            >
              <div className="flex-1">
                <div className="font-medium">{ds.name}</div>
                <div className="text-sm text-gray-500">{ds.connector_type}</div>
              </div>
              <div className="text-xs text-gray-400">
                {new Date(ds.created_at).toLocaleDateString()}
              </div>
              <button
                onClick={() => handleDelete(ds.id)}
                className="text-xs text-red-500 hover:text-red-700"
              >
                Delete
              </button>
            </div>
          ))}
          {sources.data?.length === 0 && (
            <p className="text-gray-400 text-sm">
              No data sources configured.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
