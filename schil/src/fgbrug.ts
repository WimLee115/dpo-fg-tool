// De brug van het persoonlijke venster.
//
// Een eigen module met een eigen, veel kortere lijst commando's. Het venster
// van de organisatie kent deze niet en dit venster kent die van de organisatie
// niet; dat is naast de rechten van Tauri ook een grens in de bouw.

import type { Persoonlijkdossier } from './fgsoorten';

export interface Fgbrug {
  ontgrendel(wachtwoord: string): Promise<Persoonlijkdossier>;
  vergrendel(): Promise<void>;
  spiegel(kenmerk: string): Promise<Persoonlijkdossier>;
}

let huidige: Fgbrug | null = null;

export function zetFgbrug(brug: Fgbrug): void {
  huidige = brug;
}

export function fgbrug(): Fgbrug {
  if (!huidige) throw new Error('er is geen brug gezet; dit venster kan niets opvragen');
  return huidige;
}

export async function tauriFgbrug(): Promise<Fgbrug> {
  const { invoke } = await import('@tauri-apps/api/core');
  return {
    ontgrendel: (wachtwoord) => invoke('fg_ontgrendel', { wachtwoord }),
    vergrendel: () => invoke('fg_vergrendel'),
    spiegel: (kenmerk) => invoke('fg_spiegel', { kenmerk }),
  };
}
