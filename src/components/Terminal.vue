<template>
	<div class="terminal">
		<div class="terminal-header">
			<div class="header-left">
				<div class="terminal-controls">
					<div class="terminal-dot red"></div>
					<div class="terminal-dot yellow"></div>
					<div class="terminal-dot green"></div>
				</div>
			</div>
			<div class="terminal-actions">
				<button class="terminal-btn export" title="Export logs" @click="handleExportLogs">
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
						<polyline points="7,10 12,15 17,10" />
						<line x1="12" y1="15" x2="12" y2="3" />
					</svg>
				</button>
				<button class="terminal-btn clear" title="Clear all logs" @click="handleClearLogs">
					<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path
							d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14z"
						/>
					</svg>
				</button>
			</div>
		</div>

		<div ref="logs.logContainer" class="terminal-body">
			<div v-if="logs.logs.length === 0" class="no-logs">
				<div class="no-logs-icon">📝</div>
				<div class="no-logs-text">No logs yet</div>
				<div class="no-logs-hint">System events will appear here</div>
			</div>

			<div v-for="(log, index) in logs.logs" :key="log.id || index" :class="['log-entry', log.type]">
				<div class="log-indicator"></div>
				<div class="log-content">
					<div class="log-header">
						<span class="log-time">{{ log.time }}</span>
						<span v-if="log.category" class="log-category">{{ log.category }}</span>
						<span class="log-type">{{ log.type.toUpperCase() }}</span>
					</div>
					<div class="log-message">{{ log.message }}</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { useLogs } from '@/composables/useLogs';

	// Composable
	const logs = useLogs();

	// Handlers
	const handleExportLogs = async (): Promise<void> => {
		try {
			await logs.exportLogs();
		} catch (error) {
			console.error('Failed to export logs:', error);
		}
	};

	const handleClearLogs = (): void => {
		logs.clearLogs();
	};
</script>

