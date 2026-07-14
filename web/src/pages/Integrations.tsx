import { FormEvent, useState } from 'react';
import { api } from '../api/client';

type ActiveTab = 'dbt' | 'airflow' | 'ge' | 'lineage';

export default function Integrations() {
  const [tab, setTab] = useState<ActiveTab>('dbt');

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Integrations</h1>

      <div className="flex gap-1 mb-6 border-b">
        {(['dbt', 'airflow', 'ge', 'lineage'] as ActiveTab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === t
                ? 'border-brand-600 text-brand-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            {t === 'dbt' ? 'dbt' : t === 'ge' ? 'Great Expectations' : capitalize(t)}
          </button>
        ))}
      </div>

      {tab === 'dbt' && <DbtTab />}
      {tab === 'airflow' && <AirflowTab />}
      {tab === 'ge' && <GeTab />}
      {tab === 'lineage' && <LineageTab />}
    </div>
  );
}

function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function DbtTab() {
  const [manifest, setManifest] = useState('');
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      const parsed = JSON.parse(manifest);
      const res = await api.dbtParseManifest(parsed);
      setResult(res);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Parse dbt Manifest</h2>
        <form onSubmit={handleSubmit} className="space-y-3">
          <textarea
            rows={12}
            value={manifest}
            onChange={(e) => setManifest(e.target.value)}
            placeholder='Paste your manifest.json content here...'
            className="w-full border rounded px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
          {error && (
            <div className="bg-red-50 text-red-700 text-sm px-3 py-2 rounded">
              {error}
            </div>
          )}
          <button
            type="submit"
            disabled={loading || !manifest}
            className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
          >
            {loading ? 'Parsing...' : 'Parse Manifest'}
          </button>
        </form>
      </div>

      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Result</h2>
        {result && typeof result === 'object' && 'model_count' in result ? (
          <div className="space-y-3 text-sm">
            <div className="flex gap-4">
              <span className="text-gray-500">
                Models: <strong>{String((result as Record<string, unknown>).model_count)}</strong>
              </span>
              <span className="text-gray-500">
                Sources: <strong>{String((result as Record<string, unknown>).source_count)}</strong>
              </span>
            </div>
            <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-96">
              {JSON.stringify(result, null, 2)}
            </pre>
          </div>
        ) : (
          <p className="text-sm text-gray-400">Parse a manifest to see results</p>
        )}
      </div>
    </div>
  );
}

function AirflowTab() {
  const [payload, setPayload] = useState(
    JSON.stringify(
      {
        dag_id: 'load_orders',
        run_id: 'manual_001',
        state: 'success',
        execution_date: new Date().toISOString(),
        duration: 120.5,
        event: 'dag_run_success',
      },
      null,
      2,
    ),
  );
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      const parsed = JSON.parse(payload);
      const res = await api.airflowWebhook(parsed);
      setResult(res);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Airflow Webhook</h2>
        <p className="text-sm text-gray-500 mb-3">
          Simulate an Airflow webhook payload to test parsing.
        </p>
        <form onSubmit={handleSubmit} className="space-y-3">
          <textarea
            rows={10}
            value={payload}
            onChange={(e) => setPayload(e.target.value)}
            className="w-full border rounded px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
          {error && (
            <div className="bg-red-50 text-red-700 text-sm px-3 py-2 rounded">
              {error}
            </div>
          )}
          <button
            type="submit"
            disabled={loading}
            className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
          >
            {loading ? 'Sending...' : 'Send Webhook'}
          </button>
        </form>
      </div>

      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Parsed Event</h2>
        {result ? (
          <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-96">
            {JSON.stringify(result, null, 2)}
          </pre>
        ) : (
          <p className="text-sm text-gray-400">Send a webhook to see the parsed result</p>
        )}
      </div>
    </div>
  );
}

function GeTab() {
  const [suite, setSuite] = useState(
    JSON.stringify(
      {
        expectation_suite_name: 'my_suite',
        expectations: [
          {
            expectation_type: 'expect_column_values_to_not_be_null',
            kwargs: { column: 'id' },
            meta: {},
          },
          {
            expectation_type: 'expect_column_values_to_be_unique',
            kwargs: { column: 'email' },
            meta: {},
          },
          {
            expectation_type: 'expect_column_values_to_be_in_set',
            kwargs: {
              column: 'status',
              value_set: ['active', 'inactive'],
            },
            meta: {},
          },
        ],
        meta: {},
      },
      null,
      2,
    ),
  );
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      const parsed = JSON.parse(suite);
      const res = await api.geTranslate(parsed);
      setResult(res);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Great Expectations Suite</h2>
        <form onSubmit={handleSubmit} className="space-y-3">
          <textarea
            rows={12}
            value={suite}
            onChange={(e) => setSuite(e.target.value)}
            className="w-full border rounded px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
          {error && (
            <div className="bg-red-50 text-red-700 text-sm px-3 py-2 rounded">
              {error}
            </div>
          )}
          <button
            type="submit"
            disabled={loading}
            className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
          >
            {loading ? 'Translating...' : 'Translate to OpenQuality'}
          </button>
        </form>
      </div>

      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Translated Expectations</h2>
        {result && typeof result === 'object' && 'suite_name' in result ? (
          <div className="space-y-3 text-sm">
            <div>
              Suite:{' '}
              <strong>{String((result as Record<string, unknown>).suite_name)}</strong>
            </div>
            <div>
              Expectations:{' '}
              <strong>{String((result as Record<string, unknown>).count)}</strong>
            </div>
            <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-80">
              {JSON.stringify(result, null, 2)}
            </pre>
          </div>
        ) : (
          <p className="text-sm text-gray-400">Translate a suite to see results</p>
        )}
      </div>
    </div>
  );
}

function LineageTab() {
  const [sql, setSql] = useState(
    'CREATE TABLE analytics.orders AS\nSELECT o.id, o.amount, u.name AS user_name\nFROM raw.orders o\nJOIN raw.users u ON o.user_id = u.id',
  );
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      const res = await api.lineageParseSql(sql);
      setResult(res);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  const handleBatchBuild = async () => {
    setLoading(true);
    setError('');
    try {
      const statements = sql
        .split(';')
        .map((s) => s.trim())
        .filter(Boolean);
      const res = await api.lineageBuildGraph(statements);
      setResult(res);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">SQL Lineage Parser</h2>
        <form onSubmit={handleSubmit} className="space-y-3">
          <textarea
            rows={8}
            value={sql}
            onChange={(e) => setSql(e.target.value)}
            className="w-full border rounded px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
          {error && (
            <div className="bg-red-50 text-red-700 text-sm px-3 py-2 rounded">
              {error}
            </div>
          )}
          <div className="flex gap-2">
            <button
              type="submit"
              disabled={loading || !sql}
              className="bg-brand-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-brand-700 disabled:opacity-50"
            >
              {loading ? 'Parsing...' : 'Parse SQL'}
            </button>
            <button
              type="button"
              onClick={handleBatchBuild}
              disabled={loading || !sql}
              className="bg-gray-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-gray-700 disabled:opacity-50"
            >
              Build Full Graph
            </button>
          </div>
        </form>
      </div>

      <div className="bg-white border rounded-lg p-5">
        <h2 className="font-semibold mb-3">Lineage Result</h2>
        {result ? (
          <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-96">
            {JSON.stringify(result, null, 2)}
          </pre>
        ) : (
          <p className="text-sm text-gray-400">
            Enter SQL and parse to see column-level lineage
          </p>
        )}
      </div>
    </div>
  );
}
