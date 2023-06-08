<script lang="ts">
  // import { invoke } from '@tauri-apps/api/tauri';
  import { open, save } from '@tauri-apps/api/dialog';
  import StartTable from '$lib/Start-table.svelte';
  import State from '$lib/State';

  let state = new State();

  async function load_config_file() {
    let path = <string | null>await open({
      multiple: false,
      filters: [
        {
          name: 'sma config',
          extensions: ['json']
        }
      ]
    });
    if (path === null) {
      return;
    }
    state.config_path = path;
  }
  async function save_config_file() {
    let path = <string | null>await save({
      filters: [
        {
          name: 'sma config',
          extensions: ['json']
        }
      ]
    });
    if (path === null) {
      return;
    }
    state.config_path = path;
  }

  function generate_shortcut_with_config() {}

  function disable_right_click() {
    document.addEventListener(
      'contextmenu',
      (e) => {
        e.preventDefault();
        return false;
      },
      { capture: true }
    );
  }

  disable_right_click();
</script>

<main>
  <header>
    <h1>SMA</h1>
  </header>

  <body>
    <StartTable {state} style="margin: auto; width: 50%;" />
    <div id="plus-button">
      <button
        on:click={() => {
          state.starts.push('');
          state.starts = state.starts;
        }}>+</button
      >
    </div>
    <div class="sub-title" id="config-options-title">Options</div>

    Cascade kill:<input type="checkbox" />

    <div id="load-save-buttons">
      <button id="config-file-load-button" on:click={load_config_file}>Load config</button>
      <button id="config-file-save-button" on:click={save_config_file}>Save config</button>
    </div>
    <button id="generate-shortcut-button" on:click={generate_shortcut_with_config}>
      Generate shortcut
    </button>
  </body>

  <footer>Created by GabbeHags</footer>
</main>

<style>
  .sub-title {
    font-size: 20px;
  }

  #load-save-buttons {
    padding-top: 50px;
    padding-bottom: 1%;
  }

  main {
    text-align: center;
  }

  footer {
    position: fixed;
    bottom: 0;
    width: 100%;
    height: 20px;
  }
</style>
