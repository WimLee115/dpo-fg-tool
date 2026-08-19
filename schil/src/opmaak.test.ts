import { describe, expect, it } from 'vitest';
import { datum, resterend, teller, tijdstip } from './opmaak';

describe('opmaak', () => {
  it('geeft een streepje in plaats van een lege datum', () => {
    expect(datum(null)).toBe('—');
    expect(datum('geen datum')).toBe('—');
    expect(tijdstip(null)).toBe('—');
  });

  // Een meldtermijn van 72 uur in "3 dagen" uitdrukken verliest juist het
  // verschil waar het op aankomt.
  it('telt onder de tweeënzeventig uur in uren en daarboven in dagen', () => {
    const nu = new Date('2026-08-19T09:00:00Z');
    expect(resterend('2026-08-20T09:00:00Z', nu)).toBe('nog 24 uur');
    expect(resterend('2026-08-22T10:00:00Z', nu)).toBe('nog 3 dagen');
  });

  it('zegt hoeveel te laat, en niet hoeveel er nog over is', () => {
    const nu = new Date('2026-08-19T09:00:00Z');
    expect(resterend('2026-08-19T03:00:00Z', nu)).toBe('6 uur te laat');
    expect(resterend('2026-08-10T09:00:00Z', nu)).toBe('9 dagen te laat');
  });

  it('kent geen anker zonder datum', () => {
    expect(resterend(null, new Date())).toBe('geen anker');
  });

  // De teller is voortgang, geen verwijt, en nooit een percentage.
  it('telt in onderdelen en niet in procenten', () => {
    expect(teller(11, 14)).toBe('11 van de 14 verplichte onderdelen');
    expect(teller(11, 14)).not.toMatch(/%|procent/);
  });
});
