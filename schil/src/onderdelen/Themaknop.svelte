<!--
  De keuze tussen licht, donker en wat het systeem zegt.

  De stand staat als woord op de knop en niet alleen als pictogram: een zonnetje
  zegt niet of je erop drukt om licht te krijgen of omdat het al licht is.
-->
<script lang="ts">
  import { gekozenThema, THEMANAAM, volgende, zetThema, type Thema } from '../thema';

  let thema = $state<Thema>(gekozenThema());

  function wissel() {
    thema = volgende(thema);
    zetThema(thema);
  }
</script>

<button type="button" class="thema" onclick={wissel} aria-label={`Weergave: ${THEMANAAM[thema]}. Klik om te wisselen.`}>
  Weergave: {THEMANAAM[thema]}
</button>
