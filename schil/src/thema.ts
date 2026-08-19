// Licht, donker of wat het systeem zegt.
//
// De keuze staat als attribuut op het wortelelement en niet als stijl in het
// element: het beleid staat `style-src 'self'` zonder `'unsafe-inline'` toe,
// dus alle kleuren komen uit het stijlblad en de keuze schakelt alleen tussen
// twee sets tokens.
//
// De voorkeur wordt lokaal onthouden en niet in de kluis. Zij is nodig vóór
// het ontgrendelen — een donker slot voor wie donker wil — en zij zegt niets
// over de inhoud.

export type Thema = 'systeem' | 'licht' | 'donker';

const SLEUTEL = 'dpofg.thema';

export const THEMANAAM: Record<Thema, string> = {
  systeem: 'volgt het systeem',
  licht: 'licht',
  donker: 'donker',
};

function geldig(waarde: string | null): waarde is Thema {
  return waarde === 'systeem' || waarde === 'licht' || waarde === 'donker';
}

/** Wat er is onthouden, of anders: volg het systeem. */
export function gekozenThema(): Thema {
  try {
    const bewaard = localStorage.getItem(SLEUTEL);
    return geldig(bewaard) ? bewaard : 'systeem';
  } catch {
    // Een webview zonder opslag is geen storing; dan volgt hij het systeem.
    return 'systeem';
  }
}

/** Zet het thema op het wortelelement en onthoudt de keuze. */
export function zetThema(thema: Thema): void {
  const wortel = document.documentElement;
  if (thema === 'systeem') {
    wortel.removeAttribute('data-thema');
  } else {
    wortel.setAttribute('data-thema', thema === 'donker' ? 'donker' : 'licht');
  }
  try {
    localStorage.setItem(SLEUTEL, thema);
  } catch {
    // Niet kunnen onthouden is hinderlijk en geen fout.
  }
}

/** De volgende stand in de rondgang systeem → licht → donker. */
export function volgende(huidig: Thema): Thema {
  return huidig === 'systeem' ? 'licht' : huidig === 'licht' ? 'donker' : 'systeem';
}

/** Past het onthouden thema toe. Wordt bij het opstarten aangeroepen. */
export function pasToe(): Thema {
  const thema = gekozenThema();
  zetThema(thema);
  return thema;
}
