export interface CreatePaste {
	key: string;
	text: string;
	burn_on_read: boolean;
	seconds_until_expire: number;
}

export interface Paste {
	id: string;
	created: number;
	expires: number;
	burn_on_read: boolean;
	key: string;
	text: string;
}
