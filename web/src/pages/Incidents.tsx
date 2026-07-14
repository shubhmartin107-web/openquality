import { useApi } from '../hooks/useApi';
import { api } from '../api/client';
import { useState } from 'react';

export default function Incidents() {
  const incidents = useApi(() => api.listIncidents());
  const [selected, setSelected] = useState<string | null>(null);
  const detail = useApi(
    () => (selected ? api.getIncident(selected) : Promise.resolve(null)),
    [selected],
  );

  const handleAck = async (id: string) => {
    try {
      await api.acknowledgeIncident(id);
      incidents.refetch();
    } catch (err: unknown) {
      console.error('Acknowledge failed', err);
    }
  };

  const handleResolve = async (id: string) => {
    try {
      await api.resolveIncident(id);
      incidents.refetch();
      if (selected === id) setSelected(null);
    } catch (err: unknown) {
      console.error('Resolve failed', err);
    }
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Incidents</h1>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white border rounded-lg">
          <div className="px-4 py-3 border-b font-medium text-sm text-gray-500 uppercase tracking-wider">
            All Incidents
          </div>
          {incidents.loading ? (
            <div className="p-4 text-sm text-gray-400">Loading...</div>
          ) : (
            <ul className="divide-y">
              {incidents.data?.map((inc) => (
                <li
                  key={inc.id}
                  onClick={() => setSelected(inc.id)}
                  className={`px-4 py-3 cursor-pointer transition-colors hover:bg-gray-50 ${
                    selected === inc.id ? 'bg-brand-50 border-l-2 border-brand-500' : ''
                  }`}
                >
                  <div className="flex items-center gap-2 mb-1">
                    <SeverityBadge severity={inc.severity} />
                    {inc.resolved && (
                      <span className="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded">
                        resolved
                      </span>
                    )}
                    {inc.acked && !inc.resolved && (
                      <span className="text-xs bg-blue-100 text-blue-700 px-1.5 py-0.5 rounded">
                        acked
                      </span>
                    )}
                  </div>
                  <div className="text-sm truncate">{inc.message}</div>
                  <div className="text-xs text-gray-400 mt-0.5">
                    {new Date(inc.timestamp).toLocaleString()}
                  </div>
                </li>
              ))}
              {incidents.data?.length === 0 && (
                <li className="px-4 py-8 text-center text-sm text-gray-400">
                  No incidents
                </li>
              )}
            </ul>
          )}
        </div>

        <div className="bg-white border rounded-lg">
          <div className="px-4 py-3 border-b font-medium text-sm text-gray-500 uppercase tracking-wider">
            Details
          </div>
          {detail.error && (
            <div className="p-4 text-sm text-red-600">
              Failed to load details: {detail.error}
            </div>
          )}
          {selected && detail.data ? (
            <div className="p-4 space-y-4 text-sm">
              <div>
                <span className="text-gray-500">Severity: </span>
                <SeverityBadge severity={detail.data.severity} />
              </div>
              <div>
                <span className="text-gray-500">Message: </span>
                {detail.data.message}
              </div>
              <div>
                <span className="text-gray-500">Monitor ID: </span>
                <code className="text-xs bg-gray-100 px-1 py-0.5 rounded">
                  {detail.data.monitor_id}
                </code>
              </div>
              <div>
                <span className="text-gray-500">Time: </span>
                {new Date(detail.data.timestamp).toLocaleString()}
              </div>
              {detail.data.root_cause_hints?.length > 0 && (
                <div>
                  <span className="text-gray-500 block mb-1">
                    Root Cause Hints:
                  </span>
                  <ul className="list-disc list-inside space-y-1 text-gray-600">
                    {detail.data.root_cause_hints.map((h, i) => (
                      <li key={i}>{h}</li>
                    ))}
                  </ul>
                </div>
              )}
              <div className="flex gap-2 pt-2">
                {!detail.data.acked && !detail.data.resolved && (
                  <button
                    onClick={() => handleAck(selected)}
                    className="bg-brand-600 text-white px-3 py-1.5 rounded text-xs font-medium hover:bg-brand-700"
                  >
                    Acknowledge
                  </button>
                )}
                {!detail.data.resolved && (
                  <button
                    onClick={() => handleResolve(selected)}
                    className="bg-green-600 text-white px-3 py-1.5 rounded text-xs font-medium hover:bg-green-700"
                  >
                    Resolve
                  </button>
                )}
              </div>
            </div>
          ) : (
            <div className="p-4 text-sm text-gray-400">
              Select an incident to view details
            </div>
          )}
        </div>
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
