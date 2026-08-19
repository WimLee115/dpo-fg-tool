<!--
  De vervalprognose.

  Wat er vandaag al niet aantoonbaar is, staat apart van wat er nog aankomt.
  Die twee door elkaar halen laat een achterstand lezen als een gebeurtenis
  die nog moet plaatsvinden.
-->
<script lang="ts">
  import type { Vervalpunt } from '../soorten';
  import { datum } from '../opmaak';

  const {
    punten,
    dagen,
    kies,
    nu,
  }: { punten: Vervalpunt[]; dagen: number; kies: (dagen: number) => void; nu: Date } = $props();

  const verstreken = $derived(punten.filter((p) => new Date(p.vervalt_op) <= nu));
  const aanstaand = $derived(punten.filter((p) => new Date(p.vervalt_op) > nu));
</script>

<h1>Vervalprognose</h1>

<p>
  Horizon:
  {#each [30, 90, 365] as d (d)}
    <button type="button" onclick={() => kies(d)} aria-pressed={dagen === d}>{d} dagen</button>
  {/each}
</p>

<section class="band" aria-labelledby="verstreken-kop">
  <h2 id="verstreken-kop">Vandaag al niet aantoonbaar <span class="aantal">{verstreken.length}</span></h2>
  {#if verstreken.length === 0}
    <p class="leeg">Er is op dit moment geen eis die niet te bewijzen is.</p>
  {:else}
    <table class="tabel">
      <thead>
        <tr><th scope="col">eis</th><th scope="col">oorzaak</th><th scope="col">eigenaar</th><th scope="col">sinds</th></tr>
      </thead>
      <tbody>
        {#each verstreken as punt (punt.eis + punt.vervalt_op)}
          <tr>
            <th scope="row">{punt.eis}<div class="grondslag">{punt.grondslag}</div></th>
            <td>{punt.oorzaak}</td>
            <td>{punt.eigenaar ?? 'geen'}</td>
            <td>{datum(punt.vervalt_op)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<section class="band" aria-labelledby="aanstaand-kop">
  <h2 id="aanstaand-kop">Binnen {dagen} dagen <span class="aantal">{aanstaand.length}</span></h2>
  {#if aanstaand.length === 0}
    <p class="leeg">Er verloopt niets binnen deze horizon.</p>
  {:else}
    <table class="tabel">
      <thead>
        <tr><th scope="col">eis</th><th scope="col">oorzaak</th><th scope="col">eigenaar</th><th scope="col">vervalt op</th></tr>
      </thead>
      <tbody>
        {#each aanstaand as punt (punt.eis + punt.vervalt_op)}
          <tr>
            <th scope="row">{punt.eis}<div class="grondslag">{punt.grondslag}</div></th>
            <td>{punt.oorzaak}</td>
            <td>{punt.eigenaar ?? 'geen'}</td>
            <td>{datum(punt.vervalt_op)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<p class="terzijde">
  Dit is een lijst met eisen die onbewijsbaar worden, geen takenlijst en geen score. Een bestuur
  weegt een informatiebeveiligingsrisico als datum, niet als kleur.
</p>
