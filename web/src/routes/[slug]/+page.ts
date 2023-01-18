import type { LoadEvent } from '@sveltejs/kit';

export async function load({ fetch, params }: LoadEvent) {
	const pasteID = params.slug;
	return { pasteID: pasteID };
}
