import type { LoadEvent } from '@sveltejs/kit';
import type { Paste } from '../../../interfaces';
import { env } from '$env/dynamic/public';

export async function load({ fetch, params }: LoadEvent) {
	const pasteID = params.slug;
	const endpoint = `http://${env.SERVER_HOST}:${env.SERVER_PORT}/api/paste/${pasteID}`;
	const res = await fetch(endpoint);
	const paste: Paste = await res.json();
	return { paste: paste };
}
