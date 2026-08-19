import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    // Runes staan aan: de expliciete vorm laat in de code zien wat er
    // afhankelijk is van wat, en dat is bij een scherm dat de werkelijkheid
    // moet weergeven belangrijker dan beknoptheid.
    runes: true,
  },
};
