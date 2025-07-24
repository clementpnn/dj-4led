<template>
	<div class="colors-panel">
		<div class="panel-header">
			<h2>Color Modes</h2>
		</div>

		<!-- Mode Selection -->
		<div class="modes-section">
			<div class="modes-grid">
				<button
					v-for="mode in availableModes"
					:key="mode.value"
					class="mode-btn"
					:class="{ active: currentMode === mode.value }"
					:disabled="loading"
					@click="handleModeSelect(mode.value)"
				>
					{{ mode.label }}
				</button>
			</div>
		</div>

		<!-- Custom Color Section -->
		<div v-if="isCustomMode" class="custom-section">
			<div class="section-header">
				<span class="section-title">Custom Color</span>
				<div class="color-preview" :style="colorPreviewStyle">
					<span class="hex-value">{{ hexColor }}</span>
				</div>
			</div>

			<div class="color-controls">
				<div v-for="channel in colorChannels" :key="channel.key" class="color-control">
					<div class="control-header">
						<span class="channel-name">{{ channel.name }}</span>
						<span class="channel-value">{{ Math.round(customColor[channel.key] * 255) }}</span>
					</div>
					<input
						:value="customColor[channel.key]"
						type="range"
						min="0"
						max="1"
						step="0.01"
						class="color-slider"
						:disabled="loading"
						@input="handleColorChange(channel.key, Number(($event.target as HTMLInputElement)?.value || 0))"
					/>
				</div>
			</div>

			<div class="custom-actions">
				<button class="action-btn primary" :disabled="loading" @click="$emit('apply-custom-color')">
					Apply
				</button>
				<button class="action-btn secondary" :disabled="loading" @click="resetToDefault">Reset</button>
			</div>
		</div>

		<!-- Current Mode Display -->
		<div class="current-mode">
			<div class="mode-info">
				<span class="mode-label">Current</span>
				<span class="mode-name">{{ currentModeInfo?.label || currentMode }}</span>
			</div>
			<div class="mode-description">
				{{ getModeDescription(currentMode) }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { ColorMode, CustomColor } from '../types';

	interface ColorChannel {
		key: keyof CustomColor;
		name: string;
	}

	interface Props {
		currentMode: string;
		customColor: CustomColor;
		availableModes: ColorMode[];
		loading: boolean;
		colorPreviewStyle: Record<string, string>;
		hexColor: string;
		currentModeInfo: ColorMode | undefined;
		isCustomMode: boolean;
	}

	interface Emits {
		(e: 'mode-change', mode: string): void;
		(e: 'color-change', color: CustomColor): void;
		(e: 'apply-custom-color'): void;
		(e: 'refresh-modes'): void;
		(e: 'save-custom-preset'): void;
	}

	const props = defineProps<Props>();
	const emit = defineEmits<Emits>();

	const colorChannels: ColorChannel[] = [
		{ key: 'r', name: 'Red' },
		{ key: 'g', name: 'Green' },
		{ key: 'b', name: 'Blue' },
	];

	const handleModeSelect = (mode: string): void => {
		emit('mode-change', mode);
	};

	const handleColorChange = (channel: keyof CustomColor, value: number): void => {
		const newColor = { ...props.customColor };
		newColor[channel] = value;
		emit('color-change', newColor);
	};

	const resetToDefault = (): void => {
		emit('color-change', { r: 1, g: 0.5, b: 0 });
	};

	const getModeDescription = (mode: string): string => {
		const descriptions: Record<string, string> = {
			rainbow: 'Smooth rainbow color cycling',
			fire: 'Warm fire-like colors',
			ocean: 'Cool ocean colors',
			forest: 'Natural green tones',
			sunset: 'Warm sunset gradient',
			custom: 'User-defined solid color',
			pulse: 'Pulsing single color',
			gradient: 'Multi-color gradient',
		};
		return descriptions[mode] || 'Dynamic color pattern';
	};
</script>

<style scoped>
	.colors-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		padding: 1.5rem;
		color: #c9d1d9;
	}

	.panel-header {
		margin-bottom: 1.5rem;
		padding-bottom: 0.75rem;
		border-bottom: 1px solid #21262d;
	}

	.panel-header h2 {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.modes-section {
		margin-bottom: 1.5rem;
	}

	.modes-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
		gap: 0.75rem;
	}

	.mode-btn {
		padding: 0.75rem 1rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: center;
	}

	.mode-btn:hover:not(:disabled) {
		background: #30363d;
	}

	.mode-btn.active {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	.mode-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.custom-section {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
		margin-bottom: 1.5rem;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.color-preview {
		width: 50px;
		height: 30px;
		border: 1px solid #30363d;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
	}

	.hex-value {
		font-size: 0.625rem;
		font-weight: 600;
		color: #fff;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
		font-family: monospace;
	}

	.color-controls {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.color-control {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.control-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.channel-name {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		text-transform: uppercase;
	}

	.channel-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #c9d1d9;
		font-family: monospace;
		background: #21262d;
		padding: 0.125rem 0.375rem;
		border-radius: 3px;
		min-width: 35px;
		text-align: center;
	}

	.color-slider {
		width: 100%;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		outline: none;
		appearance: none;
		-webkit-appearance: none;
	}

	.color-slider::-webkit-slider-thumb {
		appearance: none;
		-webkit-appearance: none;
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		cursor: pointer;
	}

	.color-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		border: none;
		cursor: pointer;
	}

	.custom-actions {
		display: flex;
		gap: 0.75rem;
	}

	.action-btn {
		padding: 0.5rem 1rem;
		border: 1px solid #30363d;
		border-radius: 4px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		flex: 1;
	}

	.action-btn.primary {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: #b1bac4;
	}

	.action-btn.secondary {
		background: #21262d;
		color: #c9d1d9;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: #30363d;
	}

	.action-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.current-mode {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
	}

	.mode-info {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.mode-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		text-transform: uppercase;
	}

	.mode-name {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.mode-description {
		font-size: 0.75rem;
		color: #7d8590;
		line-height: 1.4;
	}

	@media (max-width: 768px) {
		.modes-grid {
			grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
		}

		.section-header {
			flex-direction: column;
			gap: 0.75rem;
			align-items: flex-start;
		}

		.custom-actions {
			flex-direction: column;
		}

		.mode-info {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.25rem;
		}
	}
</style>