<style scoped>
	.terminal {
		background: #0d1117;
		border: 1px solid #30363d;
		border-radius: 8px;
		overflow: hidden;
		font-family: 'SF Mono', Consolas, 'Monaco', monospace;
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 300px;
		color: #c9d1d9;
	}

	/* Header */
	.terminal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		background: #161b22;
		border-bottom: 1px solid #21262d;
		flex-shrink: 0;
		gap: 1rem;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 0.75rem;
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
		color: #c9d1d9;
		font-weight: 600;
		font-size: 0.875rem;
		letter-spacing: -0.025em;
	}

	.header-stats {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.stat-badge {
		padding: 0.25rem 0.5rem;
		border-radius: 12px;
		font-size: 0.625rem;
		font-weight: 600;
		line-height: 1;
		font-family: monospace;
	}

	.stat-badge.total {
		background: #21262d;
		color: #c9d1d9;
		border: 1px solid #30363d;
	}

	.stat-badge.error {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
		border: 1px solid rgba(248, 81, 73, 0.3);
	}

	.stat-badge.warning {
		background: rgba(210, 153, 34, 0.1);
		color: #d29922;
		border: 1px solid rgba(210, 153, 34, 0.3);
	}

	.terminal-actions {
		display: flex;
		gap: 0.5rem;
	}

	.terminal-btn {
		background: #21262d;
		border: 1px solid #30363d;
		color: #7d8590;
		cursor: pointer;
		padding: 0.5rem;
		border-radius: 4px;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
	}

	.terminal-btn:hover {
		background: #30363d;
		color: #c9d1d9;
		border-color: #c9d1d9;
	}

	.terminal-btn.active {
		background: #c9d1d9;
		color: #0d1117;
		border-color: #c9d1d9;
	}

	.terminal-btn.clear:hover {
		background: rgba(248, 81, 73, 0.1);
		border-color: #f85149;
		color: #f85149;
	}

	/* Body */
	.terminal-body {
		padding: 1rem;
		flex: 1;
		overflow-y: auto;
		font-size: 0.75rem;
		line-height: 1.4;
		background: #0d1117;
	}

	.no-logs {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem 2rem;
		text-align: center;
		opacity: 0.7;
	}

	.no-logs-icon {
		font-size: 2rem;
		margin-bottom: 1rem;
	}

	.no-logs-text {
		color: #c9d1d9;
		font-size: 0.875rem;
		font-weight: 500;
		margin-bottom: 0.5rem;
	}

	.no-logs-hint {
		color: #7d8590;
		font-size: 0.75rem;
	}

	/* Log Entries */
	.log-entry {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 0.75rem;
		padding: 0.75rem;
		background: #161b22;
		border: 1px solid #21262d;
		border-radius: 6px;
		border-left: 3px solid #30363d;
		transition: all 0.2s ease;
	}

	.log-entry:hover {
		background: #1c2128;
		border-color: #30363d;
	}

	.log-entry.success {
		border-left-color: #2ea043;
	}

	.log-entry.error {
		border-left-color: #f85149;
	}

	.log-entry.warning {
		border-left-color: #d29922;
	}

	.log-entry.info {
		border-left-color: #79c0ff;
	}

	.log-entry.debug {
		border-left-color: #7d8590;
		opacity: 0.8;
	}

	.log-indicator {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #30363d;
		margin-top: 0.5rem;
		flex-shrink: 0;
	}

	.log-entry.success .log-indicator {
		background: #2ea043;
	}

	.log-entry.error .log-indicator {
		background: #f85149;
		animation: pulse 2s infinite;
	}

	.log-entry.warning .log-indicator {
		background: #d29922;
	}

	.log-entry.info .log-indicator {
		background: #79c0ff;
	}

	.log-entry.debug .log-indicator {
		background: #7d8590;
	}

	.log-content {
		flex: 1;
		min-width: 0;
	}

	.log-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.25rem;
		flex-wrap: wrap;
	}

	.log-time {
		color: #7d8590;
		font-size: 0.6875rem;
		font-family: monospace;
		flex-shrink: 0;
	}

	.log-category {
		padding: 0.125rem 0.375rem;
		background: #21262d;
		color: #c9d1d9;
		border-radius: 8px;
		font-size: 0.625rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.025em;
		flex-shrink: 0;
	}

	.log-type {
		padding: 0.125rem 0.375rem;
		border-radius: 8px;
		font-size: 0.625rem;
		font-weight: 600;
		letter-spacing: 0.025em;
		flex-shrink: 0;
	}

	.log-entry.success .log-type {
		background: rgba(46, 160, 67, 0.1);
		color: #2ea043;
	}

	.log-entry.error .log-type {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
	}

	.log-entry.warning .log-type {
		background: rgba(210, 153, 34, 0.1);
		color: #d29922;
	}

	.log-entry.info .log-type {
		background: rgba(121, 192, 255, 0.1);
		color: #79c0ff;
	}

	.log-entry.debug .log-type {
		background: rgba(125, 133, 144, 0.1);
		color: #7d8590;
	}

	.log-message {
		color: #c9d1d9;
		word-wrap: break-word;
		line-height: 1.4;
	}

	.log-entry.debug .log-message {
		font-style: italic;
		opacity: 0.8;
	}

	/* Terminal Prompt */
	.terminal-prompt {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 1rem;
		padding: 0.5rem 0;
		border-top: 1px solid #21262d;
	}

	.prompt-symbol {
		color: #c9d1d9;
		font-weight: 600;
	}

	.prompt-cursor {
		color: #c9d1d9;
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

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	/* Scrollbar */
	.terminal-body::-webkit-scrollbar {
		width: 8px;
	}

	.terminal-body::-webkit-scrollbar-track {
		background: #161b22;
	}

	.terminal-body::-webkit-scrollbar-thumb {
		background: #21262d;
		border-radius: 4px;
	}

	.terminal-body::-webkit-scrollbar-thumb:hover {
		background: #30363d;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.terminal-header {
			padding: 0.75rem;
			flex-wrap: wrap;
			gap: 0.75rem;
		}

		.header-stats {
			order: -1;
			width: 100%;
			justify-content: center;
		}

		.terminal-body {
			padding: 0.75rem;
		}

		.log-entry {
			padding: 0.5rem;
			margin-bottom: 0.5rem;
		}

		.log-header {
			gap: 0.5rem;
		}

		.terminal-btn {
			width: 24px;
			height: 24px;
		}

		.terminal-btn svg {
			width: 10px;
			height: 10px;
		}
	}

	@media (max-width: 480px) {
		.terminal-title {
			display: none;
		}

		.header-left {
			gap: 0.5rem;
		}

		.log-category {
			display: none;
		}

		.log-entry {
			gap: 0.5rem;
		}
	}

	/* Reduced motion */
	@media (prefers-reduced-motion: reduce) {
		.prompt-cursor,
		.log-indicator {
			animation: none;
		}

		.terminal-btn,
		.log-entry {
			transition: none;
		}
	}
</style>
