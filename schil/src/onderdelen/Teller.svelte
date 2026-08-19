<!--
  De volledigheidsteller: losse blokjes en geen doorlopende balk.

  Een balk nodigt uit om als percentage te worden gelezen, en dit product
  geeft geen percentages. Veertien blokjes waarvan er elf gevuld zijn, is
  precies zo nauwkeurig en niet te vertalen naar een cijfer.
-->
<script lang="ts">
  import type { Volledigheid } from '../soorten';
  import { teller } from '../opmaak';

  const { volledigheid }: { volledigheid: Volledigheid } = $props();

  const blokjes = $derived(
    Array.from({ length: volledigheid.verplicht }, (_, i) => i < volledigheid.compleet),
  );
  const blokkades = $derived(volledigheid.ontbreekt.filter((o) => o.blokkeert_vaststelling));
  const signalen = $derived(volledigheid.ontbreekt.filter((o) => !o.blokkeert_vaststelling));
</script>

<section aria-labelledby="volledigheid-kop">
  <h3 id="volledigheid-kop">Volledigheid</h3>
  <p>{teller(volledigheid.compleet, volledigheid.verplicht)}</p>

  <!-- De blokjes zijn versiering naast de tekst hierboven; een voorlezer
       hoeft ze niet nog eens op te sommen. -->
  <div class="blokjes" aria-hidden="true">
    {#each blokjes as gevuld, i (i)}
      <span class="blokje" class:ingevuld={gevuld}></span>
    {/each}
  </div>

  {#if volledigheid.ontbreekt.length === 0}
    <p>Alle verplichte onderdelen zijn ingevuld.</p>
  {:else}
    <ul class="ontbreekt">
      {#each blokkades as o (o.veld)}
        <li class="blokkeert">
          <span class="veld">{o.veld}</span> — {o.omschrijving}
          <div class="grondslag">{o.grondslag}</div>
        </li>
      {/each}
      {#each signalen as o (o.veld)}
        <li>
          <span class="veld">{o.veld}</span> — {o.omschrijving}
          <div class="grondslag">{o.grondslag}</div>
        </li>
      {/each}
    </ul>
    <p class="terzijde">
      De onderdelen met een rode streep houden vaststellen tegen; de overige blijven zichtbaar
      maar blokkeren niet.
    </p>
  {/if}
</section>
