
<script lang="ts">
    import type {Paste, CreatePaste} from "../../interfaces"

    let text = "content"
    let key = "path"

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
        // alert(resp)

	}
</script>

<div>
    <h1 class="text-3xl font-bold underline">
        raste
    </h1>
</div>
<textarea class="w-full h-64 border p-2" bind:value={text}></textarea>
<input class="border p-1" bind:value={key}/>
<div>
    <button class="border p-1" on:click={handleSubmit} >submit</button>
</div>