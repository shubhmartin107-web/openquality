const BASE = '';

let token: string | null = localStorage.getItem('oq_token');

export function setAuthToken(t: string | null) {
  token = t;
  if (t) localStorage.setItem('oq_token', t);
  else localStorage.removeItem('oq_token');
}

export function getAuthToken(): string | null {
  return token ?? localStorage.getItem('oq_token');
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {};
  const t = getAuthToken();
  if (t) headers['Authorization'] = `Bearer ${t}`;
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (res.status === 401) {
    setAuthToken(null);
    window.location.href = '/login';
    throw new Error('Unauthorized');
  }
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? res.statusText);
  }
  if (res.status === 204) return undefined as unknown as T;
  return res.json();
}

export const api = {
  health: () => request<{ status: string }>('GET', '/health'),

  login: (email: string, password: string) =>
    request<{ token: string; user: unknown }>('POST', '/api/v1/auth/login', {
      email,
      password,
    }),

  register: (email: string, password: string, name: string) =>
    request<{ token: string; user: unknown }>('POST', '/api/v1/auth/register', {
      email,
      password,
      name,
    }),

  refreshToken: () =>
    request<{ token: string }>('POST', '/api/v1/auth/refresh'),

  listApiKeys: () =>
    request<Array<{ id: string; prefix: string; created_at: string }>>(
      'GET',
      '/api/v1/auth/api-keys',
    ),

  createApiKey: (label?: string) =>
    request<{ id: string; key: string; prefix: string }>(
      'POST',
      '/api/v1/auth/api-keys',
      { label },
    ),

  revokeApiKey: (id: string) =>
    request<void>('DELETE', `/api/v1/auth/api-keys/${id}`),

  listWorkspaces: () =>
    request<Array<{ id: string; name: string; slug: string }>>(
      'GET',
      '/api/v1/workspaces',
    ),

  getWorkspace: (id: string) =>
    request<{ id: string; name: string; slug: string }>(
      'GET',
      `/api/v1/workspaces/${id}`,
    ),

  createWorkspace: (name: string, slug: string) =>
    request<{ id: string }>('POST', '/api/v1/workspaces', { name, slug }),

  listUsers: (wsId: string) =>
    request<
      Array<{
        id: string;
        email: string;
        name: string;
        role: string;
      }>
    >('GET', `/api/v1/workspaces/${wsId}/users`),

  createUser: (wsId: string, email: string, role: string, password?: string) =>
    request<{ id: string }>('POST', `/api/v1/workspaces/${wsId}/users`, {
      email,
      role,
      password,
    }),

  getUser: (id: string) =>
    request<{ id: string; email: string; name: string; role: string }>(
      'GET',
      `/api/v1/users/${id}`,
    ),

  listDataSources: (wsId: string) =>
    request<
      Array<{
        id: string;
        name: string;
        connector_type: string;
        created_at: string;
      }>
    >('GET', `/api/v1/workspaces/${wsId}/data-sources`),

  createDataSource: (
    wsId: string,
    name: string,
    connectorType: string,
    config: unknown,
  ) =>
    request<{ id: string }>(
      'POST',
      `/api/v1/workspaces/${wsId}/data-sources`,
      { name, connector_type: connectorType, config },
    ),

  getDataSource: (id: string) =>
    request<{
      id: string;
      name: string;
      connector_type: string;
      config: unknown;
    }>('GET', `/api/v1/data-sources/${id}`),

  deleteDataSource: (id: string) =>
    request<void>('DELETE', `/api/v1/data-sources/${id}`),

  listMonitors: (wsId: string) =>
    request<
      Array<{
        id: string;
        name: string;
        monitor_type: string;
        table_name: string;
        enabled: boolean;
        schedule_cron: string | null;
        created_at: string;
      }>
    >('GET', `/api/v1/workspaces/${wsId}/monitors`),

  createMonitor: (
    wsId: string,
    name: string,
    monitorType: string,
    tableName: string,
    scheduleCron?: string,
  ) =>
    request<{ id: string }>('POST', `/api/v1/workspaces/${wsId}/monitors`, {
      name,
      monitor_type: monitorType,
      table_name: tableName,
      schedule_cron: scheduleCron,
    }),

  getMonitor: (id: string) =>
    request<{
      id: string;
      name: string;
      monitor_type: string;
      table_name: string;
      enabled: boolean;
      schedule_cron: string | null;
    }>('GET', `/api/v1/monitors/${id}`),

  updateMonitor: (
    id: string,
    data: { enabled?: boolean; schedule_cron?: string },
  ) => request<{ id: string }>('PUT', `/api/v1/monitors/${id}`, data),

  deleteMonitor: (id: string) =>
    request<void>('DELETE', `/api/v1/monitors/${id}`),

  runMonitor: (id: string) =>
    request<{ result: unknown }>('POST', `/api/v1/monitors/${id}/run`),

  getMonitorHistory: (id: string, limit?: number) =>
    request<Array<unknown>>(
      'GET',
      `/api/v1/monitors/${id}/history?limit=${limit ?? 20}`,
    ),

  listIncidents: () =>
    request<
      Array<{
        id: string;
        monitor_id: string;
        severity: string;
        message: string;
        timestamp: string;
        resolved: boolean;
        acked: boolean;
      }>
    >('GET', '/api/v1/incidents'),

  getIncident: (id: string) =>
    request<{
      id: string;
      monitor_id: string;
      severity: string;
      message: string;
      detail: unknown;
      root_cause_hints: string[];
      timestamp: string;
      resolved: boolean;
      acked: boolean;
    }>('GET', `/api/v1/incidents/${id}`),

  acknowledgeIncident: (id: string) =>
    request<{ id: string }>('POST', `/api/v1/incidents/${id}/acknowledge`),

  resolveIncident: (id: string) =>
    request<{ id: string }>('POST', `/api/v1/incidents/${id}/resolve`),

  snoozeIncident: (id: string, until: string) =>
    request<{ id: string }>('POST', `/api/v1/incidents/${id}/snooze`, {
      until,
    }),

  profileTable: (wsId: string, tableName: string) =>
    request<{ columns: unknown }>('POST', '/api/v1/profile', {
      workspace_id: wsId,
      table_name: tableName,
    }),

  suggestExpectations: (wsId: string, tableName: string) =>
    request<{ suggestions: unknown }>('POST', '/api/v1/suggest', {
      workspace_id: wsId,
      table_name: tableName,
    }),

  executeMcp: (tool: string, args: unknown) =>
    request<{ result: unknown }>('POST', '/api/v1/mcp/execute', {
      tool,
      args,
    }),

  dbtParseManifest: (manifestJson: unknown) =>
    request<{ models: unknown[]; sources: unknown[]; lineage_edges: unknown[]; model_count: number; source_count: number }>(
      'POST', '/api/v1/integrations/dbt/parse-manifest', { manifest_json: manifestJson }),

  airflowWebhook: (payload: unknown) =>
    request<{ event: string; dag_id: string; run_id: string; is_terminal: boolean; is_success: boolean; is_failure: boolean; affected_tables: string[]; duration_seconds: number | null; task_id: string | null }>(
      'POST', '/api/v1/integrations/airflow/webhook', payload),

  geTranslate: (suiteJson: unknown) =>
    request<{ suite_name: string; expectations: Array<{ expectation_type: string; column: string | null; kwargs: Record<string, unknown> }>; count: number }>(
      'POST', '/api/v1/integrations/ge/translate', { suite_json: suiteJson }),

  lineageParseSql: (sql: string) =>
    request<{ target_table: string; source_tables: string[]; column_mappings: Array<{ source_column: string; target_column: string }> }>(
      'POST', '/api/v1/integrations/lineage/parse-sql', { sql }),

  lineageBuildGraph: (statements: string[]) =>
    request<{ tables: unknown[] }>('POST', '/api/v1/integrations/lineage/build-graph', { statements }),
};
