
<script lang="ts">
    import { goto } from '$app/navigation';
    import type {Paste, CreatePaste} from "../../interfaces"
    import {DISPLAY_URL} from "../constants"
    import Select from 'svelte-select';


    const expire_times = [
        {value: 10, label: '10 seconds'},
        {value: 60,label: '1 minute'},
        {value: 600,label: '10 minutes'},
        {value: 3600,label: '1 hour'},
        {value: 24 * 3600,label: '1 day'},
        {value: -1, label: 'Never'},
    ];
    const default_expire = expire_times[1]

    let text = ""
    let key = ""
    let expires = default_expire
    let burn = false
    let placeholder = "big-fat-lobster"

    function handleSelect(event:any) {
		expires = event.detail;
	}
	
	function handleClear() {
		expires = default_expire;
	}

    async function handleSubmit() {
        let baseURL = "http://localhost:8080/api/paste"

        let paste: CreatePaste = {text: text, key: key ? key : placeholder, seconds_until_expire: expires.value, burn_on_read: burn};

        let requestOptions = {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(paste)
        };

        let res = await fetch(baseURL, requestOptions);
        let resp = await res.text();

        // redirect to paste url
        goto(`/${paste.key}`, { replaceState: false }) 

	}
</script>

<div class="flex flex-row mb-4">
    <div class="border-y border-l bg-zinc-800 text-neutral-400 py-2 pl-2 border-zinc-700">{DISPLAY_URL + "/"}</div>
    <input class="mr-4 border-y border-r py-2 pr-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:outline-none" bind:value={key} placeholder={placeholder}/>
    <form class="w-48 flex">
        <label for="expire">Expires:</label>
        <Select id="expire" items={expire_times} on:select={handleSelect} on:clear={handleClear}></Select>
    </form>
    <div>
        <button class="p-2 bg-sky-900/60 border border-sky-900/90 text-neutral-300 hover:text-neutral-300 hover:bg-sky-500/30 hover:border-sky-300/40" on:click={handleSubmit} >submit</button>
    </div>
</div>
<textarea placeholder="enter you paste..." class="w-full h-64 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:border-zinc-600 focus:outline-none" bind:value={text}></textarea>