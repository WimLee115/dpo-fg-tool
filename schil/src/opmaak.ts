// Alles wat een waarde in tekst omzet staat hier, en niet in een scherm.
//
// De reden is niet netheid maar toetsbaarheid: een verkeerd afgeronde termijn
// is in een scherm niet te vinden en in een functie wel.

/** Een tijdstip als datum, in de vorm die de rest van het product gebruikt. */
export function datum(iso: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  const dag = String(d.getDate()).padStart(2, '0');
  const maand = String(d.getMonth() + 1).padStart(2, '0');
  return `${dag}-${maand}-${d.getFullYear()}`;
}

/** Datum en tijd, voor termijnen die op het uur aankomen. */
export function tijdstip(iso: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  const uur = String(d.getHours()).padStart(2, '0');
  const minuut = String(d.getMinutes()).padStart(2, '0');
  return `${datum(iso)} ${uur}:${minuut}`;
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
