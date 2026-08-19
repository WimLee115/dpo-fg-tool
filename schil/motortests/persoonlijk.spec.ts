import { expect, test, type Page } from '@playwright/test';

// Het persoonlijke venster heeft een eigen ingang, een eigen brug en een eigen
// wachtwoordzin. Wat hier wordt getoetst is dat die scheiding ook in de
// gebouwde bundel bestaat en niet alleen in de bedoeling.

async function metNabootsing(page: Page) {
  await page.addInitScript(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (naam: string, args: Record<string, unknown>) => {
        if (naam !== 'fg_ontgrendel' && naam !== 'fg_vergrendel' && naam !== 'fg_spiegel') {
          throw new Error(`onbekend commando: ${naam}`);
        }
        if (naam === 'fg_vergrendel') return null;
        if (naam === 'fg_ontgrendel' && args.wachtwoord !== 'juist') {
          throw new Error('de wachtwoordzin klopt niet');
        }
        return {
          pad: '/thuis/fg-persoonlijk.dpofg',
          ontgrendeld: true,
          adviezen: [
            {
              kenmerk: 'ADV-2026-014',
              onderwerp: 'de aanwezigheidsregistratie',
              uitgebracht_aan: 'de directie',
              uitgebracht_op: '2026-07-20T09:00:00Z',
              tijdig_betrokken: 'niet tijdig betrokken',
              reactie: null,
              escalatiestappen: 1,
              spiegelstand: 'nooit_gespiegeld',
            },
          ],
          gebeurtenissen: [],
        };
      },
    };
  });
  await page.goto('/fg.html');
}

test('het persoonlijke venster opent met zijn eigen zin', async ({ page }) => {
  await metNabootsing(page);
  await expect(page.getByText(/niet de kluis van de organisatie/i)).toBeVisible();
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByText('ADV-2026-014')).toBeVisible();
});

// De scheiding is niet alleen een kwestie van rechten maar ook van bouw: het
// persoonlijke venster laadt de code van de organisatie niet, want het kent
// haar niet.
test('het persoonlijke venster laadt de commandos van de organisatie niet', async ({ page }) => {
  const geladen: string[] = [];
  page.on('request', (r) => {
    if (r.url().endsWith('.js')) geladen.push(r.url());
  });
  await metNabootsing(page);
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByText('ADV-2026-014')).toBeVisible();

  expect(geladen.some((u) => u.includes('/fg-'))).toBe(true);
  expect(geladen.some((u) => u.includes('/organisatie-'))).toBe(false);
});

test('ook dit venster kent geen stijl in het element', async ({ page }) => {
  await metNabootsing(page);
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByText('ADV-2026-014')).toBeVisible();
  expect(await page.locator('[style]').count()).toBe(0);
  expect(await page.locator('head style, body style').count()).toBe(0);
});

test('het persoonlijke venster zegt wat er in de organisatiekluis belandt', async ({ page }) => {
  await metNabootsing(page);
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await expect(page.getByText(/uitsluitend een hash/i)).toBeVisible();
});
