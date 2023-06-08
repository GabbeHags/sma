<script lang="ts">
  import { open } from '@tauri-apps/api/dialog';
  import type State from '$lib/State';

  export let state: State;
  export let on_x: () => void = () => {};
  export let style: string = '';

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
    state.starts[index] = path;
  }
</script>

<table {style}>
  <tbody>
    {#each state.starts as path, index}
      <tr>
        <td>{index + 1}.</td>
        <td><input type="text" placeholder="Path to Application to start" bind:value={path} /></td>
        <td
          ><button
            on:click={() => {
              get_start_with_dialog(index);
            }}>Find</button
          ></td
        >
        <button
          disabled={state.starts.length <= 1}
          on:click={() => {
            state.starts.splice(index, 1);
            state.starts = state.starts;
            on_x();
          }}>X</button
        >
      </tr>
    {/each}
  </tbody>
</table>
