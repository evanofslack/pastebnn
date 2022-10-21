
<script lang='ts'>
    import type { Paste } from '../../../interfaces';

    /** @type {import('./$types').PageData} */
    export let data;

    let paste: Paste = data.paste
    let text = paste.text

    let expires_in = Math.floor((paste.expires - Date.now()) / 1000)
    if (expires_in < 0) {
        expires_in = 0
    }

</script>

<h1 class="flex justify-between w-full">
    <h1 class="font-mono text-neutral-300/70 pl-2 pb-3">{paste.key}</h1>
    <p class="text-neutral-300/40">expires in ~{expires_in} sec</p>
</h1>
<textarea class="w-full h-64 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/30 focus:border-zinc-600 focus:outline-none" bind:value={text}></textarea>