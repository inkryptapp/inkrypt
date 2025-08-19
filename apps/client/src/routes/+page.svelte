<script lang="ts">
  import { Button, Input } from '@inkrypt/ui';
  import { invoke } from '@tauri-apps/api/core';

  let name = $state('');
  let greetMsg = $state('');

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke('greet', { name });
  }
</script>

<main class="container m-auto py-5">
  <h1 class="text-2xl underline">Welcome to Tauri + Svelte</h1>

  <form class="flex gap-2" onsubmit={greet}>
    <Input class="flex-1" placeholder="Enter a name..." bind:value={name} />
    <Button type="submit">Greet</Button>
  </form>
  <p>{greetMsg}</p>
</main>
