<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { open, save } from '@tauri-apps/api/dialog';

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
    config_path = path;
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
    config_path = path;
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
  let config_path: string = '';

  let starts: string[] = [''];

  async function get_start_with_dialog(index: number) {
    let path = <string | null>await open({
      multiple: false,
      filters: [
        {
          name: '',
          extensions: ['exe', 'shortcut']
        }
      ]
    });
    if (path === null) {
      return;
    }
    starts[index] = path;
  }

  disable_right_click();
</script>

<main>
  <header>
    <h1>SMA</h1>
  </header>

  <body>
    <form>
      <label>
        <div class="sub-title" id="config-start-title">Start</div>
        {#each starts as path, index}
          <label>
            <div>
              {index + 1}.
              <input type="text" bind:value={path} />
              <button on:click={() => get_start_with_dialog(index)}>Find</button>
              {#if starts.length > 1}
                <button
                  on:click={() => {
                    starts.splice(index, 1);
                    starts = starts;
                  }}>X</button
                >
              {/if}
            </div>
          </label>
        {/each}
        <div>
          <button
            on:click={() => {
              starts.push('');
              starts = starts;
            }}>+</button
          >
        </div>
      </label>
      <label>
        <div class="sub-title" id="config-options-title">Options</div>
        <label>
          Cascade kill: <input type="checkbox">
        </label>
      </label>
    </form>

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
