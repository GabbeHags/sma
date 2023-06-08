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
    // TODO: load the config from a file
  }
  async function save_config_file() {
    let path = <string | null>await save({
      defaultPath: state.config_path,
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
    // TODO: write the state to a file.
  }

  function generate_shortcut_with_config() {
    // TODO: generate a shortcut which spawns sma.exe with this config
  }

  function update_exit_on_display() {
    state.exit_on.clamp_display_num(1, state.starts.length);
    state.exit_on.update_num();
  }

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
    <div class="sub-title">Start</div>
    {#if state.exit_on.active}
      <StartTable
        {state}
        on_x={() => {
          state.exit_on.clamp_display_num(1, state.starts.length);
          state.exit_on.update_num();
          state = state;
        }}
        style="margin: auto; width: 50%;"
      />
    {:else}
      <StartTable {state} style="margin: auto; width: 50%;" />
    {/if}
    <div id="plus-button">
      <button
        on:click={() => {
          state.starts.push('');
          state.starts = state.starts;
        }}>+</button
      >
    </div>
    <div class="sub-title" id="config-options-title">Options</div>
    <div class="options">
      <div class="option">
        Cascade kill:<input type="checkbox" bind:checked={state.cascade_kill} />
      </div>
      <div class="option">
        Exit on: <input type="checkbox" bind:checked={state.exit_on.active} />
        {#if state.exit_on.active}
          <input
            type="number"
            min="1"
            max={state.starts.length}
            on:change={() => {
              state.exit_on.clamp_display_num(1, state.starts.length);
              state.exit_on.update_num();
              state = state;
            }}
            bind:value={state.exit_on.display_num}
          />
        {/if}
      </div>
    </div>

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
  /* .options {
    border: 1px solid;
  } */
  .option {
    padding-left: 10px;
    padding-right: 10px;
  }

  .sub-title {
    padding-top: 20px;
    padding-bottom: 10px;
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
