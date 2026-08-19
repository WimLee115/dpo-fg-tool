import { expect, test, type Page } from '@playwright/test';

// De schil krijgt hier een nagebootste brug in het venster geschoven, vóórdat
// de eigen code draait. Daarmee is de hele interface te toetsen zonder Rust,
// op de motoren waar het op aankomt.
async function metNabootsing(page: Page) {
  await page.addInitScript(() => {
    const nu = new Date('2026-08-19T09:00:00Z').getTime();
    const uur = (n: number) => new Date(nu + n * 3_600_000).toISOString();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (naam: string, args: Record<string, unknown>) => {
        switch (naam) {
          case 'ontgrendel':
            if (args.wachtwoord !== 'juist') throw new Error('de wachtwoordzin klopt niet');
            return {
              pad: '/proef/dossier.dpofg',
              ontgrendeld: true,
              kennispakket: 'nl-start 0.3-start',
              consolidatiedatum: '2026-08-19',
              ketenreikwijdte: 'De keten van 42 regels is intern samenhangend.',
              keten_in_orde: true,
            };
          case 'werkbak':
            return [
              {
                record_soort: 'incident',
                record_kenmerk: '2026-0041',
                wat: 'melding van een inbreuk aan de toezichthouder',
                grondslag: 'art. 33 lid 1 AVG',
                anker: 'kennisname door de organisatie',
                deadline: uur(-6),
                band: 'onherstelbaar_verstreken',
                onherstelbaar: true,
                eigenaar: null,
                spoor: { nummer: 1, totaal: 3 },
              },
              {
                record_soort: 'correctie',
                record_kenmerk: 'COR-001',
                wat: 'herstel: ZRP-04 op zorgplicht ZRP-2026',
                grondslag: 'interne norm',
                anker: 'de afgesproken einddatum',
                deadline: uur(24 * 40),
                band: 'loopt',
                onherstelbaar: false,
                eigenaar: 'de security officer (K. de Wit)',
                spoor: null,
              },
            ];
          case 'buitenbeeld':
            return [{ wat: 'wat verloopt zonder dat er iets moet', waar: "staat in 'dpofg prognose'" }];
          case 'controle':
            return [];
          case 'prognose':
            return [];
          case 'vergrendel':
            return null;
          default:
            throw new Error(`onbekend commando: ${naam}`);
        }
      },
    };
  });
  await page.goto('/');
}

async function ontgrendel(page: Page) {
  await metNabootsing(page);
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByRole('heading', { name: 'Werkbak' })).toBeVisible();
}

test('de schil opent en toont de werkbak', async ({ page }) => {
  await ontgrendel(page);
  await expect(page.getByText('melding van een inbreuk aan de toezichthouder')).toBeVisible();
  await expect(page.getByText('spoor 1 van 3')).toBeVisible();
});

// Dit is de belangrijkste toets van het hele bestand: het beleid staat
// `style-src 'self'` zonder `'unsafe-inline'` toe, en een stijl in het element
// zou daarop stuklopen — op WebKitGTK anders dan op Chromium.
test('er staat geen stijl in het element en geen inline script', async ({ page }) => {
  await ontgrendel(page);
  const metStijl = await page.locator('[style]').count();
  expect(metStijl).toBe(0);
  const inlineScripts = await page.locator('script:not([src])').count();
  expect(inlineScripts).toBe(0);
  const styleElementen = await page.locator('head style, body style').count();
  expect(styleElementen).toBe(0);
});

test('de banden staan in de vaste volgorde', async ({ page }) => {
  await ontgrendel(page);
  const koppen = await page.locator('section.band > h2').allTextContents();
  expect(koppen[0]).toContain('onherstelbaar en verstreken');
  expect(koppen[1]).toContain('loopt');
});

test('wat er niet in de lijst staat, staat er permanent onder', async ({ page }) => {
  await ontgrendel(page);
  await expect(page.getByRole('heading', { name: 'Niet in deze lijst' })).toBeVisible();
  // Niet in te klappen: er is geen enkel element dat hem verbergt.
  await expect(page.locator('.buitenbeeld details')).toHaveCount(0);
});

test('de schil is met het toetsenbord te bedienen', async ({ page }) => {
  await metNabootsing(page);

  // Het wachtwoordveld heeft de focus zodra het slot verschijnt; er hoeft dus
  // niet eerst met Tab naartoe te worden gegaan. Wie dat wel doet, springt
  // juist weg van het enige veld dat er is.
  await expect(page.getByLabel(/wachtwoordzin/i)).toBeFocused();
  await page.keyboard.type('juist');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'Werkbak' })).toBeVisible();

  // En daarna is de navigatie met Tab te bereiken zonder muis. Het aantal
  // toetsaanslagen staat hier niet vast: dat zou breken zodra er een element
  // bij komt, terwijl de eis is dát het bereikbaar is.
  const nav = page.getByRole('button', { name: 'Werkbak' });
  for (let i = 0; i < 6 && !(await nav.evaluate((el) => el === document.activeElement)); i++) {
    await page.keyboard.press('Tab');
  }
  await expect(nav).toBeFocused();
});

test('een verkeerde zin meldt zich en houdt het slot dicht', async ({ page }) => {
  await metNabootsing(page);
  await page.getByLabel(/wachtwoordzin/i).fill('fout');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByRole('alert')).toContainText(/klopt niet/i);
  await expect(page.getByRole('heading', { name: 'Werkbak' })).toHaveCount(0);
});

test('op slot laat niets van de inhoud staan', async ({ page }) => {
  await ontgrendel(page);
  await page.getByRole('button', { name: 'Op slot' }).click();
  await expect(page.getByLabel(/wachtwoordzin/i)).toBeVisible();
  await expect(page.getByText('melding van een inbreuk')).toHaveCount(0);
});

test('er staat nergens een percentage of een voortgangsbalk', async ({ page }) => {
  await ontgrendel(page);
  await expect(page.locator('progress')).toHaveCount(0);
  const tekst = (await page.locator('body').textContent()) ?? '';
  expect(tekst).not.toMatch(/\d+\s*%/);
});
