
<script lang="ts">
    import type {Paste, CreatePaste} from "../../interfaces"

    let text = ""
    let key = ""

    async function handleSubmit() {
        let baseURL = "http://localhost:3000/api/paste"
        let paste: CreatePaste = {text: text, key: key};

        let requestOptions = {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(paste)
        };

        let res = await fetch(baseURL, requestOptions);
        let resp = await res.text();
        window.location.replace("/" + paste.key)

	}
</script>

<h1 class="text-5xl font-light text-neutral-300/40 mb-10">
    raste
</h1>
<div class="flex flex-row mb-4">
    <div class="border-y border-l bg-zinc-800 text-neutral-400 py-2 pl-2 border-zinc-700">localhost:3000/</div>
    <input class="mr-4 border-y border-r py-2 pr-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/30 focus:outline-none" bind:value={key} placeholder="silly-red-beaver"/>
    <div>
        <button class="p-2 bg-sky-900/60 border border-sky-900/90 text-neutral-300 hover:text-neutral-300 hover:bg-sky-500/30 hover:border-sky-300/40" on:click={handleSubmit} >submit</button>
    </div>
</div>
<textarea placeholder="enter you paste..." class="w-full h-64 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/30 focus:border-zinc-600 focus:outline-none" bind:value={text}></textarea>