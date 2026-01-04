'use client';

import useSWR from 'swr';
import Link from 'next/link';
import { TrendingUp, AlertTriangle } from 'lucide-react';
import { getOpportunities } from '@/lib/api';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { formatPercent, formatScore, getRiskColor } from '@/lib/utils';
import type { Opportunity } from '@/types/api';

function TableSkeleton() {
  return (
    <div className="space-y-2">
      {[...Array(5)].map((_, i) => (
        <Skeleton key={i} className="h-16 w-full" />
      ))}
    </div>
  );
}

function ErrorState({ error }: { error: Error }) {
  return (
    <div className="flex items-center justify-center p-8 text-destructive">
      <AlertTriangle className="mr-2 h-5 w-5" />
      <span>Failed to load opportunities: {error.message}</span>
    </div>
  );
}

function OpportunityRow({ opportunity }: { opportunity: Opportunity }) {
  const hasFlags = opportunity.risk_flags && opportunity.risk_flags.length > 0;

  return (
    <TableRow>
      <TableCell>
        <Link
          href={`/market/${opportunity.market_id}`}
          className="font-medium text-primary hover:underline"
        >
          {opportunity.market_id.substring(0, 8)}...
        </Link>
      </TableCell>
      <TableCell>
        <Badge variant={opportunity.recommended_side === 'YES' ? 'default' : 'secondary'}>
          {opportunity.recommended_side}
        </Badge>
      </TableCell>
      <TableCell className="font-mono">
        {formatPercent(opportunity.entry_price)}
      </TableCell>
      <TableCell className="font-mono">
        {formatPercent(opportunity.expected_payout)}
      </TableCell>
      <TableCell className="font-mono">
        {formatPercent(opportunity.max_position_pct)}
      </TableCell>
      <TableCell>
        <span className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${getRiskColor(opportunity.risk_score)}`}>
          {formatScore(opportunity.risk_score)}
        </span>
      </TableCell>
      <TableCell>
        {hasFlags && (
          <div className="flex items-center gap-1 text-yellow-600">
            <AlertTriangle className="h-4 w-4" />
            <span className="text-xs">{opportunity.risk_flags.length}</span>
          </div>
        )}
      </TableCell>
    </TableRow>
  );
}

export function OpportunitiesTable() {
  const { data, error, isLoading } = useSWR(
    'opportunities',
    () => getOpportunities({ limit: 50 }),
    {
      refreshInterval: 30000,
      revalidateOnFocus: false,
    }
  );

  if (isLoading) return <TableSkeleton />;
  if (error) return <ErrorState error={error} />;
  if (!data || data.opportunities.length === 0) {
    return (
      <div className="flex items-center justify-center p-8 text-muted-foreground">
        <TrendingUp className="mr-2 h-5 w-5" />
        <span>No opportunities available</span>
      </div>
    );
  }

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Market ID</TableHead>
            <TableHead>Side</TableHead>
            <TableHead>Entry Price</TableHead>
            <TableHead>Expected Payout</TableHead>
            <TableHead>Position %</TableHead>
            <TableHead>Risk Score</TableHead>
            <TableHead>Flags</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.opportunities.map((opp) => (
            <OpportunityRow key={opp.market_id} opportunity={opp} />
          ))}
        </TableBody>
      </Table>
      <div className="border-t px-4 py-3 text-xs text-muted-foreground">
        Showing {data.opportunities.length} of {data.total} opportunities
      </div>
    </div>
  );
}
