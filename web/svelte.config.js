import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';

export default {
	preprocess: [
		preprocess({
			postcss: true
		})
	],
	kit: {
		adapter: adapter({
			fallback: '200.html'
		}),
		prerender: { entries: [] }
	}
};
