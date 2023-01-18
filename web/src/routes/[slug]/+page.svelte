<script lang='ts'>
    import type { Paste } from '../../../interfaces';
    import { env } from '$env/dynamic/public';
    import { onMount } from 'svelte';

    /** @type {import('./$types').PageData} */
    export let data;

    const app_port = env.PASTEBNN_API_PORT || 8080;
    const pasteID: string = data.pasteID;
    const endpoint = `http://localhost:${app_port}/api/paste/${pasteID}`;
    let key = "";
    let text = "";
    let expires_in = 0;

    onMount(async () => {
        const res = await fetch(endpoint)
        let paste = await res.json()

        key = paste.key
        text = paste.text

        expires_in = Math.floor((paste.expires - Date.now()) / 1000)
        if (expires_in < 0) {
            expires_in = 0
        }
    });


</script>

<h1 class="flex justify-between w-full">
    <h1 class="font-mono text-neutral-300/70 pl-2 pb-3">{key}</h1>
    <p class="text-neutral-300/40">expires in ~{expires_in} sec</p>
</h1>
<textarea class="w-full h-2/3 h-64 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/30 focus:border-zinc-600 focus:outline-none" bind:value={text}></textarea>
