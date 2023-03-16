<script>
  import Connect from "./lib/Connect.svelte";
</script>

<main class="container">
  <h1>Sonatype OpenVpn Client!</h1>

  <div class="row">
    <Connect />
  </div>

</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }
</style>
