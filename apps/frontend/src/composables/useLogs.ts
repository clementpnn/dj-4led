import { nextTick, ref } from "vue";

interface LogEntry {
  time: string;
  message: string;
  type: "info" | "success" | "error" | "warning";
}

export function useLogs() {
  const logs = ref<LogEntry[]>([]);
  const logContainer = ref<HTMLElement | undefined>(undefined);

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
    log("📡 Server: udp://127.0.0.1:8081", "info");
  };

  return {
    logs,
    logContainer,
    log,
    clearLogs,
    initLogs,
  };
}
