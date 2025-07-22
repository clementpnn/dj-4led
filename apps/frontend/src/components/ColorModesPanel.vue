<template>
    <div class="panel modes-panel">
        <div class="panel-header">
            <h2>🌈 Color Modes</h2>
            <div class="panel-subtitle">Select color pattern</div>
        </div>

        <div class="modes-grid">
            <button
                v-for="mode in colorModes"
                :key="mode.value"
                class="mode-card"
                :class="{ active: currentMode === mode.value }"
                :disabled="!isConnected || loading"
                @click="handleModeSelect(mode.value)"
            >
                <span class="mode-emoji">{{ mode.emoji }}</span>
                <span class="mode-label">{{ mode.label }}</span>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { ColorMode } from '../utils/constants';

interface Props {
    colorModes: ColorMode[];
    currentMode: string | undefined;
    isConnected: boolean;
    loading: boolean;
}

interface Emits {
    (e: 'mode-change', mode: string): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const handleModeSelect = (mode: string): void => {
    emit('mode-change', mode);
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

.panel-header {
    margin-bottom: 1.5rem;
}

.panel-header h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #f0f6fc;
}

.panel-subtitle {
    color: #8b949e;
    font-size: 0.875rem;
}

.modes-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
}

.mode-card {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #0d1117;
    cursor: pointer;
    transition: all 0.2s ease;
    flex: 1;
    min-width: 120px;
    font-size: 0.875rem;
}

.mode-card:hover:not(:disabled) {
    border-color: #484f58;
    background: #21262d;
}

.mode-card.active {
    background: #0d4929;
    border-color: #2ea043;
}

.mode-card:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.mode-emoji {
    font-size: 1.2rem;
}

.mode-label {
    font-weight: 500;
}

@media (max-width: 768px) {
    .modes-grid {
        flex-direction: column;
    }

    .mode-card {
        min-width: auto;
    }
}
</style>
