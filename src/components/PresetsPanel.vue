<template>
	<div class="presets-panel">
		<!-- Header -->
		<div class="panel-header">
			<div class="header-title">
				<h2>Presets</h2>
				<div class="presets-count">{{ allPresets.length }} available</div>
			</div>

			<div v-if="currentPreset" class="current-preset-mini">
				<div class="current-name">{{ currentPreset.name }}</div>
			</div>
		</div>

		<!-- Quick Actions -->
		<div class="quick-actions">
			<button class="quick-btn create" :disabled="loading" @click="showCreateForm = !showCreateForm">
				<span class="btn-text">{{ showCreateForm ? 'Cancel' : 'Create New' }}</span>
			</button>

			<button class="quick-btn export" :disabled="loading" @click="handleExportPresets">
				<span class="btn-text">Export</span>
			</button>

			<label class="quick-btn import">
				<span class="btn-text">Import</span>
				<input ref="fileInput" type="file" accept=".json" style="display: none" @change="handleFileImport" />
			</label>
		</div>

		<!-- Create Form - Amélioration design -->
		<div v-if="showCreateForm" class="create-form">
			<div class="form-container">
				<div class="form-header">
					<div class="header-icon">
						<svg
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M12 5v14M5 12h14" />
						</svg>
					</div>
					<div class="header-content">
						<h3>Create New Preset</h3>
						<p>Save your current configuration as a new preset</p>
					</div>
				</div>

				<div class="form-body">
					<div class="input-group">
						<label class="input-label">
							<span class="label-text">Preset Name</span>
							<span class="label-required">*</span>
						</label>
						<input
							v-model="newPresetName"
							type="text"
							class="form-input"
							placeholder="Enter preset name..."
							@keyup.enter="handleCreatePreset"
						/>
						<div class="input-hint">Choose a descriptive name for your preset</div>
					</div>

					<div class="input-group">
						<label class="input-label">
							<span class="label-text">Description</span>
							<span class="label-optional">(optional)</span>
						</label>
						<textarea
							v-model="newPresetDescription"
							class="form-textarea"
							placeholder="Describe what makes this preset special..."
							rows="3"
						></textarea>
						<div class="input-hint">Add details about when to use this preset</div>
					</div>
				</div>

				<div class="form-footer">
					<button class="action-btn secondary" @click="resetCreateForm">Cancel</button>
					<button class="action-btn primary" :disabled="!newPresetName.trim()" @click="handleCreatePreset">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
							<polyline points="17,21 17,13 7,13 7,21" />
							<polyline points="7,3 7,8 15,8" />
						</svg>
						Save Preset
					</button>
				</div>
			</div>
		</div>

		<!-- Presets Grid -->
		<div class="presets-section">
			<h3 class="section-title">Available Presets</h3>

			<div class="presets-grid">
				<div v-for="preset in allPresets" :key="preset.id" class="preset-card">
					<!-- Card Header -->
					<div class="card-header">
						<div class="preset-info">
							<div class="preset-name">{{ preset.name }}</div>
							<div class="preset-description">{{ preset.description }}</div>
						</div>
					</div>

					<!-- Config Preview -->
					<div class="config-preview">
						<div class="config-item">
							<span class="config-label">Effect:</span>
							<span class="config-value">{{ preset.config.effect?.name || 'None' }}</span>
						</div>
						<div class="config-item">
							<span class="config-label">Color:</span>
							<span class="config-value">{{ preset.config.color?.mode || 'Default' }}</span>
						</div>
						<div class="config-item">
							<span class="config-label">Brightness:</span>
							<span class="config-value"
								>{{ Math.round((preset.config.led?.brightness || 1) * 100) }}%</span
							>
						</div>
					</div>

					<!-- Actions -->
					<div class="card-actions">
						<button
							class="card-btn apply"
							:disabled="currentPreset?.id === preset.id || isApplying"
							@click="handleApplyPreset(preset.id)"
						>
							{{ currentPreset?.id === preset.id ? 'Active' : 'Apply' }}
						</button>

						<button
							class="card-btn duplicate"
							@click="handleDuplicatePreset(preset.id)"
							title="Duplicate preset"
						>
							Copy
						</button>

						<button
							v-if="!isDefaultPreset(preset.id)"
							class="card-btn delete"
							@click="handleDeletePreset(preset.id)"
							title="Delete preset"
						>
							Delete
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	import { useAudio } from '../composables/useAudio';
	import { useColors } from '../composables/useColors';
	import { useEffects } from '../composables/useEffects';
	import { useLED } from '../composables/useLED';
	import { usePresets } from '../composables/usePresets';

	// Composables
	const presets = usePresets();
	const audio = useAudio();
	const colors = useColors();
	const effects = useEffects();
	const led = useLED();

	// Local state
	const showCreateForm = ref(false);
	const newPresetName = ref('');
	const newPresetDescription = ref('');
	const fileInput = ref<HTMLInputElement>();

	// Computed - Conversion en boolean pour TypeScript
	const loading = computed(() => !!presets.loading);
	const isApplying = computed(() => !!presets.isApplying);
	const allPresets = computed(() => presets.allPresets);
	const currentPreset = computed(() => presets.currentPreset);

	const isDefaultPreset = (presetId: string): boolean => {
		const defaultIds = ['party-mode', 'chill-mode', 'focus-mode', 'gaming-mode'];
		return defaultIds.includes(presetId);
	};

	// Handlers
	const handleCreatePreset = async (): Promise<void> => {
		if (!newPresetName.value.trim()) return;

		const composables = { audio, effects, colors, led };
		const result = await presets.createPreset(
			newPresetName.value.trim(),
			newPresetDescription.value.trim(),
			composables
		);

		if (result.success) {
			resetCreateForm();
		} else {
			console.error('Failed to create preset:', result.message);
		}
	};

	const handleApplyPreset = async (presetId: string): Promise<void> => {
		const composables = { audio, effects, colors, led };
		const result = await presets.applyPreset(presetId, composables);

		if (!result.success) {
			console.error('Failed to apply preset:', result.message);
		}
	};

	const handleDuplicatePreset = async (presetId: string): Promise<void> => {
		const originalPreset = presets.getPresetById(presetId);
		if (!originalPreset) return;

		const newName = `${originalPreset.name} (Copy)`;
		const result = presets.duplicatePreset(presetId, newName);

		if (!result) {
			console.error('Failed to duplicate preset');
		}
	};

	const handleDeletePreset = async (presetId: string): Promise<void> => {
		const result = presets.deletePreset(presetId);

		if (!result) {
			console.error('Failed to delete preset');
		}
	};

	const handleExportPresets = async (): Promise<void> => {
		const result = presets.exportPresets();

		if (!result.success) {
			console.error('Failed to export presets:', result.message);
		}
	};

	const handleFileImport = async (event: Event): Promise<void> => {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (file) {
			const result = await presets.importPresets(file);
			target.value = '';

			if (!result.success) {
				console.error('Failed to import presets:', result.message);
			}
		}
	};

	const resetCreateForm = (): void => {
		newPresetName.value = '';
		newPresetDescription.value = '';
		showCreateForm.value = false;
	};
