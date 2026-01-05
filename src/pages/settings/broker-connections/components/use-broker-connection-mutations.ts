import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "@/components/ui/use-toast";
import {
  createBrokerConnection,
  updateBrokerConnection,
  deleteBrokerConnection,
  toggleBrokerConnection,
  testBrokerConnection,
  syncBrokerConnection,
} from "@/commands/broker-connections";
import { QueryKeys } from "@/lib/query-keys";
import { logger } from "@/adapters";
import type {
  CreateBrokerConnectionRequest,
  UpdateBrokerConnectionRequest,
} from "@/lib/types";

interface UseBrokerConnectionMutationsProps {
  onSuccess?: () => void;
}

export function useBrokerConnectionMutations({
  onSuccess = () => undefined,
}: UseBrokerConnectionMutationsProps) {
  const queryClient = useQueryClient();

  const handleSuccess = (message?: string) => {
    onSuccess();
    if (message) {
      toast({ title: message, variant: "success" });
    }
  };

  const handleError = (action: string, error?: unknown) => {
    const errorMessage = error instanceof Error ? error.message : String(error);
    toast({
      title: `Failed to ${action} broker connection`,
      description: errorMessage || "Please try again or report an issue if the problem persists.",
      variant: "destructive",
    });
  };

  const createMutation = useMutation({
    mutationFn: (request: CreateBrokerConnectionRequest) => createBrokerConnection(request),
    onSuccess: () => {
      handleSuccess("IBKR connection created successfully.");
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
    },
    onError: (e) => {
      logger.error(`Error creating broker connection: ${e}`);
      handleError("create", e);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({
      connectionId,
      request,
    }: {
      connectionId: string;
      request: UpdateBrokerConnectionRequest;
    }) => updateBrokerConnection(connectionId, request),
    onSuccess: () => {
      handleSuccess("Connection updated successfully.");
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
    },
    onError: (e) => {
      logger.error(`Error updating broker connection: ${e}`);
      handleError("update", e);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteBrokerConnection,
    onSuccess: () => {
      handleSuccess("Connection deleted successfully.");
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
    },
    onError: (e) => {
      logger.error(`Error deleting broker connection: ${e}`);
      handleError("delete", e);
    },
  });

  const toggleMutation = useMutation({
    mutationFn: ({ connectionId, isActive }: { connectionId: string; isActive: boolean }) =>
      toggleBrokerConnection(connectionId, isActive),
    onSuccess: (_data, variables) => {
      handleSuccess(
        variables.isActive
          ? "Connection enabled successfully."
          : "Connection disabled successfully."
      );
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
    },
    onError: (e) => {
      logger.error(`Error toggling broker connection: ${e}`);
      handleError("toggle", e);
    },
  });

  const testMutation = useMutation({
    mutationFn: testBrokerConnection,
    onSuccess: () => {
      toast({
        title: "Connection test successful",
        description: "Your IBKR credentials are valid.",
        variant: "success",
      });
    },
    onError: (e) => {
      logger.error(`Error testing broker connection: ${e}`);
      handleError("test", e);
    },
  });

  const syncMutation = useMutation({
    mutationFn: syncBrokerConnection,
    onSuccess: (result) => {
      if (result.success) {
        toast({
          title: "Sync completed successfully",
          description: `Imported ${result.activitiesCount} activities.`,
          variant: "success",
        });
        queryClient.invalidateQueries({ queryKey: [QueryKeys.ACTIVITIES] });
        queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
      } else {
        toast({
          title: "Sync failed",
          description: result.error || "Unknown error occurred",
          variant: "destructive",
        });
      }
    },
    onError: (e) => {
      logger.error(`Error syncing broker connection: ${e}`);
      handleError("sync", e);
    },
  });

  return {
    createMutation,
    updateMutation,
    deleteMutation,
    toggleMutation,
    testMutation,
    syncMutation,
  };
}
