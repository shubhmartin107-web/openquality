import { useApi } from '../hooks/useApi';
import { api } from '../api/client';

export default function Dashboard() {
  const health = useApi(() => api.health());
  const workspaces = useApi(() => api.listWorkspaces());
  const incidents = useApi(() => api.listIncidents());
  const ws = workspaces.data?.[0];
  const monitors = useApi(
    () => (ws ? api.listMonitors(ws.id) : Promise.resolve([])),
    [ws?.id],
  );

  const openIncidents =
    incidents.data?.filter((i) => !i.resolved).length ?? 0;
  const activeMonitors = monitors.data?.filter((m) => m.enabled).length ?? 0;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Dashboard</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        <StatCard
          label="Active Monitors"
          value={String(activeMonitors)}
          loading={monitors.loading}
        />
        <StatCard
          label="Open Incidents"
          value={String(openIncidents)}
          loading={incidents.loading}
        />
        <StatCard
          label="Server Status"
          value={health.data?.status ?? 'unknown'}
          loading={health.loading}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white rounded-lg shadow-sm border p-5">
          <h2 className="font-semibold mb-3">Workspaces</h2>
          {workspaces.loading ? (
            <Skeleton />
          ) : (
            <ul className="space-y-2">
              {workspaces.data?.map((w) => (
                <li key={w.id} className="text-sm flex justify-between">
                  <span className="font-medium">{w.name}</span>
                  <span className="text-gray-400">{w.slug}</span>
                </li>
              ))}
              {workspaces.data?.length === 0 && (
                <li className="text-sm text-gray-400">No workspaces yet</li>
              )}
            </ul>
          )}
        </div>

        <div className="bg-white rounded-lg shadow-sm border p-5">
          <h2 className="font-semibold mb-3">Recent Incidents</h2>
          {incidents.loading ? (
            <Skeleton />
          ) : (
            <ul className="space-y-2">
              {incidents.data?.slice(0, 5).map((inc) => (
                <li key={inc.id} className="text-sm flex items-center gap-2">
                  <SeverityBadge severity={inc.severity} />
                  <span className="truncate flex-1">{inc.message}</span>
                  <span className="text-gray-400 text-xs">
                    {new Date(inc.timestamp).toLocaleDateString()}
                  </span>
                </li>
              ))}
              {incidents.data?.length === 0 && (
                <li className="text-sm text-gray-400">All clear — no incidents</li>
              )}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

function StatCard({
  label,
  value,
  loading,
}: {
  label: string;
  value: string;
  loading: boolean;
}) {
  return (
    <div className="bg-white rounded-lg shadow-sm border p-5">
      <div className="text-sm text-gray-500 mb-1">{label}</div>
      <div className="text-2xl font-bold">
        {loading ? '...' : value}
      </div>
    </div>
  );
}

function SeverityBadge({ severity }: { severity: string }) {
  const color =
    severity === 'critical'
      ? 'bg-red-100 text-red-700'
      : severity === 'warning'
        ? 'bg-yellow-100 text-yellow-700'
        : 'bg-blue-100 text-blue-700';
  return (
    <span
      className={`inline-block text-xs font-medium px-1.5 py-0.5 rounded ${color}`}
    >
      {severity}
    </span>
  );
}

function Skeleton() {
  return (
    <div className="space-y-2">
      {[1, 2, 3].map((i) => (
        <div key={i} className="h-4 bg-gray-100 rounded animate-pulse" />
      ))}
    </div>
  );
}