</script>

<style scoped>
	.presets-panel {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 12px;
		padding: 1.5rem;
		color: #c9d1d9;
		max-height: 80vh;
		overflow-y: auto;
	}

	/* Header */
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid #21262d;
	}

	.header-title h2 {
		margin: 0 0 0.25rem 0;
		font-size: 1.25rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.presets-count {
		font-size: 0.75rem;
		color: #7d8590;
		background: #21262d;
		padding: 0.25rem 0.75rem;
		border-radius: 12px;
		font-weight: 500;
	}

	.current-preset-mini {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		background: rgba(63, 185, 80, 0.1);
		border: 1px solid rgba(63, 185, 80, 0.3);
		border-radius: 8px;
		font-size: 0.75rem;
	}

	.current-name {
		font-weight: 600;
		color: #3fb950;
	}

	/* Quick Actions */
	.quick-actions {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 0.75rem;
		margin-bottom: 1.5rem;
	}

	.quick-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 8px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		text-decoration: none;
	}

	.quick-btn:hover:not(:disabled) {
		background: #21262d;
		border-color: #30363d;
	}

	.quick-btn.create {
		border-color: #3fb950;
		color: #3fb950;
	}

	.quick-btn.create:hover {
		background: rgba(63, 185, 80, 0.1);
	}

	.quick-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-text {
		font-weight: 600;
	}

	/* Create Form - Design amélioré */
	.create-form {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 12px;
		margin-bottom: 1.5rem;
		border-left: 3px solid #3fb950;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.form-container {
		padding: 1.5rem;
	}

	.form-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1.5rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid rgba(63, 185, 80, 0.2);
	}

	.header-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		background: rgba(63, 185, 80, 0.1);
		border: 1px solid rgba(63, 185, 80, 0.3);
		border-radius: 8px;
		color: #3fb950;
		flex-shrink: 0;
	}

	.header-content h3 {
		margin: 0 0 0.25rem 0;
		font-size: 1.1rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.header-content p {
		margin: 0;
		font-size: 0.875rem;
		color: #7d8590;
		line-height: 1.4;
	}

	.form-body {
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		margin-bottom: 1.5rem;
	}

	.input-group {
		display: flex;
		flex-direction: column;
	}

	.input-label {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		margin-bottom: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.label-text {
		color: #c9d1d9;
	}

	.label-required {
		color: #f85149;
		font-size: 0.75rem;
	}

	.label-optional {
		color: #7d8590;
		font-size: 0.75rem;
		font-weight: 400;
	}

	.form-input,
	.form-textarea {
		padding: 0.875rem 1rem;
		background: #21262d;
		border: 1px solid #30363d;
		border-radius: 8px;
		color: #c9d1d9;
		font-size: 0.875rem;
		font-family: inherit;
		transition: all 0.2s ease;
		width: 100%;
		box-sizing: border-box;
	}

	.form-input:focus,
	.form-textarea:focus {
		outline: none;
		border-color: #3fb950;
		box-shadow: 0 0 0 2px rgba(63, 185, 80, 0.1);
		background: #161b22;
	}

	.form-textarea {
		resize: vertical;
		min-height: 80px;
		font-family: inherit;
		line-height: 1.4;
	}

	.input-hint {
		margin-top: 0.375rem;
		font-size: 0.75rem;
		color: #7d8590;
		line-height: 1.3;
	}

	.form-footer {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
		padding-top: 1rem;
		border-top: 1px solid #21262d;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1.25rem;
		border: 1px solid #30363d;
		border-radius: 8px;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.action-btn.primary {
		background: #21262d;
		color: #3fb950;
		border-color: #3fb950;
		min-width: 140px;
		justify-content: center;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: rgba(63, 185, 80, 0.1);
		transform: translateY(-1px);
		box-shadow: 0 2px 8px rgba(63, 185, 80, 0.2);
	}

	.action-btn.primary:disabled {
		background: #21262d;
		color: #7d8590;
		border-color: #21262d;
		cursor: not-allowed;
		transform: none;
		box-shadow: none;
	}

	.action-btn.secondary {
		background: #161b22;
		color: #7d8590;
		border-color: #21262d;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: #21262d;
		color: #c9d1d9;
		border-color: #30363d;
	}

	/* Presets Section */
	.presets-section {
		margin-bottom: 1rem;
	}

	.section-title {
		font-size: 1rem;
		font-weight: 600;
		color: #c9d1d9;
		margin-bottom: 1rem;
	}

	/* Presets Grid */
	.presets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 1rem;
	}

	.preset-card {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 12px;
		padding: 1rem;
		transition: all 0.2s ease;
		position: relative;
		overflow: hidden;
	}

	.preset-card:hover {
		border-color: #30363d;
		background: #1c2128;
	}

	.preset-card.active {
		background: rgba(63, 185, 80, 0.05);
		border-color: #3fb950;
		border-width: 2px;
	}

	/* Card Header */
	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 0.75rem;
	}

	.preset-info {
		flex: 1;
		min-width: 0;
	}

	.preset-name {
		font-size: 0.875rem;
		font-weight: 600;
		color: #c9d1d9;
		margin-bottom: 0.25rem;
		line-height: 1.2;
	}

	.preset-description {
		font-size: 0.75rem;
		color: #7d8590;
		line-height: 1.3;
		display: -webkit-box;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	/* Config Preview */
	.config-preview {
		background: #0d1117;
		border: 1px solid #21262d;
		border-radius: 6px;
		padding: 0.75rem;
		margin-bottom: 0.75rem;
		font-size: 0.75rem;
		font-family: monospace;
	}

	.config-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.25rem;
	}

	.config-item:last-child {
		margin-bottom: 0;
	}

	.config-label {
		color: #7d8590;
		font-weight: 500;
	}

	.config-value {
		color: #79c0ff;
		font-weight: 600;
	}

	/* Card Actions */
	.card-actions {
		display: flex;
		gap: 0.5rem;
	}

	.card-btn {
		flex: 1;
		padding: 0.5rem 0.75rem;
		border: 1px solid #30363d;
		border-radius: 6px;
		font-size: 0.75rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.25rem;
	}

	.card-btn.apply {
		background: #21262d;
		color: #c9d1d9;
		border-color: #c9d1d9;
	}

	.card-btn.apply:hover:not(:disabled) {
		background: #30363d;
	}

	.card-btn.apply:disabled {
		background: #238636;
		border-color: #238636;
		color: white;
		cursor: not-allowed;
	}

	.card-btn.duplicate,
	.card-btn.delete {
		background: #161b22;
		color: #7d8590;
		border-color: #21262d;
		flex: 0 0 auto;
	}

	.card-btn.duplicate:hover:not(:disabled) {
		background: #21262d;
		color: #c9d1d9;
		border-color: #30363d;
	}

	.card-btn.delete:hover:not(:disabled) {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
		border-color: #f85149;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.presets-panel {
			padding: 1rem;
		}

		.panel-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.75rem;
		}

		.quick-actions {
			grid-template-columns: 1fr;
		}

		.presets-grid {
			grid-template-columns: 1fr;
		}

		.card-header {
			flex-direction: column;
			gap: 0.5rem;
		}

		.card-actions {
			flex-wrap: wrap;
		}

		.card-btn.apply {
			flex: 1 1 100%;
		}

		.form-footer {
			flex-direction: column;
		}

		.action-btn {
			width: 100%;
		}
	}

	@media (max-width: 480px) {
		.config-preview {
			padding: 0.5rem;
		}

		.form-container {
			padding: 1rem;
		}
	}

	/* Scrollbar */
	.presets-panel::-webkit-scrollbar {
		width: 8px;
	}

	.presets-panel::-webkit-scrollbar-track {
		background: #161b22;
	}

	.presets-panel::-webkit-scrollbar-thumb {
		background: #21262d;
		border-radius: 4px;
	}

	.presets-panel::-webkit-scrollbar-thumb:hover {
		background: #30363d;
	}

	/* Loading states */
	.quick-btn:disabled,
	.action-btn:disabled,
	.card-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
