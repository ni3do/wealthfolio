import { Dialog, DialogContent } from "@/components/ui/dialog";
import { useIsMobileViewport } from "@/hooks/use-platform";
import type { BrokerConnection } from "@/lib/types";
import { ConnectionForm } from "./connection-form";

export interface ConnectionEditModalProps {
  connection?: BrokerConnection;
  open?: boolean;
  onClose?: () => void;
}

export function ConnectionEditModal({ connection, open, onClose }: ConnectionEditModalProps) {
  const defaultValues = connection
    ? {
        id: connection.id,
        name: connection.name,
        accountId: connection.accountId || null,
        queryId: connection.config.queryId,
        token: "", // Don't populate token for security
        syncInterval: connection.config.syncInterval,
        autoImport: connection.config.autoImport,
      }
    : undefined;

  return (
    <Dialog open={open} onOpenChange={onClose} useIsMobile={useIsMobileViewport}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-[625px]">
        <ConnectionForm defaultValues={defaultValues} onSuccess={onClose} />
      </DialogContent>
    </Dialog>
  );
}
