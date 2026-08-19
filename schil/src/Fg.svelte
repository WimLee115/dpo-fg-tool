<!--
  Het persoonlijke dossier van de functionaris.

  Een eigen venster met een eigen wachtwoordzin. De balk bovenaan zegt
  onophoudelijk in welk dossier u werkt: twee vensters die op elkaar lijken
  zijn de meest voorkomende manier waarop iemand in de verkeerde kluis werkt.
-->
<script lang="ts">
  import { fgbrug } from './fgbrug';
  import { SPIEGELNAAM, type Persoonlijkdossier } from './fgsoorten';
  import { datum } from './opmaak';
  import Themaknop from './onderdelen/Themaknop.svelte';

  let dossier = $state<Persoonlijkdossier | null>(null);
  let wachtwoord = $state('');
  let bezig = $state(false);
  let fout = $state<string | null>(null);

  const open = $derived(dossier?.ontgrendeld === true);

  async function ontgrendel(gebeurtenis: SubmitEvent) {
    gebeurtenis.preventDefault();
    bezig = true;
    fout = null;
    try {
      dossier = await fgbrug().ontgrendel(wachtwoord);
      wachtwoord = '';
    } catch (e) {
      fout = e instanceof Error ? e.message : String(e);
    } finally {
      bezig = false;
    }
  }

  async function vergrendel() {
    await fgbrug().vergrendel();
    dossier = null;
  }

  async function spiegel(kenmerk: string) {
    fout = null;
    try {
      dossier = await fgbrug().spiegel(kenmerk);
    } catch (e) {
      fout = e instanceof Error ? e.message : String(e);
    }
  }
</script>

{#if !open}
  <div class="slot">
    <form onsubmit={ontgrendel}>
      <h1>Uw persoonlijke dossier</h1>
      <p class="terzijde">
        Dit is niet de kluis van de organisatie. Dit dossier heeft een eigen
        wachtwoordzin, en de organisatie kan de inhoud niet lezen.
      </p>

      <label>
        Wachtwoordzin van uw persoonlijke dossier
        <!-- svelte-ignore a11y_autofocus -->
        <input type="password" bind:value={wachtwoord} autocomplete="off" autofocus />
      </label>

      {#if fout}<p class="fout" role="alert">{fout}</p>{/if}

      <button type="submit" class="hoofd" disabled={bezig || wachtwoord.length === 0}>
        {bezig ? 'Bezig met openen…' : 'Openen'}
      </button>

      <p><Themaknop /></p>

      <p class="terzijde">
        Of deze constructie standhoudt tegenover eigendoms- en archiefaanspraken van de
        organisatie, is niet vastgesteld. De keuze om dit dossier te voeren is aan u.
      </p>
    </form>
  </div>
{:else if dossier}
  <div class="schil">
    <header class="balk">
      <strong>Persoonlijk dossier</strong>
      <span class="terzijde">{dossier.pad}</span>
      <span class="rek terzijde">niet de kluis van de organisatie</span>
      <Themaknop />
      <button type="button" onclick={vergrendel}>Op slot</button>
    </header>

    <main class="inhoud">
      {#if fout}<p class="fout" role="alert">{fout}</p>{/if}

      <section class="band" aria-labelledby="adviezen-kop">
        <h2 id="adviezen-kop">Adviezen <span class="aantal">{dossier.adviezen.length}</span></h2>
        {#if dossier.adviezen.length === 0}
          <p class="leeg">Er staat nog geen advies in dit dossier.</p>
        {:else}
          <ul class="regels">
            {#each dossier.adviezen as advies (advies.kenmerk)}
              <li class="regel">
                <div class="kop">
                  <span class="dossier">{advies.kenmerk}</span>
                  <span class="wanneer">{datum(advies.uitgebracht_op)}</span>
                </div>
                <div class="wat">{advies.onderwerp}</div>
                <div class="anker">uitgebracht aan {advies.uitgebracht_aan}</div>
                <div class="grondslag">betrokkenheid: {advies.tijdig_betrokken}</div>
                <div class="grondslag">
                  reactie: {advies.reactie ?? 'nog geen'}
                  {#if advies.escalatiestappen > 0}
                    · {advies.escalatiestappen} escalatiestap{advies.escalatiestappen === 1 ? '' : 'pen'}
                  {/if}
                </div>
                <div class="eigenaar">{SPIEGELNAAM[advies.spiegelstand]}</div>
                {#if advies.spiegelstand !== 'sluitend'}
                  <div>
                    <button type="button" onclick={() => spiegel(advies.kenmerk)}>
                      Hash vastleggen in de kluis van de organisatie
                    </button>
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="band" aria-labelledby="onafhankelijkheid-kop">
        <h2 id="onafhankelijkheid-kop">
          Onafhankelijkheid <span class="aantal">{dossier.gebeurtenissen.length}</span>
        </h2>
        {#if dossier.gebeurtenissen.length === 0}
          <p class="leeg">Er staat nog geen gebeurtenis in dit dossier.</p>
        {:else}
          <ul class="regels">
            {#each dossier.gebeurtenissen as g (g.kenmerk)}
              <li class="regel">
                <div class="kop">
                  <span class="dossier">{g.kenmerk}</span>
                  <span class="wanneer">{datum(g.datum)}</span>
                </div>
                <div class="wat">{g.soort}</div>
                <div class="grondslag">{g.grondslag}</div>
                <div class="anker">van {g.van}</div>
                <div class="grondslag">opvolging: {g.opvolging ?? 'nog geen'}</div>
                <div class="eigenaar">{SPIEGELNAAM[g.spiegelstand]}</div>
                {#if g.spiegelstand !== 'sluitend'}
                  <div>
                    <button type="button" onclick={() => spiegel(g.kenmerk)}>
                      Hash vastleggen in de kluis van de organisatie
                    </button>
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="buitenbeeld">
        <h2>Wat er in de kluis van de organisatie belandt</h2>
        <ul>
          <li>
            Uitsluitend een hash. Geen kenmerk, geen onderwerp, geen tekst: wat er is
            geadviseerd blijft in dit dossier.
          </li>
          <li>
            Wijzigt u een record na het spiegelen, dan komt de hash niet meer overeen en is er
            geen bewijs. Spiegel dan opnieuw.
          </li>
        </ul>
      </section>
    </main>
  </div>
{/if}
