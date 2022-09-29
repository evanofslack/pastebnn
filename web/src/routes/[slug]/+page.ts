import type { Page } from '@sveltejs/kit';
import type { Paste } from '../../../interfaces';

const baseURL = 'http://localhost:3000/';

// Standard variation
function fetchPastes<T>(url: string): Promise<T> {
	return fetch(url).then((response) => {
		if (!response.ok) {
			throw new Error(response.statusText);
		}
		return response.json() as Promise<T>;
	});
}

/** @type {import('./$types').PageLoad} */
export async function load({ params }: Page) {
	const pasteID = params.slug;
	const endpoint = baseURL + 'api/paste/' + pasteID;
	const res = await fetch(endpoint);
	const paste: Paste = await res.json();
	console.log(paste);
	return { paste: paste };
}

// /** @type {import('./$types').PageLoad} */
// export async function load({ params }: Page) {
// 	const pasteID = params.slug;
// 	const endpoint = baseURL + 'api/paste/' + pasteID;
// 	fetchPastes<Paste>(endpoint)
// 		.then((paste) => {
// 			console.log(paste.text);
// 			return { paste: paste };
// 		})
// 		.catch((error) => {
// 			/* show error message */
// 			console.log(error);
// 		});
// }
