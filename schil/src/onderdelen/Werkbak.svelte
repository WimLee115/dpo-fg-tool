<!--
  De werkbak: het eerste scherm na het ontgrendelen.

  Eén lijst over alle regimes heen, in een vaste volgorde die de gebruiker
  niet kan omdraaien. Er is geen sorteerknop en geen kolomkop om op te
  klikken: de volgorde is een uitspraak over wat er het eerst misgaat, en die
  hoort niet door de gebruiker te worden overruled.
-->
<script lang="ts">
  import type { Buitenbeeld, Band, Werkbakregel } from '../soorten';
  import { BANDNAAM, BANDVOLGORDE } from '../soorten';
  import Regel from './Werkbakregel.svelte';
  import BuitenbeeldLijst from './Buitenbeeld.svelte';

  const {
    regels,
    buitenbeeld,
    nu,
    open,
  }: {
    regels: Werkbakregel[];
    buitenbeeld: Buitenbeeld[];
    nu: Date;
    open: (soort: string, kenmerk: string) => void;
  } = $props();

  let soortfilter = $state('');

  const gefilterd = $derived(
    soortfilter ? regels.filter((r) => r.record_soort === soortfilter) : regels,
  );
  const weggefilterd = $derived(regels.length - gefilterd.length);

  const soorten = $derived([...new Set(regels.map((r) => r.record_soort))].sort());

  const perBand = $derived(
    BANDVOLGORDE.map((band: Band) => ({
      band,
      regels: gefilterd.filter((r) => r.band === band),
    })).filter((g) => g.regels.length > 0),
  );

  const onherstelbaar = $derived(gefilterd.filter((r) => r.onherstelbaar).length);
</script>

<h1>Werkbak</h1>
<p class="terzijde">
  {gefilterd.length} openstaande verplichting{gefilterd.length === 1 ? '' : 'en'}
  {#if onherstelbaar > 0}
    · {onherstelbaar} daarvan {onherstelbaar === 1 ? 'is' : 'zijn'} onherstelbaar: te laat is te
    laat, en dat staat in de melding zelf
  {/if}
</p>

{#if soorten.length > 1}
  <p>
    <label>
      Toon alleen
      <select bind:value={soortfilter}>
        <option value="">alle dossiersoorten</option>
        {#each soorten as s (s)}
          <option value={s}>{s}</option>
        {/each}
      </select>
    </label>
  </p>
{/if}

{#if weggefilterd > 0}
  <p class="fout">
    {weggefilterd} regel{weggefilterd === 1 ? '' : 's'} valt buiten de gekozen filter en staat
    hier niet.
  </p>
{/if}

{#if perBand.length === 0}
  <p class="leeg">Er staat niets open in deze lijst.</p>
{:else}
  {#each perBand as groep (groep.band)}
    <section class="band" data-band={groep.band} aria-labelledby={`band-${groep.band}`}>
      <h2 id={`band-${groep.band}`}>
        {BANDNAAM[groep.band]}
        <span class="aantal">{groep.regels.length}</span>
      </h2>
      <ul class="regels">
        {#each groep.regels as regel (regel.record_soort + regel.record_kenmerk + regel.wat)}
          <Regel {regel} {nu} {open} />
        {/each}
      </ul>
    </section>
  {/each}
{/if}

<p class="terzijde">
  De volgorde ligt vast en is niet om te draaien: onherstelbaar gaat vóór herstelbaar, en
  verstreken vóór aanstaand.
</p>

<BuitenbeeldLijst punten={buitenbeeld} />
