<template>
	<div class="control-panel">
		<!-- Header -->
		<div class="panel-header">
			<h2>Lighting Control</h2>
			<button class="refresh-btn" :disabled="effects.loading" @click="handleRefreshEffects" title="Refresh">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
					<path d="M21 3v5h-5" />
					<path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
					<path d="M3 21v-5h5" />
				</svg>
			</button>
		</div>

		<!-- Current Status -->
		<div class="status-section">
			<div class="status-row">
				<div class="status-item">
					<span class="status-label">Effect</span>
					<span class="status-value">{{ effects.currentEffectName }}</span>
				</div>
				<div class="status-item">
					<span class="status-label">Color Mode</span>
					<span class="status-value">{{ colors.currentModeInfo?.label || colors.currentMode }}</span>
				</div>
			</div>
			<div v-if="effects.isTransitioning" class="transition-indicator">
				<div class="transition-bar">
					<div class="transition-progress" :style="{ width: `${effects.transitionProgress * 100}%` }"></div>
				</div>
				<span class="transition-text">{{ Math.round(effects.transitionProgress * 100) }}%</span>
			</div>
		</div>

		<!-- Effects Control -->
		<div class="control-section">
			<div class="section-header">
				<h3>Effects</h3>
				<span class="effect-count">{{ effects.availableEffects.length }} available</span>
			</div>
			<div class="effects-grid">
				<button
					v-for="effect in effects.availableEffects"
					:key="effect.id"
					class="effect-btn"
					:class="{ active: effects.currentEffect?.id === effect.id }"
					:disabled="effects.loading || effects.isTransitioning"
					@click="handleEffectSelect(effect.id)"
					@mouseenter="handleEffectHover(effect.id)"
				>
					{{ effect.name }}
				</button>
			</div>
		</div>

		<!-- Color Control -->
		<div class="control-section">
			<div class="section-header">
				<h3>Color Modes</h3>
				<div v-if="colors.isCustomMode" class="color-preview" :style="colors.colorPreviewStyle">
					<span class="hex-value">{{ colors.hexColor }}</span>
				</div>
			</div>

			<div class="modes-container">
				<div class="modes-grid">
					<button
						v-for="mode in colors.availableModes"
						:key="mode.value"
						class="mode-btn"
						:class="{ active: colors.currentMode === mode.value }"
						:disabled="colors.loading"
						@click="handleModeSelect(mode.value)"
					>
						<span class="mode-emoji">{{ mode.emoji }}</span>
						<span class="mode-label">{{ mode.label }}</span>
					</button>
				</div>

				<!-- Custom Color Controls -->
				<div v-if="colors.isCustomMode" class="custom-controls">
					<div class="color-sliders">
						<div v-for="channel in colorChannels" :key="channel.key" class="slider-group">
							<div class="slider-header">
								<span class="channel-name">{{ channel.name }}</span>
								<span class="channel-value">{{
									Math.round(colors.customColor[channel.key] * 255)
								}}</span>
							</div>
							<input
								:value="colors.customColor[channel.key]"
								type="range"
								min="0"
								max="1"
								step="0.01"
								class="color-slider"
								:class="`slider-${channel.key}`"
								:disabled="colors.loading"
								@input="
									handleColorChange(
										channel.key,
										Number(($event.target as HTMLInputElement)?.value || 0)
									)
								"
							/>
						</div>
					</div>

					<div class="hex-controls">
						<div class="hex-input-group">
							<input
								:value="hexInputValue"
								type="text"
								class="hex-input"
								placeholder="#FF8000"
								:disabled="colors.loading"
								@input="handleHexInputChange"
							/>
							<button class="apply-btn" :disabled="colors.loading || !isValidHex" @click="applyHexColor">
								Apply
							</button>
						</div>
						<button class="reset-btn" :disabled="colors.loading" @click="resetToDefault">Reset</button>
					</div>
				</div>
			</div>
		</div>

		<!-- Effect Details (si disponible) -->
		<div v-if="effects.effectInfo" class="details-section">
			<div class="section-header">
				<h3>Effect Details</h3>
			</div>
			<div class="details-content">
				<div class="detail-item">
					<span class="detail-label">Name</span>
					<span class="detail-value">{{ effects.effectInfo.name }}</span>
				</div>
				<div class="detail-item">
					<span class="detail-label">Description</span>
					<span class="detail-value">{{ effects.effectInfo.description }}</span>
				</div>
				<div v-if="effects.effectInfo.performance_impact" class="detail-item">
					<span class="detail-label">Performance</span>
					<span class="detail-value">{{ effects.effectInfo.performance_impact }}</span>
				</div>
				<div class="detail-item">
					<span class="detail-label">Transitions</span>
					<span class="detail-value">{{
						effects.effectInfo.supports_transitions ? 'Supported' : 'Not supported'
					}}</span>
				</div>
				<div class="detail-item">
					<span class="detail-label">Custom Colors</span>
					<span class="detail-value">{{
						effects.effectInfo.supports_custom_colors ? 'Supported' : 'Not supported'
					}}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, ref, watch } from 'vue';
	import { useColors } from '../composables/useColors';
	import { useEffects } from '../composables/useEffects';
	import type { CustomColor } from '../types';

	interface ColorChannel {
		key: keyof CustomColor;
		name: string;
	}

	// Composables
	const colors = useColors();
	const effects = useEffects();

	// Local state
	const hexInputValue = ref(colors.hexColor);

	// Color channels configuration
	const colorChannels: ColorChannel[] = [
		{ key: 'r', name: 'Red' },
		{ key: 'g', name: 'Green' },
		{ key: 'b', name: 'Blue' },
	];

	// Computed properties
	const isValidHex = computed(() => {
		return /^#[0-9A-Fa-f]{6}$/.test(hexInputValue.value);
	});

	// Watch for hex color changes from store to sync input
	watch(
		() => colors.hexColor,
		(newHex) => {
			hexInputValue.value = newHex;
		}
	);

	// Effect handlers
	const handleEffectSelect = async (effectId: number): Promise<void> => {
		try {
			await effects.setEffect(effectId);
		} catch (error) {
			console.error('Failed to select effect:', error);
		}
	};

	const handleEffectHover = async (effectId: number): Promise<void> => {
		try {
			await effects.getEffectInfo(effectId);
		} catch (error) {
			console.error('Failed to get effect info:', error);
		}
	};

	const handleRefreshEffects = async (): Promise<void> => {
		try {
			await effects.getEffectsList();
		} catch (error) {
			console.error('Failed to refresh effects:', error);
		}
	};

	// Color handlers
	const handleModeSelect = async (mode: string): Promise<void> => {
		try {
			await colors.setColorMode(mode);
		} catch (error) {
			console.error('Failed to set color mode:', error);
		}
	};

	const handleColorChange = async (channel: keyof CustomColor, value: number): Promise<void> => {
		try {
			const newColor = { ...colors.customColor };
			newColor[channel] = value;
			await colors.setCustomColor(newColor);
		} catch (error) {
			console.error('Failed to change color:', error);
		}
	};

	const handleHexInputChange = (event: Event): void => {
		const input = event.target as HTMLInputElement;
		hexInputValue.value = input.value;
	};

	const applyHexColor = async (): Promise<void> => {
		if (isValidHex.value) {
			try {
				await colors.setColorFromHex(hexInputValue.value);
			} catch (error) {
				console.error('Failed to apply hex color:', error);
			}
		}
	};

	const resetToDefault = async (): Promise<void> => {
		try {
			await colors.setCustomColor({ r: 1, g: 0.5, b: 0 });
			hexInputValue.value = '#FF8000';
		} catch (error) {
			console.error('Failed to reset color:', error);
		}
	};
