export interface LogEntry {
	id?: string;
	time: string;
	timestamp: number;
	message: string;
	type: 'info' | 'success' | 'error' | 'warning' | 'debug';
	category?: 'audio' | 'effects' | 'led' | 'system' | 'user';
	details?: any;
}

export interface LogFilter {
	types: LogEntry['type'][];
	categories: LogEntry['category'][];
	timeRange?: {
		start: number;
		end: number;
	};
	searchText?: string;
}

export interface LogStats {
	total: number;
	byType: Record<LogEntry['type'], number>;
	byCategory: Record<string, number>;
	timeRange: {
		oldest: number;
		newest: number;
	};
}
