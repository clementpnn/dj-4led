<template>
  <div class="terminal">
    <div class="terminal-header">
      <div class="terminal-controls">
        <div class="terminal-dot red"></div>
        <div class="terminal-dot yellow"></div>
        <div class="terminal-dot green"></div>
      </div>
      <div class="terminal-title">DJ-4LED Console</div>
      <button class="clear-btn" @click="handleClearLogs">Clear</button>
    </div>

    <div ref="logContainer" class="terminal-body">
      <div
        v-for="(log, index) in logs"
        :key="index"
        :class="['terminal-line', log.type]"
      >
        <span class="terminal-prompt">$</span>
        <span class="terminal-time">{{ log.time }}</span>
        <span class="terminal-message">{{ log.message }}</span>
      </div>
      <div class="terminal-cursor">_</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, type Ref } from "vue";

interface LogEntry {
  time: string;
  message: string;
  type: "info" | "success" | "error" | "warning";
}

interface Props {
  logs: LogEntry[];
}

interface Emits {
  (e: "clear-logs"): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const logContainer: Ref<HTMLElement | undefined> = ref(undefined);

const handleClearLogs = (): void => {
  emit("clear-logs");
};

defineExpose({
  logContainer,
});
</script>

<style scoped>
.terminal {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 12px;
  overflow: hidden;
  font-family: "SF Mono", Consolas, monospace;
}

.terminal-header {
  display: flex;
  align-items: center;
  padding: 1rem 1.5rem;
  background: #161b22;
  border-bottom: 1px solid #30363d;
}

.terminal-controls {
  display: flex;
  gap: 0.5rem;
}

.terminal-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.terminal-dot.red {
  background: #f85149;
}

.terminal-dot.yellow {
  background: #d29922;
}

.terminal-dot.green {
  background: #2ea043;
}

.terminal-title {
  flex: 1;
  text-align: center;
  color: #f0f6fc;
  font-weight: 500;
  font-size: 0.875rem;
}

.clear-btn {
  background: #21262d;
  border: 1px solid #30363d;
  color: #8b949e;
  cursor: pointer;
  padding: 0.375rem 0.75rem;
  border-radius: 6px;
  font-size: 0.75rem;
  transition: all 0.2s ease;
}

.clear-btn:hover {
  background: #30363d;
  color: #f0f6fc;
}

.terminal-body {
  padding: 1rem 1.5rem;
  max-height: 300px;
  overflow-y: auto;
  font-size: 0.8rem;
  line-height: 1.5;
}

.terminal-line {
  display: flex;
  gap: 0.75rem;
  padding: 0.125rem 0;
  color: #8b949e;
}

.terminal-prompt {
  color: #2ea043;
  font-weight: 600;
}

.terminal-time {
  color: #6e7681;
  min-width: 70px;
}

.terminal-line.success .terminal-message {
  color: #2ea043;
}

.terminal-line.error .terminal-message {
  color: #f85149;
}

.terminal-line.warning .terminal-message {
  color: #d29922;
}

.terminal-line.info .terminal-message {
  color: #58a6ff;
}

.terminal-cursor {
  color: #2ea043;
  margin-top: 0.25rem;
}
</style>
