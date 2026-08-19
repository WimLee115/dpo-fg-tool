import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import Fg from './Fg.svelte';
import { zetFgbrug, type Fgbrug } from './fgbrug';
import type { Persoonlijkdossier } from './fgsoorten';

function dossier(overschrijf: Partial<Persoonlijkdossier> = {}): Persoonlijkdossier {
  return {
    pad: '/thuis/fg-persoonlijk.dpofg',
    ontgrendeld: true,
    adviezen: [
      {
        kenmerk: 'ADV-2026-014',
        onderwerp: 'de invoering van een aanwezigheidsregistratie',
        uitgebracht_aan: 'de directie',
        uitgebracht_op: '2026-07-20T09:00:00Z',
        tijdig_betrokken: 'niet tijdig betrokken',
        reactie: null,
        escalatiestappen: 1,
        spiegelstand: 'nooit_gespiegeld',
      },
    ],
    gebeurtenissen: [
      {
        kenmerk: 'ONA-001',
        soort: 'met een sanctie gedreigd',
        grondslag: 'art. 38 lid 3, tweede volzin AVG',
        datum: '2026-08-01T09:00:00Z',
        van: 'de directeur',
        opvolging: null,
        spiegelstand: 'gewijzigd',
      },
    ],
    ...overschrijf,
  };
}

let aanroepen: string[];

function brug(): Fgbrug {
  aanroepen = [];
  return {
    async ontgrendel(wachtwoord) {
      aanroepen.push('ontgrendel');
      if (wachtwoord !== 'juist') throw new Error('de wachtwoordzin klopt niet');
      return dossier();
    },
    async vergrendel() {
      aanroepen.push('vergrendel');
    },
    async spiegel(kenmerk) {
      aanroepen.push(`spiegel:${kenmerk}`);
      const d = dossier();
      d.adviezen[0]!.spiegelstand = 'sluitend';
      return d;
    },
  };
}

beforeEach(() => zetFgbrug(brug()));
afterEach(cleanup);

async function open() {
  const gebruiker = userEvent.setup();
  render(Fg);
  await gebruiker.type(screen.getByLabelText(/wachtwoordzin/i), 'juist');
  await gebruiker.click(screen.getByRole('button', { name: /openen/i }));
  return gebruiker;
}

describe('het persoonlijke venster', () => {
  // Twee vensters die op elkaar lijken zijn de meest voorkomende manier
  // waarop iemand in de verkeerde kluis werkt.
  it('zegt op het slot al dat dit niet de kluis van de organisatie is', () => {
    render(Fg);
    expect(screen.getByText(/niet de kluis van de organisatie/i)).toBeTruthy();
  });

  it('noemt het voorbehoud over de houdbaarheid van deze constructie', () => {
    render(Fg);
    expect(screen.getByText(/niet vastgesteld/i)).toBeTruthy();
  });

  it('blijft het na openen zeggen in de balk', async () => {
    await open();
    expect(await screen.findByText(/niet de kluis van de organisatie/i)).toBeTruthy();
  });

  it('toont de adviezen met hun grondslag en hun betrokkenheid', async () => {
    await open();
    expect(await screen.findByText('ADV-2026-014')).toBeTruthy();
    expect(screen.getByText(/niet tijdig betrokken/)).toBeTruthy();
    expect(screen.getByText(/art\. 38 lid 3, tweede volzin AVG/)).toBeTruthy();
  });

  // "Nooit gespiegeld" en "sinds het spiegelen gewijzigd" zijn twee heel
  // verschillende antwoorden; die door elkaar halen is bij bewijs het
  // slechtste wat er is.
  it('onderscheidt nooit gespiegeld van sinds het spiegelen gewijzigd', async () => {
    await open();
    expect(await screen.findByText(/nooit gespiegeld/)).toBeTruthy();
    expect(screen.getByText(/sinds de laatste spiegeling gewijzigd/)).toBeTruthy();
  });

  it('biedt spiegelen aan waar dat nog nodig is, en niet waar het klopt', async () => {
    const gebruiker = await open();
    const knoppen = await screen.findAllByRole('button', { name: /hash vastleggen/i });
    expect(knoppen).toHaveLength(2);
    await gebruiker.click(knoppen[0]!);
    expect(aanroepen).toContain('spiegel:ADV-2026-014');
  });

  it('zegt wat er in de kluis van de organisatie belandt', async () => {
    await open();
    expect(await screen.findByText(/uitsluitend een hash/i)).toBeTruthy();
    expect(screen.getByText(/blijft in dit dossier/i)).toBeTruthy();
  });

  it('gaat op slot en laat niets van de inhoud staan', async () => {
    const gebruiker = await open();
    await screen.findByText('ADV-2026-014');
    await gebruiker.click(screen.getByRole('button', { name: 'Op slot' }));
    expect(await screen.findByLabelText(/wachtwoordzin/i)).toBeTruthy();
    expect(screen.queryByText('ADV-2026-014')).toBeNull();
  });

  it('meldt een verkeerde zin en houdt het dossier dicht', async () => {
    const gebruiker = userEvent.setup();
    render(Fg);
    await gebruiker.type(screen.getByLabelText(/wachtwoordzin/i), 'fout');
    await gebruiker.click(screen.getByRole('button', { name: /openen/i }));
    expect((await screen.findByRole('alert')).textContent).toMatch(/klopt niet/i);
    expect(screen.queryByText('ADV-2026-014')).toBeNull();
  });
});
