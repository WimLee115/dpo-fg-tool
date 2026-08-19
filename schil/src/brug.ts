// De brug naar Rust.
//
// Eén module, één eindige lijst commando's, geen jokertekens. Alles wat het
// scherm nodig heeft komt hierlangs; er is geen tweede route naar buiten en
// er staat geen enkele rekenregel aan deze kant.
//
// De implementatie is verwisselbaar. In de webview is het `invoke` van Tauri;
// in de motortests is het een nabootsing. Dat ene injectiepunt is wat de
// schermen zonder Rust te draaien toetsbaar maakt.

import type {
  Bevinding,
  Buitenbeeld,
  Dossier,
  Kluisstand,
  Vervalpunt,
  Werkbakregel,
} from './soorten';

/** De vorm van een aanroep. Namen zijn vast en worden nergens samengesteld. */
export interface Brug {
  ontgrendel(pad: string | null, wachtwoord: string): Promise<Kluisstand>;
  vergrendel(): Promise<void>;
  stand(): Promise<Kluisstand>;
  werkbak(): Promise<Werkbakregel[]>;
  buitenbeeld(): Promise<Buitenbeeld[]>;
  dossier(soort: string, kenmerk: string): Promise<Dossier>;
  controle(): Promise<Bevinding[]>;
  prognose(dagen: number): Promise<Vervalpunt[]>;
  /**
   * Opent het venster van het persoonlijke dossier.
   *
   * Het openen gebeurt in Rust; dit venster krijgt daarmee geen recht om zelf
   * vensters te maken, en het krijgt ook niets van dat dossier te zien.
   */
  toonPersoonlijkVenster(): Promise<void>;
}

let huidige: Brug | null = null;

/** Zet de brug. De motortests vervangen hem hiermee door een nabootsing. */
export function zetBrug(brug: Brug): void {
  huidige = brug;
}

export function brug(): Brug {
  if (!huidige) {
    throw new Error('er is geen brug gezet; de schil kan niets opvragen');
  }
  return huidige;
}

/**
 * De echte brug, over de IPC van Tauri.
 *
 * Wordt pas geladen wanneer de schil in een webview draait. In een gewone
 * browser bestaat `__TAURI_INTERNALS__` niet en zou het importeren van
 * `@tauri-apps/api` een fout geven bij het opstarten in plaats van bij het
 * gebruik.
 */
export async function tauribrug(): Promise<Brug> {
  const { invoke } = await import('@tauri-apps/api/core');
  return {
    ontgrendel: (pad, wachtwoord) => invoke('ontgrendel', { pad, wachtwoord }),
    vergrendel: () => invoke('vergrendel'),
    stand: () => invoke('stand'),
    werkbak: () => invoke('werkbak'),
    buitenbeeld: () => invoke('buitenbeeld'),
    dossier: (soort, kenmerk) => invoke('dossier', { soort, kenmerk }),
    controle: () => invoke('controle'),
    prognose: (dagen) => invoke('prognose', { dagen }),
    toonPersoonlijkVenster: () => invoke('toon_persoonlijk_venster'),
  };
}

/** Of de schil in een webview van Tauri draait. */
export function inWebview(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}
