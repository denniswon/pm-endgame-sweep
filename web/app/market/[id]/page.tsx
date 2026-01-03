'use client';

import { use } from 'react';
import useSWR from 'swr';
import Link from 'next/link';
import { ArrowLeft, AlertTriangle } from 'lucide-react';
import { getMarket } from '@/lib/api';
import { MarketCard } from '@/components/features/market-card';
import { Skeleton } from '@/components/ui/skeleton';

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <Skeleton className="h-8 w-24" />
      <Skeleton className="h-64 w-full" />
      <Skeleton className="h-48 w-full" />
      <Skeleton className="h-48 w-full" />
    </div>
  );
}

function ErrorState({ error }: { error: Error }) {
  return (
    <div className="space-y-4">
      <Link
        href="/"
        className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to opportunities
      </Link>
      <div className="flex items-center justify-center p-8 text-destructive">
        <AlertTriangle className="mr-2 h-5 w-5" />
        <span>Failed to load market details: {error.message}</span>
      </div>
    </div>
  );
}

export default function MarketPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);

  const { data, error, isLoading } = useSWR(
    `market-${id}`,
    () => getMarket(id),
    {
      refreshInterval: 30000,
      revalidateOnFocus: false,
    }
  );

  if (isLoading) return <LoadingSkeleton />;
  if (error) return <ErrorState error={error} />;
  if (!data) return <ErrorState error={new Error('No data returned')} />;

  return (
    <div className="space-y-6">
      <Link
        href="/"
        className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to opportunities
      </Link>

      <MarketCard data={data} />
    </div>
  );
}
