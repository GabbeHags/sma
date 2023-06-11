<script lang="ts">
  import { open } from '@tauri-apps/api/dialog';
  import type { Config } from '$lib/State';

  export let config: Config;
  export let on_x: () => void = () => {};
  export let style: string = '';

  async function getStartWithDialog(index: number) {
    // TODO: if the selected file is a shortcut,
    // grab how that shortcut is executed and set that in this start.
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
    config.start[index] = path;
  }
</script>

<table {style}>
  <tbody>
    {#each config.start as path, index}
      <tr>
        <td>{index + 1}.</td>
        <td><input type="text" placeholder="Path to Application to start" bind:value={path} /></td>
        <td
          ><button
            on:click={() => {
              getStartWithDialog(index);
            }}>Find</button
          ></td
        >
        <button
          disabled={config.start.length <= 1}
          on:click={() => {
            config.start.splice(index, 1);
            config.start = config.start;
            on_x();
          }}>X</button
        >
      </tr>
    {/each}
  </tbody>
</table>
