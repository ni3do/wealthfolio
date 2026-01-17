import { getRunEnv, RUN_ENV, invokeTauri, invokeWeb } from "@/adapters";
import { logger } from "@/adapters";
import type {
  BrokerConnection,
  CreateBrokerConnectionRequest,
  UpdateBrokerConnectionRequest,
  SyncResult,
  ActivityImport,
} from "@/lib/types";

export const getBrokerConnections = async (): Promise<BrokerConnection[]> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("get_broker_connections");
      case RUN_ENV.WEB:
        return invokeWeb("get_broker_connections");
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error fetching broker connections.");
    throw error;
  }
};

export const getBrokerConnection = async (
  connectionId: string
): Promise<BrokerConnection | null> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("get_broker_connection", { connectionId });
      case RUN_ENV.WEB:
        return invokeWeb("get_broker_connection", { connectionId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error fetching broker connection.");
    throw error;
  }
};

export const createBrokerConnection = async (
  request: CreateBrokerConnectionRequest
): Promise<BrokerConnection> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("create_broker_connection", { request });
      case RUN_ENV.WEB:
        return invokeWeb("create_broker_connection", { request });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error creating broker connection.");
    throw error;
  }
};

export const updateBrokerConnection = async (
  connectionId: string,
  request: UpdateBrokerConnectionRequest
): Promise<BrokerConnection> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("update_broker_connection", { connectionId, request });
      case RUN_ENV.WEB:
        return invokeWeb("update_broker_connection", { connectionId, request });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error updating broker connection.");
    throw error;
  }
};

export const deleteBrokerConnection = async (connectionId: string): Promise<void> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        await invokeTauri("delete_broker_connection", { connectionId });
        return;
      case RUN_ENV.WEB:
        await invokeWeb("delete_broker_connection", { connectionId });
        return;
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error deleting broker connection.");
    throw error;
  }
};

export const toggleBrokerConnection = async (
  connectionId: string,
  isActive: boolean
): Promise<BrokerConnection> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("toggle_broker_connection", { connectionId, isActive });
      case RUN_ENV.WEB:
        return invokeWeb("toggle_broker_connection", { connectionId, isActive });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error toggling broker connection.");
    throw error;
  }
};

export const testBrokerConnection = async (connectionId: string): Promise<boolean> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("test_broker_connection", { connectionId });
      case RUN_ENV.WEB:
        return invokeWeb("test_broker_connection", { connectionId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error testing broker connection.");
    throw error;
  }
};

export const syncBrokerConnection = async (
  connectionId: string
): Promise<SyncResult> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("sync_broker_connection", { connectionId });
      case RUN_ENV.WEB:
        return invokeWeb("sync_broker_connection", { connectionId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error syncing broker connection.");
    throw error;
  }
};

export const syncBrokerConnectionForReview = async (
  connectionId: string
): Promise<ActivityImport[]> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("sync_broker_connection_for_review", { connectionId });
      case RUN_ENV.WEB:
        return invokeWeb("sync_broker_connection_for_review", { connectionId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error syncing broker connection for review.");
    throw error;
  }
};
