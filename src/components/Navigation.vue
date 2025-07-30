<template>
	<nav class="main-nav">
		<div class="nav-container">
			<button
				v-for="tab in tabs"
				:key="tab.id"
				@click="$emit('tab-change', tab.id)"
				:class="['nav-tab', { active: activeTab === tab.id }]"
			>
				<span class="tab-label">{{ tab.label }}</span>
			</button>
		</div>
	</nav>
</template>

<script setup lang="ts">
	interface Tab {
		id: string;
		label: string;
	}

	interface Props {
		activeTab: string;
		tabs: Tab[];
	}

	interface Emits {
		(e: 'tab-change', tabId: string): void;
	}

	defineProps<Props>();
	defineEmits<Emits>();
</script>

<style scoped>
	.main-nav {
		background: #0d1117;
		border-bottom: 1px solid #21262d;
	}

	.nav-container {
		display: flex;
		padding: 0 1.5rem;
		gap: 0.5rem;
		overflow-x: auto;
		scrollbar-width: none;
		-ms-overflow-style: none;
	}

	.nav-container::-webkit-scrollbar {
		display: none;
	}

	.nav-tab {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 1rem 1.5rem;
		border: none;
		background: transparent;
		color: #7d8590;
		cursor: pointer;
		transition: all 0.2s ease;
		border-bottom: 2px solid transparent;
		white-space: nowrap;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.nav-tab:hover {
		color: #c9d1d9;
		background: rgba(125, 133, 144, 0.1);
	}

	.nav-tab.active {
		color: #c9d1d9;
		border-bottom-color: #c9d1d9;
		background: rgba(201, 209, 217, 0.05);
	}

	.nav-tab:focus {
		outline: 2px solid #c9d1d9;
		outline-offset: -2px;
	}

	.tab-label {
		font-weight: 500;
		flex-shrink: 0;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.nav-container {
			padding: 0 1rem;
		}

		.nav-tab {
			padding: 0.75rem 1rem;
		}

		.tab-label {
			display: none;
		}
	}

	@media (max-width: 480px) {
		.nav-tab {
			padding: 0.75rem 0.75rem;
		}
	}
</style>
