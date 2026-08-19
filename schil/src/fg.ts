import { mount } from 'svelte';
import Fg from './Fg.svelte';
import './opmaak.css';
import { pasToe } from './thema';
import { tauriFgbrug, zetFgbrug } from './fgbrug';

// Dit bestand importeert `brug.ts` niet en kan dat ook niet: de commando's van
// de organisatiekluis bestaan in dit venster domweg niet.
async function start() {
  // Vóór het monteren, zodat er geen licht scherm opflitst voor wie donker wil.
  pasToe();
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    zetFgbrug(await tauriFgbrug());
  }
  const doel = document.getElementById('schil');
  if (!doel) throw new Error('het element #schil ontbreekt in fg.html');
  mount(Fg, { target: doel });
}

void start();
