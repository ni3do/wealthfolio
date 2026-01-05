import { useQuery } from "@tanstack/react-query";
import { BrokerConnection } from "@/lib/types";
import { getBrokerConnections } from "@/commands/broker-connections";
import { QueryKeys } from "@/lib/query-keys";

export function useBrokerConnections() {
  const {
    data: connections = [],
    isLoading,
    isError,
    error,
  } = useQuery<BrokerConnection[], Error>({
    queryKey: [QueryKeys.BROKER_CONNECTIONS],
    queryFn: getBrokerConnections,
  });

  return { connections, isLoading, isError, error };
}
