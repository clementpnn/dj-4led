// src/composables/useConnection.ts
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { useConfig } from "./useConfig";

interface ConnectionResult {
  success: boolean;
  message: string;
  pingMs?: number;
}

export function useConnection() {
  const isConnected = ref<boolean>(false);
  const loading = ref<boolean>(false);
  const pingMs = ref<number>(0);

  const connect = async (): Promise<ConnectionResult> => {
    loading.value = true;
    try {
      const { config } = useConfig();
      const result = await invoke<string>("dj_connect", {
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      if (result.includes("✅")) {
        isConnected.value = true;
        return { success: true, message: result };
      } else {
        return { success: false, message: result };
      }
    } catch (error) {
      return { success: false, message: `❌ Error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const disconnect = async (): Promise<ConnectionResult> => {
    loading.value = true;
    try {
      const { config } = useConfig();
      const result = await invoke<string>("dj_disconnect", {
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      if (result.includes("✅")) {
        isConnected.value = false;
        pingMs.value = 0;
        return { success: true, message: result };
      } else {
        return { success: false, message: result };
      }
    } catch (error) {
      return { success: false, message: `❌ Disconnect error: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  const ping = async (): Promise<ConnectionResult> => {
    loading.value = true;
    const startTime = performance.now();
    try {
      const { config } = useConfig();
      const result = await invoke<string>("dj_ping", {
        serverIp: config.value.server.ip,
        serverPort: config.value.server.port,
      });
      const endTime = performance.now();
      pingMs.value = Math.round(endTime - startTime);
      return {
        success: result.includes("🏓"),
        message: result,
        pingMs: pingMs.value,
      };
    } catch (error) {
      pingMs.value = 0;
      return { success: false, message: `❌ Ping failed: ${error}` };
    } finally {
      loading.value = false;
    }
  };

  return {
    isConnected,
    loading,
    pingMs,
    connect,
    disconnect,
    ping,
  };
}
