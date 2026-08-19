import { mount } from 'svelte';
import App from './App.svelte';
import './opmaak.css';
import { pasToe } from './thema';
import { inWebview, tauribrug, zetBrug } from './brug';

// De brug wordt pas gezet als de schil in een webview draait. Zonder webview
// start de schil wél, maar zegt hij dat hij niets kan opvragen — dat is
// duidelijker dan een leeg scherm.
async function start() {
  // Vóór het monteren, zodat er geen licht scherm opflitst voor wie donker wil.
  pasToe();
  if (inWebview()) {
    zetBrug(await tauribrug());
  }
  const doel = document.getElementById('schil');
  if (!doel) throw new Error('het element #schil ontbreekt in index.html');
  mount(App, { target: doel });
}

void start();
