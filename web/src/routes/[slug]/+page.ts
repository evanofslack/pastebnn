import type { Page } from '@sveltejs/kit';
// import type { Paste } from '../../../interfaces';

const baseURL = 'http://localhost:3000/';

// async function getPaste(key: string) {
// 	fetch(baseURL + 'api/paste/' + key, { headers: { accept: 'application/json' } })
// 		.then((res) => res.json())
// 		.then((json) => {
// 			return json;
// 		});
// 	.then((json) => JSON.parse(json))
// 	.then((data) => console.log(data));
// 	.then((json) => {
// 		const paste = JSON.parse(json);
// 		return paste;
// 	});
// }

/** @type {import('./$types').PageLoad} */
export async function load({ params }: Page) {
	const res = await fetch(baseURL + 'api/paste/' + params.slug);
	const paste = await res.json();
	console.log(paste);
	return { paste: paste };
}
