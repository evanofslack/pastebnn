import type { Page } from '@sveltejs/kit';
import type { Paste } from '../../../interfaces';
import { BASE_URL } from '../../constants';

/** @type {import('./$types').PageLoad} */
export async function load({ params }: Page) {
	const pasteID = params.slug;
	const endpoint = BASE_URL + 'api/paste/' + pasteID;
	const res = await fetch(endpoint);
	const paste: Paste = await res.json();
	return { paste: paste };
}
