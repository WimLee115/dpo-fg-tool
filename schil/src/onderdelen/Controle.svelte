<!--
  De controleronde.

  Per ontvangerrol gegroepeerd, want dat is hoe het werk wordt verdeeld. Geen
  totaal en geen score: het aantal bevindingen zegt niets over hoe erg het is,
  en een getal boven de lijst zou precies zo gelezen worden.
-->
<script lang="ts">
  import type { Bevinding, Controleronde } from '../soorten';

  const { ronde }: { ronde: Controleronde } = $props();

  const bevindingen = $derived(ronde.bevindingen);

  const NIVEAUNAAM: Record<Bevinding['niveau'], string> = {
    blokkerend: 'houdt vaststellen tegen',
    signalerend: 'blijft zichtbaar, blokkeert niet',
    rapporterend: 'ter kennisname',
  };

  const rollen = $derived([...new Set(bevindingen.map((b) => b.ontvanger))].sort());
</script>

<h1>Controleronde</h1>

{#if ronde.niet_beoordeeld.length > 0}
  <!--
    Boven de lijst en niet eronder. Wat hier staat is niet nagekeken, en dat
    moet de lezer weten vóórdat hij de lijst eronder als het volledige beeld
    leest. Een termijn die niet te berekenen viel, is iets anders dan een
    termijn die in orde is.
  -->
  <section class="niet-nagekeken" aria-labelledby="niet-nagekeken-kop">
    <h2 id="niet-nagekeken-kop">Niet beoordeeld</h2>
    <ul>
      {#each ronde.niet_beoordeeld as regel (regel)}
        <li>{regel}</li>
      {/each}
    </ul>
    <p>
      Deze tellen niet mee in de ronde hieronder. Zwijgen zou hier betekenen dat wat niet is
      nagekeken, als in orde geldt.
    </p>
  </section>
{/if}

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
  Nagekeken: {ronde.beoordeeld} dossier{ronde.beoordeeld === 1 ? '' : 's'}. Er staat hier geen totaal en geen score. Het aantal bevindingen zegt niets over hoe ernstig de
  situatie is; wat het zegt staat per regel.
</p>

<style>
  .niet-nagekeken {
    margin-bottom: 1.5rem;
    padding: 0.75rem 1rem;
    border-left: 3px solid var(--blokkade, #b3261e);
    background: var(--vlak-op, rgba(179, 38, 30, 0.08));
  }

  .niet-nagekeken h2 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
  }

  .niet-nagekeken ul {
    margin: 0;
    padding-left: 1.2rem;
  }

  .niet-nagekeken p {
    margin: 0.5rem 0 0;
    font-size: 0.9rem;
    opacity: 0.8;
  }
</style>
