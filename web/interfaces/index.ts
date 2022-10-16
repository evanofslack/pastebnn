export interface CreatePaste {
	key: string;
	text: string;
	seconds_until_expire: number;
}

export interface Paste {
	id: string;
	created: number;
	expires: number;
	key: string;
	text: string;
}
