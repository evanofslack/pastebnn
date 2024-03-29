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
	export const PASTEBNN_APP_NAME: string;
	export const PASTEBNN_API_PORT: string;
}
