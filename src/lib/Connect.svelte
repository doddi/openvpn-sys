<script>
  import { invoke } from "@tauri-apps/api/tauri"
  import {listen} from "@tauri-apps/api/event";

  let connectionStatus = "Ready"
  let connectButtonEnabled = true

  async function connect(){
    connectButtonEnabled = false
    await invoke("connect", { name })
  }

  async function setup() {
    await listen('connect_status', (event) => {
      connectionStatus = event.payload
    });
  }

  setup();
</script>

<div>
  <div class="row">
    <button on:click={connect} disabled={!connectButtonEnabled} >
      Connect
    </button>
  </div>
  <p>{connectionStatus}</p>
</div>

