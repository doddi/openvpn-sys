<script>
  import { invoke } from "@tauri-apps/api/tauri"
  import {listen} from "@tauri-apps/api/event";

  let connectionStatus = "Unknown"
  let connectButtonDisabled = true

  async function connect(){
    connectButtonDisabled = true
    await invoke("connect")
    connectButtonDisabled = false;
  }

  async function setup() {
    await listen('connect_status', (event) => {
      console.log(event);
      connectionStatus = event.payload
      connectButtonDisabled = connectionStatus !== 'Connected' && connectionStatus !== 'Disconnected'
    });

    setInterval(() => {
      console.log('checking status');
      invoke('check_status');
      console.log(connectionStatus);
    }, 1000);
  }

  setup();
</script>

<div>
  <div class="row">
    <button on:click={connect} disabled={connectButtonDisabled} >
      {connectionStatus === 'Connected' ? "Disconnect" : "Connect" }
    </button>
  </div>
  <p>{connectionStatus}</p>
</div>

<style>
  button:disabled {
    background-color: grey;
    color: black;
  }
  button {
    background-color: blue;
    color: white;
  }
</style>
