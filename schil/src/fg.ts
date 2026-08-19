import { mount } from 'svelte';
import Fg from './Fg.svelte';
import './opmaak.css';
import { tauriFgbrug, zetFgbrug } from './fgbrug';

// Dit bestand importeert `brug.ts` niet en kan dat ook niet: de commando's van
// de organisatiekluis bestaan in dit venster domweg niet.
async function start() {
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    zetFgbrug(await tauriFgbrug());
  }
  const doel = document.getElementById('schil');
  if (!doel) throw new Error('het element #schil ontbreekt in fg.html');
  mount(Fg, { target: doel });
}

void start();
