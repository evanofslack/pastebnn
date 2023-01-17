<script lang="ts">
    import { goto } from '$app/navigation';
    import type {CreatePaste} from "../../interfaces"
    import { env } from '$env/dynamic/public'

    const expire_times = [
        {value: 600,label: '10 min'},
        {value: 3600,label: '1 hour'},
        {value: 24 * 3600,label: '1 day'},
        {value: 24 * 3600,label: '1 week'},
        {value: 24 * 3600 * 30,label: '1 month'},
        {value: -1, label: 'never'},
    ];
    const default_expire = expire_times[0]

    let text = ""
    let key = ""
    let expires = default_expire
    let burn = false
    let placeholder = randomKey()

    async function handleSubmit() {
        let baseURL = `http://${env.SERVER_HOST}:${env.SERVER_PORT}/api/paste`
        console.log(baseURL)

        let paste: CreatePaste = {text: text, key: key ? key : placeholder, seconds_until_expire: expires.value, burn_on_read: burn};

        let requestOptions = {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(paste)
        };

        let res = await fetch(baseURL, requestOptions);

        // redirect to paste url
        goto(`/${paste.key}`, { replaceState: false })
	}

    function randomKey(): string {
        let adjectives = ["young", "old", "small", "large", "cute", "ugly", "sly", "overt", "loud", "quiet", "crazy", "tame", "smart", "dumb", "lazy", "weak", "strong", "fast", "slow", "fat", "thin", "hot", "cold", "wild", "tame", "wet", "dry", "rich", "poor", "sad", "happy", "rowdy"]
        let nouns = ["lion", "trout", "ant", "hawk", "moose", "swan", "goat", "slug", "mole", "toad", "wolf", "crab","ox","seal", "fox", "moth", "worm", "yak", "loon", "deer", "hen"]
        const randAdj = Math.floor(Math.random() * adjectives.length);
        const randNoun = Math.floor(Math.random() * nouns.length);
        return adjectives[randAdj] + "-" + nouns[randNoun]
    }
</script>

<div class="flex flex-row mb-4 justify-between">
    <!-- key input -->
    <div class="flex flex-row">
        <div class="border-y border-l bg-zinc-800 text-neutral-400 py-2 pl-2 border-zinc-700">{env.APP_NAME + "/"}</div>
        <input class="mr-4 border-y border-r py-2 pr-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:outline-none" bind:value={key} placeholder={placeholder}/>
    </div>

    <div class="flex flex-row">

        <!-- expiration dropdown-->
        <div class="flex items-center mr-4">
            <p class="text-neutral-300/60 mr-1">expire</p>
            <select class="p-2" bind:value={expires} placeholder={default_expire.label}>
                {#each expire_times as time}
                    <option value={time}>
                        {time.label}
                    </option>
                {/each}
            </select>
        </div>

        <!-- burn button -->
        <label class="pr-6 flex items-center">
            <p class="text-neutral-300/60 pr-1">burn</p>
            <input type=checkbox bind:checked={burn}>
        </label>

        <!-- submit button -->
        <div>
            <button class="p-2 bg-sky-900/60 border border-sky-900/90 text-neutral-300 hover:text-neutral-300 hover:bg-sky-500/30 hover:border-sky-300/40" on:click={handleSubmit} >submit</button>
        </div>
    </div>
</div>

<!-- text area -->
<textarea placeholder="enter you paste..." class="w-full h-2/3 border p-2 bg-zinc-800 border-zinc-700 text-neutral-300 placeholder-neutral-300/20 focus:border-zinc-600 focus:outline-none" bind:value={text}></textarea>
