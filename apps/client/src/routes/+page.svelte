<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let name = $state('');
  let greetMsg = $state('');

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke('greet', { name });
  }
</script>

<main class="container">
  <h1 class="text-2xl underline">Welcome to Tauri + Svelte</h1>

  <form class="row" onsubmit={greet}>
    <input placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>
</main>
