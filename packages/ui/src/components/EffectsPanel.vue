<!-- src/components/EffectsPanel.vue -->
<template>
    <div class="panel effects-panel">
        <div class="panel-header">
            <h2>Effects</h2>
            <div class="panel-subtitle">Choose your LED effect</div>
        </div>

        <div class="effects-grid">
            <button
                v-for="effect in effects"
                :key="effect.id"
                @click="handleEffectSelect(effect.id)"
                class="effect-card"
                :class="{ active: currentEffect === effect.id }"
                :disabled="!isConnected || loading"
            >
                <div class="effect-emoji">{{ effect.emoji }}</div>
                <div class="effect-name">{{ effect.name }}</div>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { Effect } from '../utils/constants';

interface Props {
    effects: Effect[];
    currentEffect: number | null;
    isConnected: boolean;
    loading: boolean;
}

interface Emits {
    (e: 'effect-change', effectId: number): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const handleEffectSelect = (effectId: number): void => {
    emit('effect-change', effectId);
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

.effects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.75rem;
}

.effect-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #0d1117;
    cursor: pointer;
    transition: all 0.2s ease;
}

.effect-card:hover:not(:disabled) {
    border-color: #484f58;
    background: #21262d;
}

.effect-card.active {
    background: #0d4929;
    border-color: #2ea043;
}

.effect-card:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.effect-emoji {
    font-size: 1.75rem;
}

.effect-name {
    font-size: 0.8rem;
    font-weight: 500;
    color: #f0f6fc;
}
</style>
