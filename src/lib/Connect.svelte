<script>
  import { invoke } from "@tauri-apps/api/tauri"
  import {listen} from "@tauri-apps/api/event";

  let connectionStatus = "Unknown"
  let connectButtonDisabled = 'disabled'

  async function connect(){
    console.log("Connect clicked");
    if (!connectButtonDisabled) {
      if (connectionStatus === 'Disconnected') {
        console.log("Sending connect");
        connectButtonDisabled = true;
        await invoke("connect", {command: "connect"})
      }
      else if (connectionStatus === 'Connected') {
        console.log("Sending disconnect");
        connectButtonDisabled = true;
        await invoke("connect", {command: "disconnect"})
      }
    }
  }

  async function setup() {
    await listen('connect_status', (event) => {
      console.log("Listen event: " + event.payload);
      connectionStatus = event.payload;
      console.log(connectionStatus);
      if (connectionStatus !== 'Connected' && connectionStatus !== 'Disconnected') {
        connectButtonDisabled = 'disabled';
      } else {
        connectButtonDisabled = '';
      }
      console.log("Listen event status: " + connectButtonDisabled);
    });
  }

  setup();
</script>

<div>
  <div class="row">
    <button on:click={connect} disabled='{connectButtonDisabled}'>
      {connectionStatus === 'Connected' ? "Disconnect" : "Connect" }
    </button>
  </div>
  <!--{isDisabled()}-->
  <p>Status: {connectionStatus}</p>
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
