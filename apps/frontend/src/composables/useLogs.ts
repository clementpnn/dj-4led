// src/composables/useLogs.ts
import { nextTick, ref } from "vue";
import { useConfig } from "./useConfig";

interface LogEntry {
  time: string;
  message: string;
  type: "info" | "success" | "error" | "warning";
}

export function useLogs() {
  const logs = ref<LogEntry[]>([]);
  const logContainer = ref<HTMLElement | null>(null);
  const { serverAddress } = useConfig();

  const log = (message: string, type: LogEntry["type"] = "info"): void => {
    logs.value.push({
      time: new Date().toLocaleTimeString(),
      message,
      type,
    });
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight;
      }
    });
  };

  const clearLogs = (): void => {
    logs.value = [];
  };

  const initLogs = (): void => {
    log("🎵 DJ-4LED Controller ready!", "info");
    log(`📡 Server: udp://${serverAddress.value}`, "info");
  };

  return {
    logs,
    logContainer,
    log,
    clearLogs,
    initLogs,
  };
}
