<script lang="ts">
  import {getRoot} from './requests/pastes.js'

  let message = "";
  const handleClick =  async () => {
    message = await getRoot()
  }
</script>

<main>
  <h1>Raste</h1>

  <button on:click={handleClick}> Click to get message </button>

  {#if message}
    <div>
      Message: {message}
    </div>
  {/if}

</main>

<style>
  h1 {
    font-size: 3rem;
    }
</style>