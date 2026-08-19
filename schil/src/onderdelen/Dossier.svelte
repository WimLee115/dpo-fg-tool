<!--
  Het dossiervenster: één dossier tegelijk bewerken.

  Wat het toont komt uit Rust; er wordt hier niets afgeleid en niets geteld.
  De volledigheidsteller staat er onvoorwaardelijk bij, want er bestaat geen
  weergave waarin een dossier completer lijkt dan het is.
-->
<script lang="ts">
  import type { Dossier } from '../soorten';
  import Teller from './Teller.svelte';

  const { dossier, sluit }: { dossier: Dossier; sluit: () => void } = $props();
</script>

<div class="kop">
  <h1>{dossier.kop.soort} {dossier.kop.kenmerk ?? dossier.kop.id}</h1>
  <button type="button" onclick={sluit}>Terug naar de werkbak</button>
</div>

<p class="terzijde">status: {dossier.kop.status}</p>

<table class="tabel">
  <caption class="terzijde">De vastgelegde inhoud van dit dossier</caption>
  <thead>
    <tr><th scope="col">veld</th><th scope="col">waarde</th><th scope="col">herkomst</th></tr>
  </thead>
  <tbody>
    {#each dossier.velden as veld (veld.naam)}
      <tr>
        <th scope="row">{veld.naam}</th>
        <td>{veld.waarde}</td>
        <!-- Waar een veld uit een eerdere keuze volgt, staat die keuze erbij.
             Zonder die vermelding is een verdwenen verplichting niet te
             onderscheiden van een vergeten verplichting. -->
        <td class="terzijde">{veld.herkomst ?? ''}</td>
      </tr>
    {/each}
  </tbody>
</table>

<Teller volledigheid={dossier.volledigheid} />
