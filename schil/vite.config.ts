import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Eén bouwdoel voor alle drie de webviewmotoren, en niet per platform. Een
// per-platform doel zou betekenen dat een fout op macOS pas bij een
// macOS-bouw zichtbaar wordt; met één doel vangt de Linux-bouw het af.
export default defineConfig(({ mode }) => ({
  plugins: [svelte()],
  clearScreen: false,
  build: {
    target: ['safari15', 'chrome110'],
    cssTarget: ['safari15', 'chrome110'],
    modulePreload: { polyfill: false },
    // Geen bronkaarten in een uitgave: die zouden de volledige broncode van
    // de schil meeleveren aan iedereen die het pakket openmaakt.
    sourcemap: false,
    // Alles in de bundel. Geen CDN, geen weblettertypen, geen polyfilldienst.
    assetsInlineLimit: 0,
    emptyOutDir: true,
    rollupOptions: {
      // Twee ingangen, want twee kluizen zijn twee vensters met eigen rechten.
      // De scheiding is daarmee ook een bouwtijdgrens: het persoonlijke
      // venster laadt de commando's van de organisatie niet, want het kent ze
      // niet.
      input: {
        organisatie: 'index.html',
        fg: 'fg.html',
      },
    },
  },
  server: { port: 5173, strictPort: true },
  // Uitdrukkelijk op 127.0.0.1: `localhost` kan naar ::1 wijzen terwijl de
  // motortests op IPv4 wachten, en dan lijkt het alsof de bouw niet start.
  preview: { host: '127.0.0.1', port: 4173, strictPort: true },
  // Onder vitest moet Svelte de browserbouw laden en niet die voor de server:
  // zonder deze voorwaarde compileert de plug-in de componenten als
  // servercode en bestaat `mount` niet. De modus komt van vitest zelf, zodat
  // hier geen `process` nodig is en de typecontrole zonder Node-typen klopt.
  resolve: mode === 'test' ? { conditions: ['browser'] } : {},
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    globals: true,
    setupFiles: ['./src/proefopstelling.ts'],
  },
}));
