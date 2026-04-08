import { Skeleton } from "@/components/ui/skeleton";

export default function ModelsLoading() {
  return (
    <div className="flex flex-col gap-6">
      <div className="space-y-2">
        <Skeleton className="h-7 w-40" />
        <Skeleton className="h-4 w-96" />
      </div>
      <div className="flex flex-col gap-6 lg:flex-row">
        {/* Filter sidebar skeleton */}
        <div className="w-full shrink-0 lg:w-56">
          <Skeleton className="h-80 rounded-md" />
        </div>
        {/* Grid skeleton */}
        <div className="min-w-0 flex-1">
          <Skeleton className="mb-4 h-4 w-20" />
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-40 rounded-md" />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
