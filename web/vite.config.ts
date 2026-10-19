import { sveltekit } from '@sveltejs/kit/vite';
import adapter from '@sveltejs/adapter-static';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			// single page app: the rust server serves 200.html for unknown paths
			adapter: adapter({ fallback: '200.html' }),
			prerender: { entries: [] }
		})
	],
	server: {
		// `npm run dev` talks to a local `cargo run`
		proxy: { '/api': 'http://localhost:8080' }
	}
});
