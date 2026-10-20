<script lang="ts">
	import '../app.css';
	import { resolve } from '$app/paths';

	let { children } = $props();
</script>

<div class="bg-zinc-900 min-h-screen min-w-screen px-10 m-0 flex justify-center">
	<div class="w-full max-w-4xl">
		<h1 class="text-4xl font-light text-neutral-300/10 pb-10 pt-5">
			<a href={resolve('/')}>pastebnn</a>
		</h1>
		{@render children()}
	</div>
</div>
