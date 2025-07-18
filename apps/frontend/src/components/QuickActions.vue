<template>
  <div class="panel quick-actions">
    <button
      class="action-btn primary"
      :disabled="loading"
      @click="handleConnect"
    >
      <span class="btn-icon">{{
        loading ? "⏳" : isConnected ? "✅" : "🔌"
      }}</span>
      <span>{{ isConnected ? "Disconnect" : "Connect" }}</span>
    </button>

    <button
      class="action-btn secondary"
      :disabled="!isConnected || loading"
      @click="handlePing"
    >
      <span class="btn-icon">🏓</span>
      <span>Ping ({{ pingMs }}ms)</span>
    </button>

    <button class="action-btn accent" :disabled="loading" @click="handleStream">
      <span class="btn-icon">📡</span>
      <span>Stream ({{ fps }} fps)</span>
    </button>
  </div>
</template>

<script setup>
const props = defineProps({
  isConnected: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  pingMs: {
    type: Number,
    default: 0,
  },
  fps: {
    type: Number,
    default: 0,
  },
});

const emit = defineEmits(["connect", "disconnect", "ping", "stream"]);

const handleConnect = () => {
  if (props.isConnected) {
    emit("disconnect");
  } else {
    emit("connect");
  }
};

const handlePing = () => {
  emit("ping");
};

const handleStream = () => {
  emit("stream");
};
</script>

<style scoped>
.panel {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}

.panel:hover {
  border-color: #484f58;
}

.quick-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem 1.5rem;
  border: 1px solid #30363d;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: #21262d;
  color: #f0f6fc;
}

.action-btn.primary {
  background: #238636;
  border-color: #2ea043;
  color: white;
}

.action-btn.secondary {
  background: #1f6feb;
  border-color: #388bfd;
  color: white;
}

.action-btn.accent {
  background: #da3633;
  border-color: #f85149;
  color: white;
}

.action-btn:hover:not(:disabled) {
  border-color: #484f58;
  background: #30363d;
}

.action-btn.primary:hover:not(:disabled) {
  background: #2ea043;
}

.action-btn.secondary:hover:not(:disabled) {
  background: #388bfd;
}

.action-btn.accent:hover:not(:disabled) {
  background: #f85149;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon {
  font-size: 1rem;
}

@media (max-width: 768px) {
  .quick-actions {
    flex-direction: column;
  }
}
</style>
