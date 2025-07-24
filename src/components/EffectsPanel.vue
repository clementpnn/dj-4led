<template>
	<div class="effects-panel">
		<div class="panel-header">
			<h2>Effects</h2>
		</div>

		<!-- Current Effect -->
		<div v-if="currentEffect" class="current-effect">
			<div class="effect-info">
				<span class="effect-label">Current</span>
				<span class="effect-name">{{ currentEffectName }}</span>
			</div>
			<div v-if="isTransitioning" class="transition-status">
				Transitioning {{ Math.round(transitionProgress * 100) }}%
			</div>
		</div>

		<!-- Effects Grid -->
		<div class="effects-section">
			<div v-for="(categoryEffects, category) in effectsByCategory" :key="category" class="category-section">
				<h3 class="category-title">{{ formatCategoryName(category) }}</h3>
				<div class="effects-grid">
					<button
						v-for="effect in categoryEffects"
						:key="effect.id"
						class="effect-btn"
						:class="{ active: currentEffect?.id === effect.id }"
						:disabled="loading || isTransitioning"
						@click="handleEffectSelect(effect.id)"
						@mouseenter="handleEffectHover(effect.id)"
					>
						{{ effect.name }}
					</button>
				</div>
			</div>
		</div>

		<!-- Effect Details -->
		<div v-if="effectInfo" class="effect-details">
			<div class="detail-header">
				<span class="detail-title">Details</span>
			</div>
			<div class="detail-content">
				<div class="detail-item">
					<span class="detail-label">Name</span>
					<span class="detail-value">{{ effectInfo.name }}</span>
				</div>
				<div class="detail-item">
					<span class="detail-label">Description</span>
					<span class="detail-value">{{ effectInfo.description }}</span>
				</div>
			</div>
		</div>

		<!-- Actions -->
		<div class="effects-actions">
			<button class="action-btn secondary" :disabled="loading" @click="handleRefreshEffects">
				🔄 Refresh Effects
			</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { Effect, EffectInfo, EffectState } from '../types';

	interface Props {
		availableEffects: Effect[];
		currentEffect: EffectState | null;
		effectInfo: EffectInfo | null;
		loading: boolean;
		currentEffectName: string;
		isTransitioning: boolean;
		transitionProgress: number;
		effectsByCategory: Record<string, Effect[]>;
	}

	interface Emits {
		(e: 'effect-change', effectId: number): void;
		(e: 'refresh-effects'): void;
		(e: 'get-effect-info', effectId: number): void;
	}

	defineProps<Props>();
	const emit = defineEmits<Emits>();

	const handleEffectSelect = (effectId: number): void => {
		emit('effect-change', effectId);
	};

	const handleEffectHover = (effectId: number): void => {
		emit('get-effect-info', effectId);
	};

	const handleRefreshEffects = (): void => {
		emit('refresh-effects');
	};

	const formatCategoryName = (category: string): string => {
		return category.charAt(0).toUpperCase() + category.slice(1).replace(/([A-Z])/g, ' $1');
	};
</script>

<style scoped>
	.effects-panel {
		background: #161b22;
		border: 1px solid #30363d;
		border-radius: 12px;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
		color: #c9d1d9;
	}

	.effects-panel:hover {
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

	.current-effect {
		margin-bottom: 1.5rem;
		padding: 1rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
	}

	.effect-info {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.effect-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: #7d8590;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.effect-name {
		font-size: 1.125rem;
		font-weight: 600;
		color: #f0f6fc;
	}

	.transition-status {
		margin-top: 0.75rem;
		color: #d29922;
		font-weight: 600;
		font-size: 0.875rem;
	}

	.effects-section {
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
		text-transform: capitalize;
	}

	.effects-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
		gap: 0.75rem;
	}

	.effect-btn {
		padding: 1rem;
		border: 1px solid #30363d;
		border-radius: 8px;
		background: #0d1117;
		color: #f0f6fc;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: center;
		font-size: 0.875rem;
		font-weight: 600;
	}

	.effect-btn:hover:not(:disabled) {
		border-color: #484f58;
		background: #21262d;
		transform: translateY(-2px);
	}

	.effect-btn.active {
		background: #0d4929;
		border-color: #2ea043;
		box-shadow: 0 0 15px rgba(46, 160, 67, 0.3);
	}

	.effect-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none;
	}

	.effect-details {
		padding: 1rem;
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		margin-bottom: 1.5rem;
	}

	.detail-header {
		margin-bottom: 1rem;
	}

	.detail-title {
		font-size: 1rem;
		font-weight: 600;
		color: #f0f6fc;
	}

	.detail-content {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.detail-item {
		display: flex;
		gap: 0.5rem;
		align-items: flex-start;
	}

	.detail-label {
		font-size: 0.875rem;
		color: #8b949e;
		font-weight: 600;
		min-width: 80px;
		flex-shrink: 0;
	}

	.detail-value {
		font-size: 0.875rem;
		color: #f0f6fc;
		flex: 1;
	}

	.effects-actions {
		display: flex;
		gap: 0.75rem;
	}

	.action-btn {
		padding: 0.75rem 1rem;
		border: 1px solid #30363d;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.action-btn.secondary {
		background: #21262d;
		color: #f0f6fc;
	}

	.action-btn.secondary:hover:not(:disabled) {
		background: #30363d;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	@media (max-width: 768px) {
		.effects-grid {
			grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
		}

		.detail-item {
			flex-direction: column;
			gap: 0.25rem;
		}

		.detail-label {
			min-width: auto;
		}

		.effects-actions {
			flex-direction: column;
		}
	}
</style>
