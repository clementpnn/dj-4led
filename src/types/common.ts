export interface ActionResult<T = any> {
	success: boolean;
	message: string;
	data?: T;
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
