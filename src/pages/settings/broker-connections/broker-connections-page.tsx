import { useState } from "react";
import { Button, EmptyPlaceholder, Icons, Separator, Skeleton } from "@wealthfolio/ui";
import { useBrokerConnections } from "@/hooks/use-broker-connections";
import type { BrokerConnection } from "@/lib/types";
import { SettingsHeader } from "../settings-header";
import { ConnectionItem } from "./components/connection-item";
import { ConnectionEditModal } from "./components/connection-edit-modal";
import { useBrokerConnectionMutations } from "./components/use-broker-connection-mutations";

const BrokerConnectionsPage = () => {
  const { connections, isLoading } = useBrokerConnections();
  const [visibleModal, setVisibleModal] = useState(false);
  const [selectedConnection, setSelectedConnection] = useState<BrokerConnection | null>(null);
  const [syncingConnectionId, setSyncingConnectionId] = useState<string | null>(null);

  const { deleteMutation, toggleMutation, syncMutation } = useBrokerConnectionMutations({});

  const handleAddConnection = () => {
    setSelectedConnection(null);
    setVisibleModal(true);
  };

  const handleEditConnection = (connection: BrokerConnection) => {
    setSelectedConnection(connection);
    setVisibleModal(true);
  };

  const handleDeleteConnection = (connection: BrokerConnection) => {
    deleteMutation.mutate(connection.id);
  };

  const handleToggleConnection = (connection: BrokerConnection) => {
    toggleMutation.mutate({
      connectionId: connection.id,
      isActive: !connection.isActive,
    });
  };

  const handleSyncConnection = (connection: BrokerConnection) => {
    setSyncingConnectionId(connection.id);
    syncMutation.mutate(connection.id, {
      onSettled: () => {
        setSyncingConnectionId(null);
      },
    });
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-12" />
        <Skeleton className="h-12" />
      </div>
    );
  }

  return (
    <>
      <div className="space-y-6">
        <SettingsHeader
          heading="Broker Connections"
          text="Connect to your broker accounts for automatic activity import."
        >
          <>
            <Button
              size="icon"
              className="sm:hidden"
              onClick={handleAddConnection}
              aria-label="Add connection"
            >
              <Icons.Plus className="h-4 w-4" />
            </Button>
            <Button size="sm" className="hidden sm:inline-flex" onClick={handleAddConnection}>
              <Icons.Plus className="mr-2 h-4 w-4" />
              Add IBKR Connection
            </Button>
          </>
        </SettingsHeader>
        <Separator />
        <div className="w-full pt-8">
          {connections?.length ? (
            <div className="divide-border divide-y rounded-md border">
              {connections.map((connection: BrokerConnection) => (
                <ConnectionItem
                  key={connection.id}
                  connection={connection}
                  onEdit={handleEditConnection}
                  onDelete={handleDeleteConnection}
                  onToggle={handleToggleConnection}
                  onSync={handleSyncConnection}
                  isSyncing={syncingConnectionId === connection.id}
                />
              ))}
            </div>
          ) : (
            <EmptyPlaceholder>
              <EmptyPlaceholder.Icon name="Cloud" />
              <EmptyPlaceholder.Title>No broker connections!</EmptyPlaceholder.Title>
              <EmptyPlaceholder.Description>
                You haven&apos;t connected any broker accounts yet. Add an IBKR Flex Query
                connection to automatically import your trading activities.
              </EmptyPlaceholder.Description>
              <Button onClick={handleAddConnection}>
                <Icons.Plus className="mr-2 h-4 w-4" />
                Add IBKR Connection
              </Button>
            </EmptyPlaceholder>
          )}
        </div>
      </div>
      <ConnectionEditModal
        connection={selectedConnection || undefined}
        open={visibleModal}
        onClose={() => setVisibleModal(false)}
      />
    </>
  );
};

export default BrokerConnectionsPage;
