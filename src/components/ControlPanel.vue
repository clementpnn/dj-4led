<template>
	<div class="control-panel">
		<!-- Header -->
		<div class="panel-header">
			<h2>Lighting Control</h2>
			<button class="refresh-btn" :disabled="isLoading" title="Refresh" @click="refreshAll">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
					<path d="M21 3v5h-5" />
					<path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
					<path d="M3 21v-5h5" />
				</svg>
			</button>
		</div>

		<!-- Brightness Control -->
		<div class="control-section">
			<div class="section-header">
				<h3>Brightness</h3>
				<span class="brightness-value">{{ Math.round(ledStore.brightness * 100) }}%</span>
			</div>
			<div class="brightness-container">
				<input
					type="range"
					class="brightness-slider"
					:value="ledStore.brightness"
					min="0"
					max="1"
					step="0.01"
					:disabled="isLoading"
					@input="handleBrightnessChange"
				/>
			</div>
			<div class="brightness-presets">
				<button
					v-for="preset in brightnessPresets"
					:key="preset.value"
					class="preset-btn"
					:class="{ active: Math.abs(ledStore.brightness - preset.value) < 0.01 }"
					:disabled="isLoading"
					@click="setBrightnessPreset(preset.value)"
				>
					{{ preset.label }}
				</button>
			</div>
		</div>

		<!-- Effects Control -->
		<div class="control-section">
			<div class="section-header">
				<h3>Effects</h3>
				<span class="count-badge">{{ effectsStore.availableEffects.length }}</span>
			</div>
			<div class="effects-grid">
				<button
					v-for="effect in effectsStore.availableEffects"
					:key="effect.id"
					class="effect-btn"
					:class="{ active: effectsStore.currentEffect?.id === effect.id }"
					:disabled="isLoading"
					@click="handleEffectSelect(effect.id)"
				>
					{{ effect.display_name || effect.name }}
				</button>
			</div>
		</div>

		<!-- Color Control -->
		<div class="control-section">
			<div class="section-header">
				<h3>Color Modes</h3>
				<div v-if="colorsStore.isCustomMode" class="color-preview" :style="colorsStore.colorPreviewStyle">
					<span class="hex-value">{{ colorsStore.hexColor }}</span>
				</div>
			</div>

			<div class="modes-grid">
				<button
					v-for="mode in colorsStore.availableModes"
					:key="mode.value"
					class="mode-btn"
					:class="{ active: colorsStore.currentMode === mode.value }"
					:disabled="isLoading"
					@click="handleModeSelect(mode.value)"
				>
					<span class="mode-emoji">{{ mode.emoji }}</span>
					<span class="mode-label">{{ mode.label }}</span>
				</button>
			</div>

			<!-- Custom Color Controls -->
			<div v-if="colorsStore.isCustomMode" class="custom-controls">
				<div class="color-sliders">
					<div v-for="channel in colorChannels" :key="channel.key" class="slider-group">
						<div class="slider-header">
							<span class="channel-name">{{ channel.name }}</span>
							<span class="channel-value">{{
								Math.round(colorsStore.customColor[channel.key] * 255)
							}}</span>
						</div>
						<input
							:value="colorsStore.customColor[channel.key]"
							type="range"
							min="0"
							max="1"
							step="0.01"
							class="color-slider"
							:class="`slider-${channel.key}`"
							:disabled="isLoading"
							@input="
								handleColorChange(channel.key, Number(($event.target as HTMLInputElement)?.value || 0))
							"
						/>
					</div>
				</div>

				<div class="hex-controls">
					<input
						v-model="hexInputValue"
						type="text"
						class="hex-input"
						placeholder="#FF8000"
						:disabled="isLoading"
						@keyup.enter="applyHexColor"
					/>
					<button class="hex-btn" :disabled="isLoading || !isValidHex" @click="applyHexColor">Apply</button>
					<button class="hex-btn" :disabled="isLoading" @click="resetToDefault">Reset</button>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, onMounted, ref, watch } from 'vue';

	import { useColors } from '@/composables/useColors';
	import { useEffects } from '@/composables/useEffects';
	import { useLED } from '@/composables/useLED';
	import { useColorsStore } from '@/stores/colors';
	import { useEffectsStore } from '@/stores/effects';
	import { useLEDStore } from '@/stores/led';
	import type { CustomColor } from '@/types';

	// Stores
	const colorsStore = useColorsStore();
	const effectsStore = useEffectsStore();
	const ledStore = useLEDStore();

	// Composables
	const colorsAPI = useColors();
	const effectsAPI = useEffects();
	const ledAPI = useLED();

	// Local state
	const hexInputValue = ref('#FF8000');
	const isChangingEffect = ref(false);
	const isChangingColor = ref(false);
	const isChangingBrightness = ref(false);

	// Constants
	const brightnessPresets = [
		{ label: '10%', value: 0.1 },
		{ label: '25%', value: 0.25 },
		{ label: '50%', value: 0.5 },
		{ label: '75%', value: 0.75 },
		{ label: '100%', value: 1.0 },
	];

	const colorChannels = [
		{ key: 'r' as keyof CustomColor, name: 'Red' },
		{ key: 'g' as keyof CustomColor, name: 'Green' },
		{ key: 'b' as keyof CustomColor, name: 'Blue' },
	];

	// Computed
	const isLoading = computed(() => isChangingEffect.value || isChangingColor.value || isChangingBrightness.value);

	const isValidHex = computed(() => /^#[0-9A-Fa-f]{6}$/.test(hexInputValue.value));

	// Watch hex color
	watch(
		() => colorsStore.hexColor,
		(newHex) => {
			if (newHex && newHex !== hexInputValue.value) {
				hexInputValue.value = newHex;
			}
		},
		{ immediate: true }
	);

	// Brightness handlers
	const handleBrightnessChange = async (event: Event) => {
		const target = event.target as HTMLInputElement;
		if (!target || isChangingBrightness.value) return;

		isChangingBrightness.value = true;
		try {
			await ledAPI.setBrightness(Number(target.value));
		} finally {
			setTimeout(() => (isChangingBrightness.value = false), 100);
		}
	};

	const setBrightnessPreset = async (value: number) => {
		if (isChangingBrightness.value) return;

		isChangingBrightness.value = true;
		try {
			await ledAPI.setBrightness(value);
		} finally {
			setTimeout(() => (isChangingBrightness.value = false), 100);
		}
	};

	// Effect handlers
	const handleEffectSelect = async (effectId: number) => {
		if (isChangingEffect.value || effectsStore.currentEffect?.id === effectId) return;

		isChangingEffect.value = true;
		try {
			await effectsAPI.setEffect(effectId);
		} finally {
			setTimeout(() => (isChangingEffect.value = false), 300);
		}
	};

	// Color handlers
	const handleModeSelect = async (mode: string) => {
		if (isChangingColor.value || colorsStore.currentMode === mode) return;

		isChangingColor.value = true;
		try {
			await colorsAPI.setColorMode(mode);
		} finally {
			setTimeout(() => (isChangingColor.value = false), 300);
		}
	};

	const handleColorChange = async (channel: keyof CustomColor, value: number) => {
		if (isLoading.value) return;

		const newColor = { ...colorsStore.customColor };
		newColor[channel] = value;
		await colorsAPI.setCustomColor(newColor);
	};

	const applyHexColor = async () => {
		if (!isValidHex.value || isLoading.value) return;
		await colorsAPI.setColorFromHex(hexInputValue.value);
	};

	const resetToDefault = async () => {
		if (isLoading.value) return;
		await colorsAPI.setCustomColor({ r: 1, g: 0.5, b: 0 });
		hexInputValue.value = '#FF8000';
	};

	// Refresh all data
	const refreshAll = async () => {
		await Promise.all([
			effectsAPI.getEffectsList(),
			effectsAPI.getCurrentEffect(),
			colorsAPI.getColorMode(),
			colorsAPI.getCustomColor(),
			ledAPI.getBrightness(),
		]);
	};

	// Initialize
	onMounted(() => {
		setTimeout(refreshAll, 100);
	});
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
	}

	.refresh-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Sections */
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

	/* Brightness */
	.brightness-value {
		font-size: 0.875rem;
		color: #7d8590;
		font-weight: 600;
		font-family: monospace;
	}

	.brightness-container {
		margin-bottom: 1rem;
	}

	.brightness-slider {
		width: 100%;
		height: 4px;
		background: #21262d;
		border-radius: 2px;
		outline: none;
		appearance: none;
		-webkit-appearance: none;
		cursor: pointer;
	}

	.brightness-slider::-webkit-slider-thumb {
		appearance: none;
		width: 16px;
		height: 16px;
		background: #58a6ff;
		border-radius: 50%;
		cursor: pointer;
	}

	.brightness-presets {
		display: flex;
		gap: 0.5rem;
	}

	.preset-btn {
		flex: 1;
		padding: 0.5rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		color: #7d8590;
		font-size: 0.75rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.preset-btn:hover:not(:disabled) {
		background: #21262d;
		color: #c9d1d9;
	}

	.preset-btn.active {
		background: #21262d;
		color: #58a6ff;
		border-color: #30363d;
	}

	/* Effects */
	.count-badge {
		font-size: 0.75rem;
		color: #7d8590;
		background: #161b22;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
	}

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
		color: #58a6ff;
		border-color: #30363d;
	}

	/* Color Modes */
	.color-preview {
		width: 50px;
		height: 24px;
		border: 1px solid #30363d;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.hex-value {
		font-size: 0.625rem;
		font-weight: 600;
		color: #fff;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
		font-family: monospace;
	}

	.modes-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
		gap: 0.75rem;
		margin-bottom: 1rem;
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
		color: #58a6ff;
		border-color: #30363d;
	}

	.mode-emoji {
		font-size: 1.25rem;
	}

	.mode-label {
		font-size: 0.75rem;
		font-weight: 500;
	}

	/* Custom Controls */
	.custom-controls {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 4px;
		padding: 1rem;
	}

	.color-sliders {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-bottom: 1rem;
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
	}

	.channel-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: #c9d1d9;
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
		width: 16px;
		height: 16px;
		border-radius: 50%;
		cursor: pointer;
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
	}

	.hex-input:focus {
		outline: none;
		border-color: #58a6ff;
	}

	.hex-btn {
		padding: 0.5rem 1rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 4px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.hex-btn:hover:not(:disabled) {
		background: #30363d;
	}

	.hex-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.control-panel {
			padding: 1rem;
		}

		.effects-grid,
		.modes-grid {
			grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
		}

		.hex-controls {
			flex-direction: column;
		}

		.brightness-presets {
			flex-wrap: wrap;
		}
	}
</style>
