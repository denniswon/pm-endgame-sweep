import { Suspense } from 'react';
import { OpportunitiesTable } from '@/components/features/opportunities-table';
import { Skeleton } from '@/components/ui/skeleton';

export default function Home() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Opportunities</h2>
        <p className="text-muted-foreground">
          Live opportunities updated every 30 seconds
        </p>
      </div>

      <Suspense fallback={<OpportunitiesSkeleton />}>
        <OpportunitiesTable />
      </Suspense>
    </div>
  );
}

function OpportunitiesSkeleton() {
  return (
    <div className="space-y-2">
      {[...Array(5)].map((_, i) => (
        <Skeleton key={i} className="h-16 w-full" />
      ))}
    </div>
  );
}
