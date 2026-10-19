<script lang="ts">
	import { clearDeleteToken, loadDeleteToken, type Paste } from '../../../interfaces';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import MdDelete from 'svelte-icons/md/MdDelete.svelte';
	import MdContentCopy from 'svelte-icons/md/MdContentCopy.svelte';

	// convert paste expiration to alternative time unit for display
	function formatExpires(time: number | null): string {
		if (time == null) {
			return 'never expires';
		}

		let seconds_left = Math.floor((time - Date.now()) / 1000);
		switch (true) {
			case seconds_left < 0:
				return 'expired';
			case seconds_left < 60:
				return `expires in ~${seconds_left} secs`;
			case seconds_left < 60 * 60:
				return `expires in ~${Math.floor(seconds_left / 60)} mins`;
			case seconds_left < 60 * 60 * 24:
				return `expires in ~${Math.floor(seconds_left / (60 * 60))} hours`;
			default:
				return `expires in ~${Math.floor(seconds_left / (60 * 60 * 24))} days`;
		}
	}

	/** @type {import('./$types').PageData} */
	export let data;
	// get the pasteID slug from the URL
	const pasteID: string = data.pasteID;

	// mutable data, waiting for server response
	let key = '';
	let text = '';
	let expiration = '';
	let error = '';
	// only the creator's browser holds the token needed to delete
	let deleteToken: string | null = null;

	onMount(async () => {
		const endpoint = `/api/paste/${pasteID}`;
		let res: Response;
		try {
			res = await fetch(endpoint);
		} catch (e) {
			error = 'could not reach server';
			return;
		}
		if (res.status === 404) {
			error = 'paste not found: it may have expired or been burned after reading';
			clearDeleteToken(pasteID);
			return;
		}
		if (!res.ok) {
			error =
				res.status === 429 ? 'too many requests, try again in a minute' : 'something went wrong';
			return;
		}
		let paste: Paste = await res.json();

		key = paste.key;
		text = paste.text;
		expiration = formatExpires(paste.expires);
		deleteToken = loadDeleteToken(pasteID);
		if (paste.burn_on_read) {
			clearDeleteToken(pasteID);
			deleteToken = null;
		}
	});

	async function handleCopy() {
		try {
			await navigator.clipboard.writeText(text);
		} catch (e) {}
	}

	async function handleDelete() {
		if (!deleteToken) {
			return;
		}
		const endpoint = `/api/paste/${pasteID}`;
		let requestOptions = {
			method: 'DELETE',
			headers: {
				'X-Delete-Token': deleteToken
			}
		};
		let res = await fetch(endpoint, requestOptions);
		if (!res.ok && res.status !== 404) {
			error = 'failed to delete paste';
			return;
		}
		clearDeleteToken(pasteID);
		goto(`/`, { replaceState: false });
	}
</script>

{#if error}
	<p class="pl-2 pb-3 text-red-400/80">{error}</p>
{:else}
	<h1 class="flex justify-between w-full">
		<h1 class="font-mono text-neutral-300/70 pl-2 pb-3">{key}</h1>
		<div class="flex flex-row content-end">
			<p class="mb-3 text-neutral-300/40">{expiration}</p>
			<button class="mb-3 ml-4 w-5 h-5 text-neutral-300/80" title="copy" on:click={handleCopy}>
				<MdContentCopy />
			</button>
			{#if deleteToken}
				<button
					class="mb-3 ml-2 w-6 h-6 text-neutral-300/80"
					title="delete"
					on:click={handleDelete}
				>
					<MdDelete />
				</button>
			{/if}
			<div />
		</div>
	</h1>
	<textarea
		class="w-full h-2/3 h-64 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/30 focus:border-zinc-600 focus:outline-none"
		readonly
		bind:value={text}
	/>
{/if}
