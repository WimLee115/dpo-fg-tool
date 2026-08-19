<!--
  Eén regel in de werkbak.

  Drie dingen staan er onvoorwaardelijk bij en zijn niet weg te configureren:
  de citeerbare grondslag, het anker met de reden waarom de verplichting is
  ontstaan, en de eigenaar. Ze staan inline en niet achter een tooltip, want
  een tooltip is een verborgen feit voor toetsenbord, voorlezen en afdrukken.
-->
<script lang="ts">
  import type { Werkbakregel } from '../soorten';
  import { resterend, tijdstip } from '../opmaak';

  const {
    regel,
    nu,
    open,
  }: { regel: Werkbakregel; nu: Date; open: (soort: string, kenmerk: string) => void } = $props();
</script>

<li class="regel" class:onherstelbaar={regel.onherstelbaar}>
  <div class="kop">
    <span class="dossier">{regel.record_soort} {regel.record_kenmerk}</span>
    {#if regel.spoor}
      <span class="spoor">spoor {regel.spoor.nummer} van {regel.spoor.totaal}</span>
    {/if}
    <span class="wanneer">
      {#if regel.deadline}
        <time datetime={regel.deadline}>{tijdstip(regel.deadline)}</time>
        — {resterend(regel.deadline, nu)}
      {:else}
        de klok loopt nog niet
      {/if}
    </span>
  </div>

  <div class="wat">{regel.wat}</div>
  <div class="grondslag">{regel.grondslag}</div>
  <div class="anker">anker: {regel.anker}</div>
  <div class="eigenaar">eigenaar: {regel.eigenaar ?? 'niet belegd'}</div>

  <div>
    <button type="button" onclick={() => open(regel.record_soort, regel.record_kenmerk)}>
      Open {regel.record_soort} {regel.record_kenmerk}
    </button>
  </div>
</li>
