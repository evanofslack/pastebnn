
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
        alert(resp)

	}
</script>

<div>
    <h1 class="text-3xl font-bold underline">
        raste
    </h1>
</div>
<textarea class="w-full h-96 m-6 border" value={text}></textarea>
<input value={key}/>
<div>
    <button on:click={handleSubmit} >submit</button>
</div>