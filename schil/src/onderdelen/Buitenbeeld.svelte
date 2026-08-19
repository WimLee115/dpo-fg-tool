<!--
  Wat er niet in de lijst staat.

  Hoort onder elke werkvoorraad. Een lege lijst die als "klaar" wordt gelezen
  is de duurste fout die een werkvoorraad kan maken, en het enige tegengif is
  dat de grens er permanent bij staat — niet inklapbaar, niet weg te zetten.
-->
<script lang="ts">
  import type { Buitenbeeld } from '../soorten';

  const { punten }: { punten: Buitenbeeld[] } = $props();
</script>

<section class="buitenbeeld" aria-labelledby="buitenbeeld-kop">
  <h2 id="buitenbeeld-kop">Niet in deze lijst</h2>
  <ul>
    {#each punten as punt (punt.wat)}
      <li><strong>{punt.wat}</strong> — {punt.waar}</li>
    {/each}
  </ul>
</section>
