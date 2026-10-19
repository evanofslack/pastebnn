export interface CreatePaste {
	key: string | null;
	text: string;
	burn_on_read: boolean;
	seconds_until_expire: number | null;
}

export interface Paste {
	id: string;
	created: number;
	expires: number | null;
	burn_on_read: boolean;
	key: string;
	text: string;
}

export interface CreatedPaste extends Paste {
	delete_token: string;
}

const tokenStorageKey = (key: string) => `pastebnn:token:${key}`;

export function saveDeleteToken(key: string, token: string) {
	try {
		localStorage.setItem(tokenStorageKey(key), token);
	} catch (e) {}
}

export function loadDeleteToken(key: string): string | null {
	try {
		return localStorage.getItem(tokenStorageKey(key));
	} catch (e) {
		return null;
	}
}

export function clearDeleteToken(key: string) {
	try {
		localStorage.removeItem(tokenStorageKey(key));
	} catch (e) {}
}