</script>

<style scoped>
	.control-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		padding: 1.5rem;
		color: #c9d1d9;
	}

	/* Header */
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
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

	.refresh-btn {
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		padding: 0.5rem;
		color: #7d8590;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.refresh-btn:hover:not(:disabled) {
		background: #30363d;
		color: #c9d1d9;
		transform: rotate(90deg);
	}

	.refresh-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Status Section */
	.status-section {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
		margin-bottom: 1.5rem;
	}

	.status-row {
		display: flex;
		gap: 2rem;
		margin-bottom: 0.75rem;
	}

	.status-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		flex: 1;
	}

	.status-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.status-value {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.transition-indicator {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.transition-bar {
		flex: 1;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		overflow: hidden;
	}

	.transition-progress {
		height: 100%;
		background: #c9d1d9;
		transition: width 0.3s ease;
	}

	.transition-text {
		font-size: 0.75rem;
		color: #7d8590;
		font-weight: 600;
		min-width: 35px;
		font-family: monospace;
	}

	/* Control Sections */
	.control-section {
		margin-bottom: 1.5rem;
	}

	.control-section:last-child {
		margin-bottom: 0;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.section-header h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.effect-count {
		font-size: 0.75rem;
		color: #7d8590;
		background: #21262d;
		padding: 0.25rem 0.5rem;
		border-radius: 12px;
		font-weight: 500;
	}

	/* Effects Grid */
	.effects-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
		gap: 0.75rem;
	}

	.effect-btn {
		padding: 0.75rem 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: center;
	}

	.effect-btn:hover:not(:disabled) {
		background: #21262d;
		border-color: #30363d;
	}

	.effect-btn.active {
		background: #21262d;
		border-color: #c9d1d9;
		color: #c9d1d9;
		box-shadow: 0 0 0 1px rgba(201, 209, 217, 0.3);
	}

	.effect-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Color Preview */
	.color-preview {
		width: 50px;
		height: 24px;
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

	/* Modes */
	.modes-container {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.modes-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
		gap: 0.75rem;
	}

	.mode-btn {
		padding: 0.75rem 0.5rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		color: #c9d1d9;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
	}

	.mode-btn:hover:not(:disabled) {
		background: #21262d;
		border-color: #30363d;
	}

	.mode-btn.active {
		background: #21262d;
		border-color: #c9d1d9;
		box-shadow: 0 0 0 1px rgba(201, 209, 217, 0.3);
	}

	.mode-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.mode-emoji {
		font-size: 1.25rem;
	}

	.mode-label {
		font-size: 0.75rem;
		font-weight: 500;
		text-align: center;
	}

	/* Custom Controls */
	.custom-controls {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.color-sliders {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.slider-group {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.slider-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.channel-name {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.channel-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #c9d1d9;
		background: #21262d;
		padding: 0.2rem 0.4rem;
		border-radius: 3px;
		min-width: 35px;
		text-align: center;
		font-family: monospace;
	}

	.color-slider {
		width: 100%;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		outline: none;
		appearance: none;
		-webkit-appearance: none;
		cursor: pointer;
	}

	.color-slider::-webkit-slider-thumb {
		appearance: none;
		-webkit-appearance: none;
		width: 16px;
		height: 16px;
		background: #c9d1d9;
		border-radius: 50%;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.color-slider::-webkit-slider-thumb:hover {
		background: #e6edf3;
		transform: scale(1.1);
	}

	.slider-r::-webkit-slider-thumb {
		background: #ff6b6b;
	}
	.slider-g::-webkit-slider-thumb {
		background: #51cf66;
	}
	.slider-b::-webkit-slider-thumb {
		background: #339af0;
	}

	/* Hex Controls */
	.hex-controls {
		display: flex;
		gap: 0.75rem;
		align-items: flex-end;
	}

	.hex-input-group {
		flex: 1;
		display: flex;
		gap: 0.5rem;
	}

	.hex-input {
		flex: 1;
		padding: 0.5rem 0.75rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-family: monospace;
		font-size: 0.875rem;
		transition: all 0.2s ease;
	}

	.hex-input:focus {
		outline: none;
		border-color: #c9d1d9;
	}

	.apply-btn,
	.reset-btn {
		padding: 0.5rem 1rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.apply-btn:hover:not(:disabled),
	.reset-btn:hover:not(:disabled) {
		background: #30363d;
		border-color: #c9d1d9;
	}

	.apply-btn:disabled,
	.reset-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Details Section */
	.details-section {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
	}

	.details-content {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.detail-item {
		display: flex;
		gap: 1rem;
		align-items: flex-start;
	}

	.detail-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #7d8590;
		min-width: 70px;
		flex-shrink: 0;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.detail-value {
		font-size: 0.875rem;
		color: #c9d1d9;
		flex: 1;
		line-height: 1.4;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.control-panel {
			padding: 1rem;
		}

		.status-row {
			flex-direction: column;
			gap: 0.75rem;
		}

		.effects-grid {
			grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
		}

		.modes-grid {
			grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
		}

		.hex-controls {
			flex-direction: column;
		}

		.detail-item {
			flex-direction: column;
			gap: 0.25rem;
		}

		.detail-label {
			min-width: auto;
		}
	}
</style>
