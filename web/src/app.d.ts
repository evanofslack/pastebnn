// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
// and what to do when importing types
declare namespace App {
	// interface Locals {}
	// interface PageData {}
	// interface Error {}
	// interface Platform {}
}

declare module '$env/static/public' {
	export const APP_HOST: string;
	export const APP_PORT: string;
	export const APP_REMOTE_URL: string;
}
