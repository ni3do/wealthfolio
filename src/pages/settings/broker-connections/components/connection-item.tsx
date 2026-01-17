import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Icons } from "@/components/ui/icons";
import type { BrokerConnection } from "@/lib/types";
import { ConnectionOperations } from "./connection-operations";
import { format } from "date-fns";

export interface ConnectionItemProps {
  connection: BrokerConnection;
  onEdit: (connection: BrokerConnection) => void;
  onDelete: (connection: BrokerConnection) => void;
  onToggle: (connection: BrokerConnection) => void;
  onSync: (connection: BrokerConnection) => void;
  isSyncing?: boolean;
}

export function ConnectionItem({
  connection,
  onEdit,
  onDelete,
  onToggle,
  onSync,
  isSyncing = false,
}: ConnectionItemProps) {
  const getSyncStatusBadge = () => {
    if (!connection.lastSyncStatus) return null;

    const variants: Record<string, "default" | "success" | "destructive" | "secondary"> = {
      success: "success",
      error: "destructive",
      in_progress: "secondary",
    };

    return (
      <Badge variant={variants[connection.lastSyncStatus] || "secondary"}>
        {connection.lastSyncStatus === "success" && "Synced"}
        {connection.lastSyncStatus === "error" && "Error"}
        {connection.lastSyncStatus === "in_progress" && "Syncing..."}
      </Badge>
    );
  };

  const getLastSyncText = () => {
    if (!connection.lastSyncAt) return "Never synced";
    try {
      const syncDate = new Date(connection.lastSyncAt);
      return `Last synced: ${format(syncDate, "PPp")}`;
    } catch {
      return "Never synced";
    }
  };

  return (
    <div className="flex items-center justify-between p-4">
      <div className="grid flex-1 gap-1">
        <div className="flex items-center gap-2">
          <span
            className={`font-semibold ${!connection.isActive ? "text-muted-foreground" : ""}`}
          >
            {connection.name}
          </span>
          {!connection.isActive && <Badge variant="secondary">Disabled</Badge>}
          {getSyncStatusBadge()}
        </div>
        <div className="flex items-center gap-2">
          <p className="text-muted-foreground text-sm">
            {connection.brokerType} • {connection.config.syncInterval}
          </p>
        </div>
        <p className="text-muted-foreground text-xs">{getLastSyncText()}</p>
        {connection.lastSyncError && (
          <p className="text-destructive text-xs">Error: {connection.lastSyncError}</p>
        )}
      </div>
      <div className="flex items-center space-x-2">
        {connection.isActive && (
          <Button
            size="sm"
            variant="outline"
            onClick={() => onSync(connection)}
            disabled={isSyncing}
          >
            {isSyncing ? (
              <>
                <Icons.Spinner className="mr-2 h-4 w-4 animate-spin" />
                Syncing...
              </>
            ) : (
              <>
                <Icons.RefreshCw className="mr-2 h-4 w-4" />
                Sync Now
              </>
            )}
          </Button>
        )}
        <ConnectionOperations
          connection={connection}
          onEdit={onEdit}
          onDelete={onDelete}
          onToggle={onToggle}
        />
      </div>
    </div>
  );
}

ConnectionItem.Skeleton = function ConnectionItemSkeleton() {
  return (
    <div className="p-4">
      <div className="space-y-3">
        <Skeleton className="h-5 w-2/5" />
        <Skeleton className="h-4 w-4/5" />
      </div>
    </div>
  );
};
