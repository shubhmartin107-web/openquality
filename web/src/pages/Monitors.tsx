import { FormEvent, useState, useRef } from 'react';
import { useApi } from '../hooks/useApi';
import { api } from '../api/client';

const MONITOR_TYPES = [
  { value: 'freshness', label: 'Freshness', desc: 'Detect stale data' },
  { value: 'volume', label: 'Volume', desc: 'Row count anomalies' },
  { value: 'schema', label: 'Schema', desc: 'Column drift detection' },
  { value: 'distribution', label: 'Distribution', desc: 'Statistical distribution drift' },
  { value: 'correlation', label: 'Correlation', desc: 'Column relationship changes' },
  { value: 'uniqueness', label: 'Uniqueness', desc: 'Duplicate row ratio' },
  { value: 'referential_integrity', label: 'Referential Integrity', desc: 'Orphan row detection' },
  { value: 'custom_sql', label: 'Custom SQL', desc: 'Custom query assertions' },
  { value: 'ml_drift', label: 'ML Drift', desc: 'Model prediction/feature drift' },
  { value: 'cost', label: 'Cost', desc: 'Query/storage cost monitoring' },
];

export default function Monitors() {
  const workspaces = useApi(() => api.listWorkspaces());
  const ws = workspaces.data?.[0];
  const monitors = useApi(
    () => (ws ? api.listMonitors(ws.id) : Promise.resolve([])),
    [ws?.id],
  );

  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState('');
  const [monType, setMonType] = useState('freshness');
  const [tableName, setTableName] = useState('');
  const [cron, setCron] = useState('');
  const [creating, setCreating] = useState(false);
  const [runMsg, setRunMsg] = useState<{ id: string; msg: string } | null>(null);
  const runTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearRunMsg = (id: string) => {
    if (runTimerRef.current) clearTimeout(runTimerRef.current);
    runTimerRef.current = setTimeout(() => {
      setRunMsg((prev) => (prev?.id === id ? null : prev));
      runTimerRef.current = null;
    }, 3000);
  };

  const handleCreate = async (e: FormEvent) => {
    e.preventDefault();
    if (!ws) return;
    setCreating(true);
    try {
      await api.createMonitor(
        ws.id,
        name,
        monType,
        tableName,
        cron || undefined,
      );
      setShowCreate(false);
      setName('');
      setMonType('freshness');
      setTableName('');
      setCron('');
      monitors.refetch();
    } catch (err: unknown) {
      console.error('Create failed', err);
    } finally {
      setCreating(false);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await api.updateMonitor(id, { enabled: !enabled });
      monitors.refetch();
    } catch (err: unknown) {
      console.error('Toggle failed', err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteMonitor(id);
      monitors.refetch();
    } catch (err: unknown) {
      console.error('Delete failed', err);
    }
  };

  const handleRun = async (id: string) => {
    try {
      const res = await api.runMonitor(id);
      setRunMsg({ id, msg: JSON.stringify(res.result ?? 'ok').slice(0, 80) });
      clearRunMsg(id);
    } catch {
      setRunMsg({ id, msg: 'Run failed' });
      clearRunMsg(id);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Monitors</h1>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 transition-colors"
        >
          {showCreate ? 'Cancel' : 'New Monitor'}
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
              <label className="block text-sm font-medium mb-1">Table</label>
              <input
                required
                value={tableName}
                onChange={(e) => setTableName(e.target.value)}
                className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Type</label>
              <select
                value={monType}
                onChange={(e) => setMonType(e.target.value)}
                className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              >
                {MONITOR_TYPES.map((t) => (
                  <option key={t.value} value={t.value}>
                    {t.label}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">
                Cron{' '}
                <span className="text-gray-400 font-normal">(optional)</span>
              </label>
              <input
                value={cron}
                onChange={(e) => setCron(e.target.value)}
                placeholder="0 */6 * * *"
                className="w-full border rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
              />
            </div>
          </div>
          <button
            type="submit"
            disabled={creating}
            className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
          >
            {creating ? 'Creating...' : 'Create Monitor'}
          </button>
        </form>
      )}

      {monitors.loading ? (
        <p className="text-gray-400">Loading...</p>
      ) : (
        <div className="space-y-3">
          {monitors.data?.map((m) => (
            <div
              key={m.id}
              className="bg-white border rounded-lg p-4 flex items-center gap-4"
            >
              <div className="flex-1 min-w-0">
                <div className="font-medium">{m.name}</div>
                <div className="text-sm text-gray-500 truncate">
                  {m.monitor_type} &middot; {m.table_name}
                  {m.schedule_cron && ` · ${m.schedule_cron}`}
                </div>
              </div>
              <span
                className={`text-xs font-medium px-2 py-0.5 rounded ${
                  m.enabled
                    ? 'bg-green-100 text-green-700'
                    : 'bg-gray-100 text-gray-500'
                }`}
              >
                {m.enabled ? 'enabled' : 'disabled'}
              </span>
              <button
                onClick={() => handleRun(m.id)}
                className="text-xs text-brand-600 hover:text-brand-800"
              >
                Run
              </button>
              {runMsg?.id === m.id && (
                <span className="text-xs text-gray-500 truncate max-w-[120px]">
                  {runMsg.msg}
                </span>
              )}
              <button
                onClick={() => handleToggle(m.id, m.enabled)}
                className="text-xs text-gray-500 hover:text-gray-700"
              >
                {m.enabled ? 'Disable' : 'Enable'}
              </button>
              <button
                onClick={() => handleDelete(m.id)}
                className="text-xs text-red-500 hover:text-red-700"
              >
                Delete
              </button>
            </div>
          ))}
          {monitors.data?.length === 0 && (
            <p className="text-gray-400 text-sm">
              No monitors configured. Create one to get started.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
