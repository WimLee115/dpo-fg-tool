// Alles wat een waarde in tekst omzet staat hier, en niet in een scherm.
//
// De reden is niet netheid maar toetsbaarheid: een verkeerd afgeronde termijn
// is in een scherm niet te vinden en in een functie wel.

// De zone waarin dit product zijn tijdstippen toont: dezelfde waarin de
// termijnen worden gerekend, en niet die van de machine. Een functionaris die
// met zijn laptop in Lissabon zit, moet de Nederlandse meldtermijn zien staan
// en niet een uur ervoor. De opdrachtregel doet hetzelfde; anders tonen de
// twee gezichten van hetzelfde product twee tijden voor dezelfde deadline.
const ZONE = 'Europe/Amsterdam';

const DELEN = new Intl.DateTimeFormat('nl-NL', {
  timeZone: ZONE,
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
});

function ontleed(iso: string | null): Record<string, string> | null {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  const uit: Record<string, string> = {};
  for (const deel of DELEN.formatToParts(d)) uit[deel.type] = deel.value;
  return uit;
}

/** Een tijdstip als datum, in de vorm die de rest van het product gebruikt. */
export function datum(iso: string | null): string {
  const p = ontleed(iso);
  if (!p) return '—';
  return `${p.day}-${p.month}-${p.year}`;
}

/** Datum en tijd, voor termijnen die op het uur aankomen. */
export function tijdstip(iso: string | null): string {
  const p = ontleed(iso);
  if (!p) return '—';
  // Middernacht komt in sommige uitvoeringen als 24 terug in plaats van 00.
  const uur = p.hour === '24' ? '00' : p.hour;
  return `${p.day}-${p.month}-${p.year} ${uur}:${p.minute}`;
}

/**
 * Hoe lang het nog duurt, in woorden.
 *
 * Onder de tweeënzeventig uur in uren, daarboven in dagen. Een meldtermijn
 * van tweeënzeventig uur in "3 dagen" uitdrukken verliest juist het verschil
 * waar het op aankomt.
 */
export function resterend(iso: string | null, nu: Date): string {
  if (!iso) return 'geen anker';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return 'geen anker';
  const uren = Math.floor((d.getTime() - nu.getTime()) / 3_600_000);
  if (uren < 0) {
    const over = Math.abs(uren);
    return over < 72 ? `${over} uur te laat` : `${Math.floor(over / 24)} dagen te laat`;
  }
  if (uren < 72) return `nog ${uren} uur`;
  return `nog ${Math.floor(uren / 24)} dagen`;
}

/**
 * De teller, in de vorm die het product overal gebruikt.
 *
 * Nooit een percentage. "11 van de 14 onderdelen" is voortgang; 79 procent is
 * een cijfer dat in een bestuursstuk een eigen leven gaat leiden.
 */
export function teller(compleet: number, verplicht: number): string {
  return `${compleet} van de ${verplicht} verplichte onderdelen`;
}
