<script lang="ts">
  import { open, save, message, ask } from '@tauri-apps/api/dialog';
  import { appWindow } from '@tauri-apps/api/window';
  import StartTable from '$lib/Start-table.svelte';
  import { Config, State } from '$lib/State';
  import { rustCreateShortcut, rustLoadConfigFile, rustSaveConfigFile } from '$lib/rust-bindings';

  let state = new State();

  function errorMessage(err: string) {
    message(err, { title: 'Error message', type: 'error' }).catch((err) => console.log(err));
  }

  function infoMessage(info: string) {
    message(info, { title: 'Info message', type: 'info' }).catch((err) => console.log(err));
  }

  async function setTitle(title:string) {
    await appWindow.setTitle(`SMA (${title})`)
  }

  async function loadConfigFile() {
    let path = <string | null>await open({
      defaultPath: state.configPath,
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
    state.configPath = path;
    setTitle(state.configPath);
    rustLoadConfigFile(state.configPath)
      .then((rustConfig) => {
        console.log(rustConfig);
        state.config = Config.fromRustConfig(rustConfig);
      })
      .catch((err) => {
        console.log(err);
        errorMessage(err);
      });
  }

  async function saveWindow(
    name: string,
    extensions: string[],
    defaultPath?: string
  ): Promise<string | null> {
    return save({
      defaultPath: defaultPath,
      filters: [
        {
          name: name,
          extensions: extensions
        }
      ]
    });
  }

  async function saveConfigFile() {
    let path = await saveWindow('sma config', ['json'], state.configPath);
    if (path === null) {
      return;
    }

    state.configPath = path;
    setTitle(state.configPath);
    rustSaveConfigFile(state.config.toRustConfig(), state.configPath).catch((err) => {
      console.log(err);
      errorMessage(err);
    });
  }

  function testConfig() {
    // TODO: test this config by running it with sma.
  }

  async function generateShortcutWithConfig() {
    if (state.configPath === '') {
      const answer = await ask(
        'You need to save the config before creating a shortcut.\nDo you want to save this config?'
      );

      if (answer) {
        await saveConfigFile();
        generateShortcutWithConfig();
      }
    } else {
      let path = await saveWindow('shortcut', ['lnk']);
      if (path === null) {
        return;
      }
      rustCreateShortcut(state.config.toRustConfig(), state.configPath, path).catch((err) => {
        console.log(err);
        errorMessage(err);
      });
    }
  }

  function updateExitOnDisplay() {
    state.config.exitOn.clamp_display_num(1, state.config.start.length);
    state.config.exitOn.update_num();
    state = state;
  }

  function disableRightClick() {
    document.addEventListener(
      'contextmenu',
      (e) => {
        e.preventDefault();
        return false;
      },
      { capture: true }
    );
  }

  disableRightClick();
</script>

<main>
  <header>
    <h1>SMA</h1>
  </header>

  <body>
    <div class="sub-title">Start</div>
    {#if state.config.exitOn.active}
      <StartTable
        config={state.config}
        on_x={updateExitOnDisplay}
        style="margin: auto; width: 50%;"
      />
    {:else}
      <StartTable config={state.config} style="margin: auto; width: 50%;" />
    {/if}
    <div id="plus-button">
      <button
        on:click={() => {
          state.config.start.push('');
          state.config.start = state.config.start;
        }}>+</button
      >
    </div>
    <div class="sub-title" id="config-options-title">Options</div>
    <div class="options">
      <div class="option">
        Cascade kill:<input type="checkbox" bind:checked={state.config.cascadeKill} />
      </div>
      <div class="option">
        Exit on: <input type="checkbox" bind:checked={state.config.exitOn.active} />
        {#if state.config.exitOn.active}
          <input
            type="number"
            min="1"
            max={state.config.start.length}
            on:change={updateExitOnDisplay}
            bind:value={state.config.exitOn.displayNum}
          />
        {/if}
      </div>
    </div>

    <div id="load-save-buttons">
      <button id="config-file-load-button" on:click={loadConfigFile}>Load config</button>
      <button id="config-file-save-button" on:click={saveConfigFile}>Save config</button>
    </div>
    <button id="generate-shortcut-button" on:click={generateShortcutWithConfig}>
      Generate shortcut
    </button>
  </body>

  <footer>Created by GabbeHags</footer>
</main>

<style>
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
