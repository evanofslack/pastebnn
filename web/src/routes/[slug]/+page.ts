import type { LoadEvent } from '@sveltejs/kit';
import type { Paste } from '../../../interfaces';
import { APP_HOST, APP_PORT } from '$env/static/public';

export async function load({ fetch, params }: LoadEvent) {
	const pasteID = params.slug;
	const endpoint = `http://${APP_HOST}:${APP_PORT}/api/paste/${pasteID}`;
	const res = await fetch(endpoint);
	const paste: Paste = await res.json();
	return { paste: paste };
}
