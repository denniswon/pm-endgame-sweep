export interface RiskFlag {
  code: string;
  severity: string;
  evidence_spans: any[];
}

export interface Opportunity {
  market_id: string;
  as_of: string;
  recommended_side: string;
  entry_price: number;
  expected_payout: number;
  max_position_pct: number;
  risk_score: number;
  risk_flags: any[];
  notes?: string;
}

export interface OpportunitiesResponse {
  opportunities: Opportunity[];
  total: number;
  limit: number;
  offset: number;
}

export interface Quote {
  yes_bid: number | null;
  yes_ask: number | null;
  no_bid: number | null;
  no_ask: number | null;
  as_of: string;
  staleness_sec: number;
}

export interface Score {
  overall_score: number;
  yield_velocity: number;
  net_yield: number;
  liquidity_score: number;
  definition_risk_score: number;
  staleness_penalty: number;
  t_remaining_sec: number;
  as_of: string;
}

export interface Market {
  market_id: string;
  question: string;
  description?: string;
  end_date: string;
  close_time: string;
  volume: number;
  liquidity: number;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface MarketDetailsResponse {
  market: Market;
  quote?: Quote;
  score?: Score;
  recommendation?: Opportunity;
}
