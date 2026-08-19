<!--
  De controleronde.

  Per ontvangerrol gegroepeerd, want dat is hoe het werk wordt verdeeld. Geen
  totaal en geen score: het aantal bevindingen zegt niets over hoe erg het is,
  en een getal boven de lijst zou precies zo gelezen worden.
-->
<script lang="ts">
  import type { Bevinding } from '../soorten';

  const { bevindingen }: { bevindingen: Bevinding[] } = $props();

  const NIVEAUNAAM: Record<Bevinding['niveau'], string> = {
    blokkerend: 'houdt vaststellen tegen',
    signalerend: 'blijft zichtbaar, blokkeert niet',
    rapporterend: 'ter kennisname',
  };

  const rollen = $derived([...new Set(bevindingen.map((b) => b.ontvanger))].sort());
</script>

<h1>Controleronde</h1>

{#if bevindingen.length === 0}
  <p class="leeg">Geen bevindingen op dit niveau.</p>
{:else}
  {#each rollen as rol (rol)}
    <section class="band" aria-labelledby={`rol-${rol}`}>
      <h2 id={`rol-${rol}`}>Voor de {rol}</h2>
      <ul class="regels">
        {#each bevindingen.filter((b) => b.ontvanger === rol) as b (b.regelcode + b.record_kenmerk)}
          <li class="regel" class:onherstelbaar={b.niveau === 'blokkerend'}>
            <div class="kop">
              <span class="dossier">{b.regelcode}</span>
              <span class="spoor">{b.record_soort} {b.record_kenmerk ?? ''}</span>
              <span class="wanneer">{NIVEAUNAAM[b.niveau]}</span>
            </div>
            <div class="wat">{b.toelichting}</div>
            <div class="grondslag">{b.grondslag}</div>
            {#if b.afwijking_tot}
              <div class="anker">
                er loopt een vastgelegde afwijking tot {b.afwijking_tot}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </section>
  {/each}
{/if}

<p class="terzijde">
  Er staat hier geen totaal en geen score. Het aantal bevindingen zegt niets over hoe ernstig de
  situatie is; wat het zegt staat per regel.
</p>
