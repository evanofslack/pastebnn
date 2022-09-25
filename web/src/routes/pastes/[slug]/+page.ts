import type { Page } from '@sveltejs/kit';

/** @type {import('./$types').PageLoad} */
export const load = ({ params }: Page) => {
	return { slug: params.slug };
};
