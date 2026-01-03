import type { OpportunitiesResponse, MarketDetailsResponse } from '@/types/api';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

export async function getOpportunities(params?: {
  min_score?: number;
  max_risk_score?: number;
  max_t_remaining_sec?: number;
  has_flags?: boolean;
  limit?: number;
  offset?: number;
}): Promise<OpportunitiesResponse> {
  const query = new URLSearchParams();

  if (params?.min_score !== undefined) {
    query.set('min_score', params.min_score.toString());
  }
  if (params?.max_risk_score !== undefined) {
    query.set('max_risk_score', params.max_risk_score.toString());
  }
  if (params?.max_t_remaining_sec !== undefined) {
    query.set('max_t_remaining_sec', params.max_t_remaining_sec.toString());
  }
  if (params?.has_flags !== undefined) {
    query.set('has_flags', params.has_flags.toString());
  }
  if (params?.limit !== undefined) {
    query.set('limit', params.limit.toString());
  }
  if (params?.offset !== undefined) {
    query.set('offset', params.offset.toString());
  }

  const url = `${API_BASE}/v1/opportunities${query.toString() ? `?${query}` : ''}`;
  const res = await fetch(url, {
    cache: 'no-store',
  });

  if (!res.ok) {
    throw new Error(`Failed to fetch opportunities: ${res.status} ${res.statusText}`);
  }

  return res.json();
}

export async function getMarket(id: string): Promise<MarketDetailsResponse> {
  const res = await fetch(`${API_BASE}/v1/market/${id}`, {
    cache: 'no-store',
  });

  if (!res.ok) {
    throw new Error(`Failed to fetch market: ${res.status} ${res.statusText}`);
  }

  return res.json();
}

export async function checkHealth(): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/health`);

  if (!res.ok) {
    throw new Error('API health check failed');
  }

  return res.json();
}
