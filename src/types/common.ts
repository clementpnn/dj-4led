export interface ActionResult {
	success: boolean;
	message: string;
	data?: any;
	error?: string;
}

export interface LoadingState {
	[key: string]: boolean;
}

export interface ValidationError {
	field: string;
	message: string;
	code?: string;
}

export interface PaginationInfo {
	page: number;
	limit: number;
	total: number;
	totalPages: number;
}

export interface SortInfo {
	field: string;
	direction: 'asc' | 'desc';
}
