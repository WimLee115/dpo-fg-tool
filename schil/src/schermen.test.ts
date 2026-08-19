import { cleanup, render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import App from './App.svelte';
import Teller from './onderdelen/Teller.svelte';
import { zetBrug } from './brug';
import { nabootsing, type Nabootsing } from './nabootsing';

const NU = new Date('2026-08-19T09:00:00Z');
let brug: Nabootsing;

beforeEach(() => {
  brug = nabootsing(NU);
  zetBrug(brug);
});

afterEach(cleanup);

async function ontgrendel(zin = 'juist') {
  const gebruiker = userEvent.setup();
  render(App, { nu: NU });
  await gebruiker.type(screen.getByLabelText(/wachtwoordzin/i), zin);
  await gebruiker.click(screen.getByRole('button', { name: /openen/i }));
  return gebruiker;
}

describe('het slot', () => {
  it('opent de kluis en toont daarna de werkbak', async () => {
    await ontgrendel();
    expect(await screen.findByRole('heading', { name: 'Werkbak' })).toBeTruthy();
    expect(brug.aanroepen).toContain('ontgrendel');
    expect(brug.aanroepen).toContain('werkbak');
  });

  it('zegt wat er misging en blijft op het slot staan', async () => {
    await ontgrendel('fout');
    expect((await screen.findByRole('alert')).textContent).toMatch(/klopt niet/i);
    expect(screen.queryByRole('heading', { name: 'Werkbak' })).toBeNull();
  });

  // Het slot noemt de kluis en probeert nooit of de zin op de andere past:
  // dat zou een orakel zijn.
  it('noemt welke kluis het opent', () => {
    render(App, { nu: NU });
    expect(screen.getByText(/de kluis van de organisatie/i)).toBeTruthy();
  });
});

describe('de werkbak', () => {
  it('zet de banden in de vaste volgorde en draait die niet om', async () => {
    await ontgrendel();
    const koppen = await screen.findAllByRole('heading', { level: 2 });
    const namen = koppen.map((k) => k.textContent?.trim().split(/\s+/).slice(0, 3).join(' '));
    expect(namen[0]).toMatch(/onherstelbaar en verstreken/);
    expect(namen[1]).toMatch(/onherstelbaar, verloopt vandaag/);
    // Er is geen enkele knop om op te sorteren.
    expect(screen.queryByRole('button', { name: /sorteer/i })).toBeNull();
  });

  it('draagt per regel de grondslag, het anker en de eigenaar', async () => {
    await ontgrendel();
    const regel = (await screen.findByText(/melding van een inbreuk/)).closest('li');
    expect(regel).toBeTruthy();
    const inhoud = within(regel as HTMLElement);
    expect(inhoud.getByText(/art\. 33 lid 1 AVG/)).toBeTruthy();
    expect(inhoud.getByText(/anker: kennisname/)).toBeTruthy();
    expect(inhoud.getByText(/eigenaar: niet belegd/)).toBeTruthy();
  });

  // Eén dossier levert net zoveel regels als het lopende klokken heeft.
  it('noemt het spoor, zodat één afgehandeld spoor niet als klaar leest', async () => {
    await ontgrendel();
    expect(await screen.findByText('spoor 1 van 3')).toBeTruthy();
  });

  it('zegt van een verplichting zonder anker dat de klok nog niet loopt', async () => {
    await ontgrendel();
    expect(await screen.findByText(/de klok loopt nog niet/)).toBeTruthy();
  });

  // Een lege lijst die als "klaar" wordt gelezen is de duurste fout die een
  // werkvoorraad kan maken.
  it('noemt onder de lijst wat er niet in staat', async () => {
    await ontgrendel();
    const kop = await screen.findByRole('heading', { name: 'Niet in deze lijst' });
    expect(kop).toBeTruthy();
    expect(screen.getByText(/dpofg prognose/)).toBeTruthy();
  });

  it('zegt hoeveel er buiten de filter valt', async () => {
    const gebruiker = await ontgrendel();
    await screen.findByRole('heading', { name: 'Werkbak' });
    await gebruiker.selectOptions(screen.getByLabelText(/toon alleen/i), 'correctie');
    expect(await screen.findByText(/vallen buiten de gekozen filter|valt buiten de gekozen filter/)).toBeTruthy();
  });
});

describe('het dossier', () => {
  it('opent vanuit de werkbak en toont de teller met de grondslag', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(await screen.findByRole('button', { name: /open incident 2026-0041/i }));
    expect(await screen.findByText('11 van de 14 verplichte onderdelen')).toBeTruthy();
    expect(screen.getByText(/art\. 33 lid 5 AVG/)).toBeTruthy();
  });

  it('toont waarom een veld er is wanneer dat uit een eerdere keuze volgt', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(await screen.findByRole('button', { name: /open incident 2026-0041/i }));
    expect(await screen.findByText(/verschijnt zodra de melding is verzonden/)).toBeTruthy();
  });
});

