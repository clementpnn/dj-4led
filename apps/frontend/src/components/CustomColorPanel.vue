<template>
    <div class="panel color-panel">
        <div class="panel-header">
            <h2>🎨 Custom Color</h2>
            <div class="panel-subtitle">Create your own color</div>
        </div>

        <div class="color-workspace">
            <div class="color-preview-large" :style="colorPreviewStyle">
                <div class="color-info">{{ hexValue }}</div>
            </div>

            <div class="color-sliders">
                <div v-for="channel in colorChannels" :key="channel.key" class="slider-control">
                    <div class="slider-label">
                        <span class="color-emoji">{{ channel.emoji }}</span>
                        <span class="color-name">{{ channel.name }}</span>
                        <span class="color-value">{{ Math.round(customColor[channel.key] * 255) }}</span>
                    </div>
                    <div class="slider-wrapper">
                        <input
                            v-model.number="customColor[channel.key]"
                            type="range"
                            min="0"
                            max="1"
                            step="0.01"
                            :class="['slider', channel.key]"
                        />
                    </div>
                </div>
            </div>

            <button class="apply-color-btn" :disabled="!isConnected || loading" @click="handleApplyColor">
                ✨ Apply Color
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import type { ColorChannel } from '../utils/constants';

interface CustomColor {
    r: number;
    g: number;
    b: number;
}

interface Props {
    customColor: CustomColor;
    colorChannels: ColorChannel[];
    colorPreviewStyle: Record<string, string>;
    isConnected: boolean;
    loading: boolean;
}

interface Emits {
    (e: 'color-apply'): void;
    (e: 'color-update', color: CustomColor): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const customColor = computed({
    get: () => props.customColor,
    set: (value: CustomColor) => emit('color-update', value),
});

const hexValue = computed(() => {
    const r = Math.round(props.customColor.r * 255);
    const g = Math.round(props.customColor.g * 255);
    const b = Math.round(props.customColor.b * 255);
    return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b
        .toString(16)
        .padStart(2, '0')}`.toUpperCase();
});

const handleApplyColor = (): void => {
    emit('color-apply');
};
</script>

<script lang="ts">
import { computed } from 'vue';
</script>

<style scoped>
.panel {
    background: #161b23;
    border: 1px solid #333;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}

.panel:hover {
    border-color: #555;
}

.panel-header {
    margin-bottom: 1.5rem;
}

.panel-header h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #fff;
}

.panel-subtitle {
    color: #aaa;
    font-size: 0.875rem;
}

.color-workspace {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: 1.5rem;
    align-items: start;
}

.color-preview-large {
    max-width: 100px;
    height: 80px;
    border-radius: 10px;
    border: 2px solid #333;
    position: relative;
    transition: all 0.3s ease;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding: 0.5rem;
}

.color-preview-large:hover {
    border-color: #555;
    transform: scale(1.02);
}

.color-info {
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 600;
}

.color-sliders {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.slider-control {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.slider-label {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-weight: 500;
    font-size: 0.9rem;
}

.color-emoji {
    font-size: 1.2rem;
}

.color-name {
    min-width: 60px;
    font-weight: 600;
    color: #fff;
}

.color-value {
    margin-left: auto;
    color: #fff;
    font-family: monospace;
    font-size: 0.85rem;
    background: #2a2a2a;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #333;
    min-width: 50px;
    text-align: center;
}

.slider-wrapper {
    position: relative;
    padding: 0.25rem 0;
}

.slider {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    outline: none;
    cursor: pointer;
    -webkit-appearance: none;
    appearance: none;
    border: none;
    transition: all 0.2s ease;
}

.slider.r {
    background: linear-gradient(to right, #2a2a2a, #ff5555);
}

.slider.g {
    background: linear-gradient(to right, #2a2a2a, #55ff55);
}

.slider.b {
    background: linear-gradient(to right, #2a2a2a, #5555ff);
}

.slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    border: 2px solid #333;
    cursor: pointer;
    transition: all 0.2s ease;
}

.slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
    border-color: #555;
}

.slider::-moz-range-thumb {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    border: 2px solid #333;
    cursor: pointer;
    transition: all 0.2s ease;
}

.slider::-moz-range-thumb:hover {
    transform: scale(1.1);
    border-color: #555;
}

.apply-color-btn {
    grid-column: 1 / -1;
    margin-top: 1.5rem;
    padding: 1rem 2rem;
    border: 1px solid #333;
    border-radius: 8px;
    background: linear-gradient(135deg, #2a2a2a, #3a3a3a);
    color: #fff;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
}

.apply-color-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, #3a3a3a, #4a4a4a);
    transform: translateY(-2px);
    border-color: #555;
}

.apply-color-btn:active {
    transform: translateY(0);
}

.apply-color-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
}

@media (max-width: 768px) {
    .color-workspace {
        grid-template-columns: 1fr;
        text-align: center;
        gap: 1rem;
    }

    .color-preview-large {
        margin: 0 auto;
    }
}
</style>
