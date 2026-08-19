<!--
  Het slot.

  Het scherm noemt welke kluis het opent en vraagt daar de zin bij. Er wordt
  nooit geprobeerd of een zin op de andere kluis past: dat zou een orakel zijn
  waarmee iemand kan uitvinden welke van twee wachtwoorden hij te pakken heeft.
-->
<script lang="ts">
  import Themaknop from './Themaknop.svelte';

  const {
    ontgrendel,
    bezig,
    fout,
    pad,
  }: {
    ontgrendel: (wachtwoord: string) => void;
    bezig: boolean;
    fout: string | null;
    pad: string;
  } = $props();

  let wachtwoord = $state('');

  function verstuur(gebeurtenis: SubmitEvent) {
    gebeurtenis.preventDefault();
    if (wachtwoord.length > 0) ontgrendel(wachtwoord);
  }
</script>

<div class="slot">
  <form onsubmit={verstuur}>
    <h1>De kluis openen</h1>
    <p class="terzijde">{pad}</p>

    <label>
      Wachtwoordzin
      <!-- svelte-ignore a11y_autofocus -->
      <input type="password" bind:value={wachtwoord} autocomplete="off" autofocus />
    </label>

    {#if fout}
      <p class="fout" role="alert">{fout}</p>
    {/if}

    <button type="submit" class="hoofd" disabled={bezig || wachtwoord.length === 0}>
      {bezig ? 'Bezig met openen…' : 'Openen'}
    </button>

    <p><Themaknop /></p>

    <p class="terzijde">
      Er is geen herstelmogelijkheid voor deze zin. Het persoonlijke dossier van de functionaris
      heeft een eigen zin en een eigen venster; die twee lopen nooit door elkaar.
    </p>
  </form>
</div>