describe('de teller', () => {
  // Een doorlopende balk nodigt uit om als percentage te worden gelezen.
  it('tekent losse blokjes en geen balk, en noemt geen percentage', () => {
    const { container } = render(Teller, {
      volledigheid: { soort: 'incident', verplicht: 14, compleet: 11, ontbreekt: [] },
    });
    expect(container.querySelectorAll('.blokje')).toHaveLength(14);
    expect(container.querySelectorAll('.blokje.ingevuld')).toHaveLength(11);
    expect(container.textContent).not.toMatch(/%|procent/);
    expect(container.querySelector('progress')).toBeNull();
  });

  it('zet de blokkerende onderdelen boven de signalerende', () => {
    render(Teller, {
      volledigheid: {
        soort: 'incident',
        verplicht: 3,
        compleet: 1,
        ontbreekt: [
          { veld: 'a', omschrijving: 'signaal', grondslag: 'g', blokkeert_vaststelling: false },
          { veld: 'b', omschrijving: 'blokkade', grondslag: 'g', blokkeert_vaststelling: true },
        ],
      },
    });
    const items = screen.getAllByRole('listitem');
    expect(items[0]?.textContent).toContain('blokkade');
    expect(items[1]?.textContent).toContain('signaal');
  });
});

describe('de controleronde', () => {
  it('groepeert per ontvangerrol en geeft geen totaal', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(screen.getByRole('button', { name: 'Controle' }));
    expect(await screen.findByRole('heading', { name: /voor de directie/i })).toBeTruthy();
    expect(screen.getByText(/geen totaal en geen score/i)).toBeTruthy();
  });

  it('noemt een lopende afwijking bij de bevinding waarover zij gaat', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(screen.getByRole('button', { name: 'Controle' }));
    expect(await screen.findByText(/er loopt een vastgelegde afwijking/)).toBeTruthy();
  });
});

describe('de prognose', () => {
  // Achterstand en aanstaande gebeurtenis door elkaar halen laat het eerste
  // als het tweede lezen.
  it('scheidt wat vandaag al omviel van wat er nog aankomt', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(screen.getByRole('button', { name: 'Prognose' }));
    expect(await screen.findByRole('heading', { name: /vandaag al niet aantoonbaar/i })).toBeTruthy();
    expect(screen.getByRole('heading', { name: /binnen 90 dagen/i })).toBeTruthy();
  });

  it('haalt een andere horizon op wanneer daarom wordt gevraagd', async () => {
    const gebruiker = await ontgrendel();
    await gebruiker.click(screen.getByRole('button', { name: 'Prognose' }));
    await gebruiker.click(await screen.findByRole('button', { name: '365 dagen' }));
    expect(brug.aanroepen).toContain('prognose:365');
  });
});

describe('de sessie', () => {
  it('gaat op slot en laat niets van de inhoud staan', async () => {
    const gebruiker = await ontgrendel();
    await screen.findByRole('heading', { name: 'Werkbak' });
    await gebruiker.click(screen.getByRole('button', { name: 'Op slot' }));
    expect(await screen.findByLabelText(/wachtwoordzin/i)).toBeTruthy();
    expect(screen.queryByText(/melding van een inbreuk/)).toBeNull();
  });
});
