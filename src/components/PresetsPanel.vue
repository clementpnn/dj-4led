<template>
	<div class="panel presets-panel">
		<div class="panel-header">
			<h2>🎛️ Presets</h2>
			<div class="panel-subtitle">Save and load your favorite configurations</div>
		</div>

		<!-- Current Preset Display -->
		<div v-if="currentPreset" class="current-preset">
			<div class="current-preset-header">
				<div class="preset-icon">🎯</div>
				<div class="preset-info">
					<div class="preset-name">{{ currentPreset.name }}</div>
					<div class="preset-description">{{ currentPreset.description }}</div>
				</div>
				<div class="preset-actions">
					<button
						class="action-btn secondary small"
						:disabled="loading"
						@click="$emit('duplicate-preset', currentPreset.id)"
						title="Duplicate preset"
					>
						📋
					</button>
				</div>
			</div>
		</div>

		<!-- Create New Preset -->
		<div class="create-preset-section">
			<div class="create-preset-header">
				<h3>✨ Create New Preset</h3>
				<button class="action-btn primary" :disabled="loading" @click="showCreateForm = !showCreateForm">
					{{ showCreateForm ? '❌ Cancel' : '➕ New Preset' }}
				</button>
			</div>

			<div v-if="showCreateForm" class="create-form">
				<div class="form-group">
					<label class="form-label">
						<span class="label-text">Name</span>
						<input
							v-model="newPresetName"
							type="text"
							class="form-input"
							placeholder="Enter preset name..."
							:disabled="loading"
							@keyup.enter="handleCreatePreset"
						/>
					</label>
				</div>
				<div class="form-group">
					<label class="form-label">
						<span class="label-text">Description</span>
						<textarea
							v-model="newPresetDescription"
							class="form-textarea"
							placeholder="Enter description (optional)..."
							:disabled="loading"
							rows="2"
						></textarea>
					</label>
				</div>
				<div class="form-actions">
					<button
						class="action-btn primary"
						:disabled="loading || !newPresetName.trim()"
						@click="handleCreatePreset"
					>
						💾 Save Current Settings
					</button>
					<button class="action-btn secondary" :disabled="loading" @click="resetCreateForm">🔄 Reset</button>
				</div>
			</div>
		</div>

		<!-- Presets Categories -->
		<div class="presets-categories">
			<!-- Default Presets -->
			<div class="category-section">
				<h3 class="category-title">🌟 Default Presets</h3>
				<div class="presets-grid">
					<div
						v-for="preset in defaultPresets"
						:key="preset.id"
						class="preset-card"
						:class="{ active: currentPreset?.id === preset.id }"
					>
						<div class="preset-content">
							<div class="preset-header">
								<div class="preset-title">{{ preset.name }}</div>
								<div class="preset-tags">
									<span v-for="tag in preset.tags?.slice(0, 2)" :key="tag" class="preset-tag">
										{{ tag }}
									</span>
								</div>
							</div>
							<div class="preset-description">{{ preset.description }}</div>
							<div class="preset-config">
								<div class="config-item">
									<span class="config-label">Effect:</span>
									<span class="config-value">{{ preset.config.effect.name }}</span>
								</div>
								<div class="config-item">
									<span class="config-label">Color:</span>
									<span class="config-value">{{ preset.config.color.mode }}</span>
								</div>
								<div class="config-item">
									<span class="config-label">Brightness:</span>
									<span class="config-value"
										>{{ Math.round(preset.config.led.brightness * 100) }}%</span
									>
								</div>
							</div>
						</div>
						<div class="preset-actions">
							<button
								class="action-btn primary small"
								:disabled="loading"
								@click="$emit('apply-preset', preset.id)"
							>
								🚀 Apply
							</button>
							<button
								class="action-btn secondary small"
								:disabled="loading"
								@click="$emit('duplicate-preset', preset.id)"
								title="Duplicate"
							>
								📋
							</button>
							<button
								class="action-btn danger small"
								:disabled="loading"
								@click="$emit('delete-preset', preset.id)"
								title="Delete"
							>
								🗑️
							</button>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Import/Export Section -->
		<div class="import-export-section">
			<h3>📦 Import/Export</h3>
			<div class="import-export-actions">
				<button class="action-btn secondary" :disabled="loading" @click="$emit('export-presets')">
					📤 Export Presets
				</button>
				<label class="action-btn secondary import-btn">
					📥 Import Presets
					<input
						ref="fileInput"
						type="file"
						accept=".json"
						style="display: none"
						:disabled="loading"
						@change="handleFileImport"
					/>
				</label>
			</div>
		</div>

		<!-- Presets Statistics -->
		<div class="presets-stats">
			<div class="stats-grid">
				<div class="stat-item">
					<span class="stat-label">Total:</span>
					<span class="stat-value">{{ allPresets.length }}</span>
				</div>
				<div class="stat-item">
					<span class="stat-label">Custom:</span>
					<span class="stat-value">{{ customPresets.length }}</span>
				</div>
				<div class="stat-item">
					<span class="stat-label">Current:</span>
					<span class="stat-value">{{ currentPreset?.name || 'None' }}</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	import type { Preset } from '../types';

	interface Props {
		allPresets: Preset[];
		customPresets: Preset[];
		currentPreset: Preset | null;
		loading: boolean;
	}

	interface Emits {
		(e: 'apply-preset', presetId: string): void;
		(e: 'create-preset', name: string, description: string): void;
		(e: 'duplicate-preset', presetId: string): void;
		(e: 'delete-preset', presetId: string): void;
		(e: 'export-presets'): void;
		(e: 'import-presets', file: File): void;
	}

	const props = defineProps<Props>();
	const emit = defineEmits<Emits>();

	// Local state
	const showCreateForm = ref(false);
	const newPresetName = ref('');
	const newPresetDescription = ref('');
	const fileInput = ref<HTMLInputElement>();

	// Computed
	const defaultPresets = computed(() =>
		props.allPresets.filter(
			(preset) =>
				preset.tags?.includes('default') ||
				['spectrum-rainbow', 'fire-waves', 'ocean-particles', 'custom-red'].includes(preset.id)
		)
	);

	// Methods
	const handleCreatePreset = (): void => {
		if (!newPresetName.value.trim()) return;

		emit('create-preset', newPresetName.value.trim(), newPresetDescription.value.trim());
		resetCreateForm();
	};

	const resetCreateForm = (): void => {
		newPresetName.value = '';
		newPresetDescription.value = '';
		showCreateForm.value = false;
	};

	const handleFileImport = (event: Event): void => {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (file) {
			emit('import-presets', file);
			target.value = ''; // Reset input
		}
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

	.current-preset {
		margin-bottom: 1.5rem;
		padding: 1rem;
		background: #0d4929;
		border: 1px solid #2ea043;
		border-radius: 8px;
	}

	.current-preset-header {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.preset-icon {
		font-size: 1.5rem;
		flex-shrink: 0;
	}

	.preset-info {
		flex: 1;
	}

	.preset-name {
		font-size: 1rem;
		font-weight: 600;
		color: #f0f6fc;
		margin-bottom: 0.25rem;
	}

	.preset-description {
		font-size: 0.875rem;
		color: #8b949e;
	}

	.create-preset-section {
		margin-bottom: 1.5rem;
	}

	.create-preset-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.create-preset-header h3 {
		margin: 0;
		font-size: 1rem;
		color: #f0f6fc;
	}

	.create-form {
		padding: 1rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
	}

	.form-group {
		margin-bottom: 1rem;
	}

	.form-group:last-child {
		margin-bottom: 0;
	}

	.form-label {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.label-text {
		font-size: 0.875rem;
		font-weight: 500;
		color: #f0f6fc;
	}

	.form-input,
	.form-textarea {
		padding: 0.75rem;
		border: 1px solid #30363d;
		border-radius: 6px;
		background: #161b22;
		color: #f0f6fc;
		font-size: 0.875rem;
		transition: border-color 0.2s ease;
	}

	.form-input:focus,
	.form-textarea:focus {
		outline: none;
		border-color: #58a6ff;
	}

	.form-textarea {
		resize: vertical;
		min-height: 60px;
		font-family: inherit;
	}

	.form-actions {
		display: flex;
		gap: 0.75rem;
		margin-top: 1rem;
	}

	.presets-categories {
		margin-bottom: 1.5rem;
	}

	.category-section {
		margin-bottom: 2rem;
	}

	.category-section:last-child {
		margin-bottom: 0;
	}

	.category-title {
		margin: 0 0 1rem 0;
		font-size: 1rem;
		font-weight: 600;
		color: #f0f6fc;
	}

	.presets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 1rem;
	}

	.preset-card {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		overflow: hidden;
		transition: all 0.2s ease;
	}

	.preset-card:hover {
		border-color: #484f58;
		transform: translateY(-2px);
	}

	.preset-card.active {
		background: #0d4929;
		border-color: #2ea043;
		box-shadow: 0 0 15px rgba(46, 160, 67, 0.3);
	}

	.preset-card.custom {
		border-left: 3px solid #58a6ff;
	}

	.preset-content {
		padding: 1rem;
	}

	.preset-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 0.75rem;
	}

	.preset-title {
		font-size: 1rem;
		font-weight: 600;
		color: #f0f6fc;
	}

	.preset-tags {
		display: flex;
		gap: 0.25rem;
		flex-wrap: wrap;
	}

	.preset-tag {
		padding: 0.125rem 0.375rem;
		background: #30363d;
		color: #8b949e;
		border-radius: 12px;
		font-size: 0.625rem;
		font-weight: 500;
		text-transform: uppercase;
	}

	.preset-meta {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
	}

	.preset-date {
		font-size: 0.75rem;
		color: #8b949e;
		font-family: 'SF Mono', Consolas, monospace;
	}

	.preset-description {
		font-size: 0.875rem;
		color: #8b949e;
		margin-bottom: 1rem;
		line-height: 1.4;
	}

	.preset-config {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1rem;
	}

	.config-item {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
	}

	.config-label {
		color: #8b949e;
		font-weight: 500;
	}

	.config-value {
		color: #f0f6fc;
		font-family: 'SF Mono', Consolas, monospace;
		font-weight: 600;
	}

	.preset-actions {
		display: flex;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		background: rgba(48, 54, 61, 0.3);
		border-top: 1px solid #30363d;
	}

	.action-btn {
		padding: 0.75rem 1rem;
		border: 1px solid #30363d;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: center;
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 0.25rem;
	}

	.action-btn.small {
		padding: 0.5rem 0.75rem;
		font-size: 0.75rem;
		flex: 1;
	}

	.action-btn.primary {
		background: #2ea043;
		border-color: #2ea043;
		color: white;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: #2c974b;
	}

	.action-btn.secondary {
		background: #21262d;
		color: #f0f6fc;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: #30363d;
	}

	.action-btn.danger {
		background: #da3633;
		border-color: #da3633;
		color: white;
	}

	.action-btn.danger:hover:not(:disabled) {
		background: #c93026;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.import-export-section {
		margin-bottom: 1rem;
	}

	.import-export-section h3 {
		margin: 0 0 1rem 0;
		font-size: 1rem;
		color: #f0f6fc;
	}

	.import-export-actions {
		display: flex;
		gap: 0.75rem;
	}

	.import-btn {
		position: relative;
		cursor: pointer;
	}

	.presets-stats {
		padding: 1rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 6px;
	}

	.stats-grid {
		display: flex;
		gap: 1rem;
		font-size: 0.75rem;
	}

	.stat-item {
		display: flex;
		gap: 0.5rem;
		flex: 1;
	}

	.stat-label {
		color: #8b949e;
		font-weight: 500;
	}

	.stat-value {
		color: #f0f6fc;
		font-family: 'SF Mono', Consolas, monospace;
		font-weight: 600;
	}

	@media (max-width: 768px) {
		.presets-grid {
			grid-template-columns: 1fr;
		}

		.create-preset-header {
			flex-direction: column;
			gap: 1rem;
			align-items: stretch;
		}

		.form-actions,
		.import-export-actions {
			flex-direction: column;
		}

		.preset-actions {
			flex-wrap: wrap;
		}

		.action-btn.small {
			flex: 1;
			min-width: 80px;
		}

		.stats-grid {
			flex-direction: column;
			gap: 0.5rem;
		}

		.current-preset-header {
			flex-direction: column;
			gap: 0.75rem;
			text-align: center;
		}

		.preset-header {
			flex-direction: column;
			gap: 0.5rem;
			align-items: flex-start;
		}
	}
</style>
