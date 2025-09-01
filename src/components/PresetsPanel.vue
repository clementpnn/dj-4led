<template>
	<div class="presets-panel">
		<!-- Header -->
		<div class="panel-header">
			<div class="header-title">
				<h2>Presets</h2>
				<div class="presets-count">{{ store.allPresets.length }} available</div>
			</div>

			<div v-if="store.currentPreset" class="current-preset-mini">
				<div class="current-name">{{ store.currentPreset.name }}</div>
			</div>
		</div>

		<!-- Quick Actions -->
		<div class="quick-actions">
			<button class="quick-btn create" :disabled="store.loading" @click="showCreateForm = !showCreateForm">
				<span class="btn-text">{{ showCreateForm ? 'Cancel' : 'Create New' }}</span>
			</button>

			<button class="quick-btn export" :disabled="store.loading" @click="handleExportPresets">
				<span class="btn-text">Export</span>
			</button>

			<label class="quick-btn import">
				<span class="btn-text">Import</span>
				<input ref="fileInput" type="file" accept=".json" style="display: none" @change="handleFileImport" />
			</label>
		</div>

		<!-- Create Form -->
		<div v-if="showCreateForm" class="create-form">
			<div class="form-container">
				<div class="form-header">
					<h3>Create New Preset</h3>
					<p>Save your current configuration as a new preset</p>
				</div>

				<div class="form-content">
					<div class="input-group">
						<label class="input-label"> Preset Name <span class="required">*</span> </label>
						<input
							v-model="newPresetName"
							type="text"
							class="form-input"
							:class="{ error: nameErrorMessage }"
							placeholder="Enter preset name..."
							@keyup.enter="handleCreatePreset"
						/>
						<div v-if="nameErrorMessage" class="error-message">{{ nameErrorMessage }}</div>
					</div>

					<div class="input-group">
						<label class="input-label">Description</label>
						<textarea
							v-model="newPresetDescription"
							class="form-textarea"
							placeholder="Describe what makes this preset special..."
							rows="3"
						></textarea>
					</div>
				</div>

				<div class="form-actions">
					<button class="btn-secondary" @click="resetCreateForm">Cancel</button>
					<button class="btn-primary" :disabled="!isNameValid || store.loading" @click="handleCreatePreset">
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
				<div
					v-for="preset in store.allPresets"
					:key="preset.id"
					class="preset-card"
					:class="{ active: store.currentPreset?.id === preset.id }"
				>
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
							<span class="config-value">
								{{ Math.round((preset.config.led?.brightness || 1) * 100) }}%
							</span>
						</div>
					</div>

					<!-- Actions -->
					<div class="card-actions">
						<button
							class="card-btn apply"
							:disabled="store.currentPreset?.id === preset.id || store.isApplying"
							@click="handleApplyPreset(preset.id)"
						>
							{{ store.currentPreset?.id === preset.id ? 'Active' : 'Apply' }}
						</button>

						<button
							class="card-btn duplicate"
							title="Duplicate preset"
							@click="handleDuplicatePreset(preset.id)"
						>
							Copy
						</button>

						<button
							v-if="!store.isDefaultPreset(preset.id)"
							class="card-btn delete"
							title="Delete preset"
							@click="handleDeletePreset(preset.id)"
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

	import { useAudio } from '@/composables/useAudio';
	import { useColors } from '@/composables/useColors';
	import { useEffects } from '@/composables/useEffects';
	import { useLED } from '@/composables/useLED';
	import { usePresets } from '@/composables/usePresets';
	import { usePresetsStore } from '@/stores/presets';

	// Store - Source unique de vérité pour les données
	const store = usePresetsStore();

	// Composables - Pour la logique métier uniquement
	const presetsComposable = usePresets();
	const audio = useAudio();
	const colors = useColors();
	const effects = useEffects();
	const led = useLED();

	// Local UI state only
	const showCreateForm = ref(false);
	const newPresetName = ref('');
	const newPresetDescription = ref('');
	const fileInput = ref<HTMLInputElement>();

	// Computed for form validation - utilise directement le store
	const isNameValid = computed(() => {
		const trimmed = newPresetName.value.trim();
		if (!trimmed || trimmed.length < 2) return false;
		return !store.nameExists(trimmed);
	});

	const nameErrorMessage = computed(() => {
		const trimmed = newPresetName.value.trim();
		if (!trimmed) return '';
		if (trimmed.length < 2) return 'Name must be at least 2 characters';
		if (store.nameExists(trimmed)) return 'A preset with this name already exists';
		return '';
	});

	// Handlers - utilisent le composable pour la logique
	const handleCreatePreset = async (): Promise<void> => {
		if (!newPresetName.value.trim() || !isNameValid.value) {
			console.warn('❌ [PRESETS_PANEL] Invalid name or name already exists');
			return;
		}

		const composables = { audio, effects, colors, led };
		const result = await presetsComposable.createPreset(
			newPresetName.value.trim(),
			newPresetDescription.value.trim(),
			composables
		);

		if (result.success) {
			console.log('✅ [PRESETS_PANEL] Preset created successfully');
			resetCreateForm();
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to create preset:', result.message);
		}
	};

	const handleApplyPreset = async (presetId: string): Promise<void> => {
		console.log('🎯 [PRESETS_PANEL] Applying preset:', presetId);

		const composables = { audio, effects, colors, led };
		const result = await presetsComposable.applyPreset(presetId, composables);

		if (result.success) {
			console.log('✅ [PRESETS_PANEL] Preset applied successfully');
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to apply preset:', result.message);
		}
	};

	const handleDuplicatePreset = (presetId: string): void => {
		const originalPreset = store.getPresetById(presetId);
		if (!originalPreset) {
			console.error('❌ [PRESETS_PANEL] Original preset not found:', presetId);
			return;
		}

		let copyNumber = 1;
		let newName = `${originalPreset.name} (Copy)`;

		while (store.nameExists(newName)) {
			copyNumber++;
			newName = `${originalPreset.name} (Copy ${copyNumber})`;
		}

		const result = store.duplicatePreset(presetId, newName);

		if (result) {
			console.log('✅ [PRESETS_PANEL] Preset duplicated successfully:', result);
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to duplicate preset');
		}
	};

	const handleDeletePreset = (presetId: string): void => {
		const presetToDelete = store.getPresetById(presetId);
		if (presetToDelete) {
			console.log('🗑️ [PRESETS_PANEL] Found preset to delete:', presetToDelete.name);
		}

		const result = store.deletePreset(presetId);

		if (result) {
			console.log('✅ [PRESETS_PANEL] Preset deleted successfully');
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to delete preset - might be default preset');
		}
	};

	const handleExportPresets = async (): Promise<void> => {
		console.log('🎯 [PRESETS_PANEL] Starting export process...');

		const result = presetsComposable.exportPresets();

		if (result.success) {
			console.log('✅ [PRESETS_PANEL] Presets exported successfully');
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to export presets:', result.message);
		}
	};

	const handleFileImport = async (event: Event): Promise<void> => {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];

		if (!file) return;

		const result = await presetsComposable.importPresets(file);
		target.value = ''; // Reset file input

		if (result.success) {
			console.log('✅ [PRESETS_PANEL] Presets imported successfully:', result.message);
		} else {
			console.error('❌ [PRESETS_PANEL] Failed to import presets:', result.message);
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

	.header-title {
		display: flex;
		flex-direction: row;
		justify-content: space-between;
		align-items: center;
		width: 100%;
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

	/* Create Form */
	.create-form {
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 8px;
		margin-bottom: 1.5rem;
		overflow: hidden;
	}

	.form-container {
		padding: 1.25rem;
	}

	.form-header {
		margin-bottom: 1.25rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid #21262d;
	}

	.form-header h3 {
		margin: 0 0 0.5rem 0;
		font-size: 1rem;
		font-weight: 600;
		color: #c9d1d9;
	}

	.form-header p {
		margin: 0;
		font-size: 0.875rem;
		color: #7d8590;
		line-height: 1.4;
	}

	.form-content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}

	.input-group {
		display: flex;
		flex-direction: column;
	}

	.input-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: #c9d1d9;
		margin-bottom: 0.5rem;
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.required {
		color: #f85149;
		font-size: 0.75rem;
	}

	.form-input,
	.form-textarea {
		padding: 0.75rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 6px;
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
		background: #010409;
	}

	.form-input::placeholder,
	.form-textarea::placeholder {
		color: #7d8590;
	}

	.form-input.error,
	.form-textarea.error {
		border-color: #f85149;
		background: rgba(248, 81, 73, 0.1);
	}

	.error-message {
		margin-top: 0.5rem;
		font-size: 0.75rem;
		color: #f85149;
		font-weight: 500;
	}

	.form-textarea {
		resize: vertical;
		min-height: 80px;
		line-height: 1.4;
	}

	.form-actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
		padding-top: 1rem;
		border-top: 1px solid #21262d;
	}

	.btn-primary,
	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid;
		white-space: nowrap;
	}

	.btn-primary {
		background: #238636;
		border-color: #238636;
		color: #fff;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2ea043;
		border-color: #2ea043;
	}

	.btn-primary:disabled {
		background: #21262d;
		border-color: #30363d;
		color: #7d8590;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: #21262d;
		border-color: #30363d;
		color: #c9d1d9;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #30363d;
		border-color: #484f58;
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
		border-radius: 8px;
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
		padding: calc(1rem - 1px); /* Adjust for thicker border */
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

		.form-actions {
			flex-direction: column;
		}

		.btn-primary,
		.btn-secondary {
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
	.btn-primary:disabled,
	.btn-secondary:disabled,
	.card-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
