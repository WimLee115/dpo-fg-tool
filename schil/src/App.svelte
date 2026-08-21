<!--
  De schil.

  Er is geen router: schermwisseling is toestand en geen adres. Een adresbalk
  zou bovendien betekenen dat er een geschiedenis is waarin het kenmerk van
  een dossier blijft staan, en dat is een lek dat niets oplost.
-->
<script lang="ts">
  import { brug } from './brug';
  import type { Buitenbeeld, Controleronde, Dossier, Kluisstand, Vervalpunt, Werkbakregel } from './soorten';
  import Slot from './onderdelen/Slot.svelte';
  import Werkbak from './onderdelen/Werkbak.svelte';
  import DossierScherm from './onderdelen/Dossier.svelte';
  import Controle from './onderdelen/Controle.svelte';
  import Prognose from './onderdelen/Prognose.svelte';
  import Themaknop from './onderdelen/Themaknop.svelte';

  type Scherm = 'werkbak' | 'controle' | 'prognose';

  // De schil houdt geen eigen klok bij. Het peilmoment komt met de werkbak
  // mee uit Rust, zodat de band en de resterende tijd nooit uiteen kunnen
  // lopen. Alleen wanneer er nog niets is opgehaald, valt hij terug op de
  // eigen klok — dan staat er ook nog niets om tegen af te meten.
  const { nu: begin = new Date() }: { nu?: Date } = $props();
  let opgehaald = $state<Date | null>(null);
  const peilmoment = $derived(opgehaald ?? begin);

  let stand = $state<Kluisstand | null>(null);
  let bezig = $state(false);
  let fout = $state<string | null>(null);
  let scherm = $state<Scherm>('werkbak');

  let regels = $state<Werkbakregel[]>([]);
  let buitenbeeld = $state<Buitenbeeld[]>([]);
  let ronde = $state<Controleronde | null>(null);
  let vervalpunten = $state<Vervalpunt[]>([]);
  let horizon = $state(90);
  let dossier = $state<Dossier | null>(null);

  const ontgrendeld = $derived(stand?.ontgrendeld === true);

  async function ontgrendel(wachtwoord: string) {
    bezig = true;
    fout = null;
    try {
      stand = await brug().ontgrendel(null, wachtwoord);
      await haalWerkbak();
    } catch (e) {
      fout = melding(e);
    } finally {
      bezig = false;
    }
  }

  async function vergrendel() {
    await brug().vergrendel();
    stand = null;
    regels = [];
    ronde = null;
    vervalpunten = [];
    dossier = null;
    scherm = 'werkbak';
  }

  async function haalWerkbak() {
    const voorraad = await brug().werkbak();
    regels = voorraad.regels;
    opgehaald = new Date(voorraad.peilmoment);
    buitenbeeld = await brug().buitenbeeld();
  }

  async function ga(naar: Scherm) {
    scherm = naar;
    dossier = null;
    fout = null;
    try {
      if (naar === 'werkbak') await haalWerkbak();
      if (naar === 'controle') ronde = await brug().controle();
      if (naar === 'prognose') vervalpunten = await brug().prognose(horizon);
    } catch (e) {
      fout = melding(e);
    }
  }

  async function kiesHorizon(dagen: number) {
    horizon = dagen;
    try {
      vervalpunten = await brug().prognose(dagen);
    } catch (e) {
      fout = melding(e);
    }
  }

  async function openDossier(soort: string, kenmerk: string) {
    fout = null;
    try {
      dossier = await brug().dossier(soort, kenmerk);
    } catch (e) {
      fout = melding(e);
    }
  }

  function melding(e: unknown): string {
    if (typeof e === 'string') return e;
    if (e instanceof Error) return e.message;
    return 'er ging iets mis, en de schil weet niet wat';
  }
</script>

{#if !ontgrendeld}
  <Slot {ontgrendel} {bezig} {fout} pad={stand?.pad ?? 'de kluis van de organisatie'} />
{:else}
  <div class="schil">
    <header class="balk">
      <strong>dpo-fg-tool</strong>
      <nav aria-label="Schermen">
        <button type="button" onclick={() => ga('werkbak')} aria-current={scherm === 'werkbak' && !dossier ? 'page' : undefined}>Werkbak</button>
        <button type="button" onclick={() => ga('controle')} aria-current={scherm === 'controle' ? 'page' : undefined}>Controle</button>
        <button type="button" onclick={() => ga('prognose')} aria-current={scherm === 'prognose' ? 'page' : undefined}>Prognose</button>
      </nav>
      <span class="rek terzijde">
        {stand?.kennispakket} · bijgewerkt tot {stand?.consolidatiedatum}
      </span>
      <Themaknop />
      <button type="button" onclick={() => brug().toonPersoonlijkVenster()}>
        Persoonlijk dossier
      </button>
      <button type="button" onclick={vergrendel}>Op slot</button>
    </header>

    <main class="inhoud">
      {#if stand && !stand.keten_in_orde}
        <p class="fout" role="alert">
          De ketencontrole is niet zonder bevindingen doorlopen. Zolang dat zo is, staat niet vast
          dat de gegevens onder deze schermen ongewijzigd zijn. {stand.ketenreikwijdte}
        </p>
      {/if}

      {#if fout}
        <p class="fout" role="alert">{fout}</p>
      {/if}

      {#if dossier}
        <DossierScherm {dossier} sluit={() => (dossier = null)} />
      {:else if scherm === 'werkbak'}
        <Werkbak {regels} {buitenbeeld} nu={peilmoment} open={openDossier} />
      {:else if scherm === 'controle'}
        {#if ronde}
          <Controle {ronde} />
        {/if}
      {:else}
        <Prognose punten={vervalpunten} dagen={horizon} kies={kiesHorizon} nu={peilmoment} />
      {/if}
    </main>
  </div>
{/if}
