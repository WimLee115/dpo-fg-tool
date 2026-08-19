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

// De tijdzone is die van het rechtsgebied en niet die van de machine. Zonder
// dat toont een laptop die op een andere zone staat een andere deadline dan de
// opdrachtregel, voor precies hetzelfde record.
describe('de tijdzone', () => {
  it('zet een moment om naar Nederlandse tijd en niet naar die van de machine', () => {
    expect(tijdstip('2026-08-21T09:20:00Z')).toBe('21-08-2026 11:20');
    expect(tijdstip('2026-01-15T09:00:00Z')).toBe('15-01-2026 10:00');
  });

  it('laat een moment laat op de avond op de volgende Nederlandse dag vallen', () => {
    expect(datum('2026-08-19T22:50:00Z')).toBe('20-08-2026');
    expect(tijdstip('2026-08-19T22:50:00Z')).toBe('20-08-2026 00:50');
  });

  it('geeft een streepje bij niets en bij onleesbare tekst', () => {
    expect(datum(null)).toBe('—');
    expect(tijdstip(null)).toBe('—');
    expect(datum('geen datum')).toBe('—');
    expect(tijdstip('geen datum')).toBe('—');
  });
});
