<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { saveDeleteToken, type CreatePaste, type CreatedPaste } from '../../interfaces';

	let app_name = $state('localhost');

	onMount(async () => {
		// Fetch app name from backend on mount
		try {
			const res = await fetch('/api/config');
			const config = await res.json();
			app_name = config.app_url;
		} catch {
			// keep default name
		}
	});

	const expire_times = [
		{ value: 600, label: '10 min' },
		{ value: 3600, label: '1 hour' },
		{ value: 24 * 3600, label: '1 day' },
		{ value: 24 * 3600 * 7, label: '1 week' },
		{ value: 24 * 3600 * 30, label: '1 month' },
		{ value: null, label: 'never' }
	];
	const default_expire = expire_times[0];

	let text = $state('');
	let key = $state('');
	let expires = $state(default_expire);
	let burn = $state(false);
	let error = $state('');

	const errorMessages: Record<number, string> = {
		400: 'invalid paste: keys may only use letters, numbers, - and _',
		409: 'that key is already taken',
		413: 'paste is too large',
		429: 'too many requests, try again in a minute',
		507: 'server storage is full'
	};

	async function handleSubmit() {
		error = '';
		let paste: CreatePaste = {
			text: text,
			key: key ? key : null,
			seconds_until_expire: expires.value,
			burn_on_read: burn
		};
		let requestOptions = {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(paste)
		};

		let res: Response;
		try {
			res = await fetch('/api/paste', requestOptions);
		} catch {
			error = 'could not reach server';
			return;
		}
		if (!res.ok) {
			error = errorMessages[res.status] ?? 'something went wrong';
			return;
		}

		const created: CreatedPaste = await res.json();
		saveDeleteToken(created.key, created.delete_token);

		// redirect to paste url
		goto(resolve('/[slug]', { slug: created.key }));
	}
</script>

<div class="flex flex-row mb-4 justify-between">
	<!-- key input -->
	<div class="flex flex-row">
		<div class="border-y border-l bg-zinc-800 text-neutral-400 py-2 pl-2 border-zinc-700">
			{app_name + '/'}
		</div>
		<input
			class="mr-4 border-y border-r py-2 pr-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:outline-none"
			bind:value={key}
			placeholder="random"
			maxlength="64"
		/>
	</div>

	<div class="flex flex-row">
		<!-- burn button -->
		<label class="mr-4 flex items-center">
			<p class="text-neutral-300/60 pr-1">burn on read</p>
			<input class="w-5 h-5" type="checkbox" bind:checked={burn} />
		</label>

		<!-- expiration dropdown-->
		<div class="flex items-center mr-4">
			<p class="text-neutral-300/60 mr-1">expire</p>
			<select class="p-2" bind:value={expires} placeholder={default_expire.label}>
				{#each expire_times as time (time.label)}
					<option value={time}>
						{time.label}
					</option>
				{/each}
			</select>
		</div>

		<!-- submit button -->
		<div>
			<button
				class="p-2 ml-2 bg-sky-900/60 border border-sky-900/90 text-neutral-300 hover:text-neutral-300 hover:bg-sky-500/30 hover:border-sky-300/40"
				onclick={handleSubmit}>submit</button
			>
		</div>
	</div>
</div>

{#if error}
	<p class="mb-4 text-red-400/80">{error}</p>
{/if}

<!-- text area -->
<textarea
	placeholder="enter you paste..."
	class="w-full h-2/3 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:border-zinc-600 focus:outline-none"
	bind:value={text}></textarea>
