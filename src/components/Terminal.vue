<template>
	<div class="terminal">
		<div class="terminal-header">
			<div class="terminal-controls">
				<div class="terminal-dot red"></div>
				<div class="terminal-dot yellow"></div>
				<div class="terminal-dot green"></div>
			</div>
			<div class="terminal-title">Console</div>
			<div class="terminal-actions">
				<button
					class="terminal-btn"
					:class="{ active: autoScroll }"
					@click="$emit('toggle-auto-scroll')"
					title="Auto-scroll"
				>
					↓
				</button>
				<button class="terminal-btn" @click="$emit('export-logs')" title="Export logs">⇩</button>
				<button class="terminal-btn clear" @click="$emit('clear-logs')" title="Clear logs">×</button>
			</div>
		</div>

		<div ref="logContainer" class="terminal-body">
			<div v-for="(log, index) in displayedLogs" :key="log.id || index" :class="['terminal-line', log.type]">
				<span class="terminal-prompt">$</span>
				<span class="terminal-time">{{ log.time }}</span>
				<span v-if="log.category" class="terminal-category">[{{ log.category.toUpperCase() }}]</span>
				<span class="terminal-message">{{ log.message }}</span>
			</div>

			<div v-if="displayedLogs.length === 0" class="no-logs">
				<div class="no-logs-text">No logs available</div>
			</div>

			<div v-if="displayedLogs.length > 0" class="terminal-cursor">_</div>
		</div>

		<div class="terminal-footer">
			<div class="footer-stats">
				<span class="stat-item">Total: {{ logs.length }}</span>
				<span class="stat-item">Errors: {{ errorCount }}</span>
				<span class="stat-item">Warnings: {{ warningCount }}</span>
			</div>
			<div class="footer-status">
				<span class="status-item" :class="{ active: autoScroll }">
					Auto-scroll: {{ autoScroll ? 'ON' : 'OFF' }}
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	import type { LogEntry } from '../types';

	interface Props {
		logs: LogEntry[];
		autoScroll: boolean;
		maxDisplayLogs?: number;
	}

	const props = withDefaults(defineProps<Props>(), {
		maxDisplayLogs: 1000,
	});

	const logContainer = ref<HTMLElement>();

	const displayedLogs = computed(() => {
		return props.logs.slice(-props.maxDisplayLogs);
	});

	const errorCount = computed(() => props.logs.filter((log) => log.type === 'error').length);

	const warningCount = computed(() => props.logs.filter((log) => log.type === 'warning').length);

	defineExpose({
		logContainer,
	});
</script>

<style scoped>
	.terminal {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		overflow: hidden;
		font-family: 'SF Mono', Consolas, monospace;
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 300px;
		color: #c9d1d9;
	}

	.terminal-header {
		display: flex;
		align-items: center;
		padding: 0.75rem 1rem;
		background: #161b22;
		border-bottom: 1px solid #21262d;
		flex-shrink: 0;
	}

	.terminal-controls {
		display: flex;
		gap: 0.375rem;
	}

	.terminal-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
	}

	.terminal-dot.red {
		background: #f85149;
	}

	.terminal-dot.yellow {
		background: #d29922;
	}

	.terminal-dot.green {
		background: #2ea043;
	}

	.terminal-title {
		flex: 1;
		text-align: center;
		color: #c9d1d9;
		font-weight: 500;
		font-size: 0.875rem;
	}

	.terminal-actions {
		display: flex;
		gap: 0.375rem;
	}

	.terminal-btn {
		background: #21262d;
		border: 1px solid #30363d;
		color: #7d8590;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-size: 0.75rem;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 24px;
		height: 24px;
		line-height: 1;
	}

	.terminal-btn:hover {
		background: #30363d;
		color: #c9d1d9;
	}

	.terminal-btn.active {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	.terminal-btn.clear:hover {
		background: #f85149;
		border-color: #f85149;
		color: white;
	}

	.terminal-body {
		padding: 1rem;
		flex: 1;
		overflow-y: auto;
		font-size: 0.75rem;
		line-height: 1.4;
	}

	.terminal-line {
		display: flex;
		gap: 0.5rem;
		padding: 0.125rem 0;
		color: #7d8590;
		align-items: baseline;
		word-break: break-word;
	}

	.terminal-prompt {
		color: #c9d1d9;
		font-weight: 600;
		flex-shrink: 0;
	}

	.terminal-time {
		color: #6e7681;
		min-width: 60px;
		flex-shrink: 0;
		font-size: 0.7rem;
	}

	.terminal-category {
		color: #7d8590;
		font-weight: 600;
		font-size: 0.7rem;
		flex-shrink: 0;
	}

	.terminal-message {
		flex: 1;
		word-wrap: break-word;
	}

	.terminal-line.success .terminal-message {
		color: #2ea043;
	}

	.terminal-line.error .terminal-message {
		color: #f85149;
	}

	.terminal-line.warning .terminal-message {
		color: #d29922;
	}

	.terminal-line.info .terminal-message {
		color: #c9d1d9;
	}

	.terminal-line.debug .terminal-message {
		color: #7d8590;
		font-style: italic;
	}

	.no-logs {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2rem;
		color: #7d8590;
		font-style: italic;
	}

	.terminal-cursor {
		color: #c9d1d9;
		margin-top: 0.25rem;
		animation: blink 1s infinite;
	}

	@keyframes blink {
		0%,
		50% {
			opacity: 1;
		}
		51%,
		100% {
			opacity: 0;
		}
	}

	.terminal-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 1rem;
		background: #161b22;
		border-top: 1px solid #21262d;
		font-size: 0.7rem;
		flex-shrink: 0;
	}

	.footer-stats {
		display: flex;
		gap: 1rem;
	}

	.stat-item {
		color: #7d8590;
		font-family: monospace;
	}

	.footer-status {
		display: flex;
		gap: 0.5rem;
	}

	.status-item {
		color: #7d8590;
		font-size: 0.7rem;
	}

	.status-item.active {
		color: #c9d1d9;
	}

	/* Scrollbar */
	.terminal-body::-webkit-scrollbar {
		width: 6px;
	}

	.terminal-body::-webkit-scrollbar-track {
		background: #161b22;
	}

	.terminal-body::-webkit-scrollbar-thumb {
		background: #21262d;
		border-radius: 3px;
	}

	.terminal-body::-webkit-scrollbar-thumb:hover {
		background: #30363d;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.terminal-header {
			padding: 0.5rem;
		}

		.terminal-body {
			padding: 0.75rem;
			font-size: 0.7rem;
		}

		.terminal-footer {
			padding: 0.5rem;
			flex-direction: column;
			gap: 0.25rem;
			align-items: stretch;
		}

		.footer-stats {
			justify-content: center;
			gap: 0.75rem;
		}

		.terminal-time {
			min-width: 50px;
		}

		.terminal-actions {
			gap: 0.25rem;
		}

		.terminal-btn {
			min-width: 20px;
			height: 20px;
			font-size: 0.7rem;
		}
	}

	@media (max-width: 480px) {
		.terminal {
			min-height: 250px;
		}

		.terminal-title {
			display: none;
		}

		.terminal-line {
			gap: 0.375rem;
		}

		.terminal-category {
			display: none;
		}
	}

	/* Reduced motion */
	@media (prefers-reduced-motion: reduce) {
		.terminal-cursor {
			animation: none;
		}

		.terminal-btn {
			transition: none;
		}
	}
</style>
