import { test, type Page } from '@playwright/test';

// Schermafdrukken van de echte, gebouwde schil. Geen test maar een
// hulpmiddel: het draait alleen wanneer er om wordt gevraagd.
const MAP = process.env.BEELDMAP ?? 'beeld';

async function metNabootsing(page: Page, pad = '/') {
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
              pad: '/home/fg/.local/share/dpo-fg-tool/dossier.dpofg',
              ontgrendeld: true,
              kennispakket: 'nl-start 0.3-start',
              consolidatiedatum: '2026-08-19',
              ketenreikwijdte: 'De keten van 412 regels is intern samenhangend.',
              keten_in_orde: true,
            };
          case 'werkbak':
            return {
              peilmoment: new Date(nu).toISOString(),
              regels: [
              { record_soort: 'incident', record_kenmerk: '2026-0041',
                wat: 'melding van een inbreuk aan de toezichthouder',
                grondslag: 'art. 33 lid 1 AVG',
                anker: 'kennisname door de organisatie — de klok loopt vanaf kennisname',
                deadline: uur(-9), band: 'onherstelbaar_verstreken', onherstelbaar: true,
                eigenaar: null, spoor: { nummer: 1, totaal: 3 } },
              { record_soort: 'verzoek', record_kenmerk: 'BTR-2026-012',
                wat: 'het verzoek afhandelen: inzage in het personeelsdossier',
                grondslag: 'art. 12 lid 3 AVG', anker: 'ontvangst van het verzoek',
                deadline: uur(11), band: 'onherstelbaar_vandaag', onherstelbaar: true,
                eigenaar: 'de behandelaar (J. Jansen)', spoor: null },
              { record_soort: 'woo', record_kenmerk: 'WOO-2026-004',
                wat: 'beslissen op het verzoek om informatie',
                grondslag: 'art. 4.4 Wet open overheid', anker: 'ontvangst van het verzoek',
                deadline: uur(24 * 5), band: 'onherstelbaar_deze_week', onherstelbaar: true,
                eigenaar: null, spoor: null },
              { record_soort: 'incident', record_kenmerk: '2026-0041',
                wat: 'vastlegging van de inbreuk in het interne register',
                grondslag: 'art. 33 lid 5 AVG; de verordening noemt geen termijn',
                anker: 'kennisname door de organisatie',
                deadline: uur(24 * 22), band: 'loopt', onherstelbaar: false,
                eigenaar: null, spoor: { nummer: 2, totaal: 3 } },
              { record_soort: 'correctie', record_kenmerk: 'COR-001',
                wat: 'herstel: ZRP-04 op zorgplicht ZRP-2026',
                grondslag: 'interne norm; de correctieplicht volgt uit de verantwoordingsplicht',
                anker: 'de afgesproken einddatum', deadline: uur(24 * 40), band: 'loopt',
                onherstelbaar: false, eigenaar: 'de security officer (K. de Wit)', spoor: null },
              { record_soort: 'incident', record_kenmerk: '2026-0042',
                wat: 'mededeling aan de betrokkenen',
                grondslag: 'art. 34 lid 1 AVG',
                anker: 'vaststelling van een hoog risico — het anker ontbreekt nog',
                deadline: null, band: 'wacht_op_anker', onherstelbaar: true,
                eigenaar: null, spoor: { nummer: 1, totaal: 2 } },
              ],
            };
          case 'buitenbeeld':
            return [
              { wat: 'wat verloopt zonder dat er iets moet',
                waar: "bewijs met een geldigheidsvenster, risicobeoordelingen en subverwerkerscontroles staan in 'dpofg prognose'" },
              { wat: 'de meldketen van de zorgplicht',
                waar: 'die geldt alleen voor aangewezen entiteiten, en er is nog geen entiteitrecord waaruit dat blijkt' },
              { wat: 'onvolledige dossiers zonder termijn',
                waar: "wat er per dossier ontbreekt, staat bij het dossier zelf en in 'dpofg controle'" },
            ];
          case 'dossier':
            return {
              kop: { id: 'a1', soort: 'incident', kenmerk: '2026-0041', status: 'concept',
                     gewijzigd_op: uur(-2) },
              volledigheid: { soort: 'incident', verplicht: 14, compleet: 11, ontbreekt: [
                { veld: 'incident.oorzaak', omschrijving: 'kies de oorzaakcategorie uit het kennispakket',
                  grondslag: 'art. 33 lid 5 AVG', blokkeert_vaststelling: true },
                { veld: 'incident.maatregelen', omschrijving: 'noem ten minste één genomen of voorgestelde maatregel',
                  grondslag: 'art. 33 lid 3 onder d AVG', blokkeert_vaststelling: true },
                { veld: 'incident.aantal_betrokkenen', omschrijving: 'schat hoeveel betrokkenen het betreft',
                  grondslag: 'art. 33 lid 3 onder a AVG', blokkeert_vaststelling: false },
              ] },
              velden: [
                { naam: 'kenmerk', waarde: '2026-0041', herkomst: null },
                { naam: 'omschrijving', waarde: 'verkeerd geadresseerde brief met loongegevens', herkomst: null },
                { naam: 'kanaal', waarde: 'intern vastgesteld', herkomst: null },
                { naam: 'kennisname op', waarde: '19-08-2026 00:00', herkomst: null },
                { naam: 'meldreferentie', waarde: '—', herkomst: 'verschijnt zodra de melding is verzonden' },
                { naam: 'betrokkenen geinformeerd op', waarde: '—', herkomst: 'alleen bij een hoog risico' },
              ],
            };
          case 'controle':
            return [
              { regelcode: 'ZRP-02', niveau: 'blokkerend', ontvanger: 'directie',
                record_soort: 'zorgplicht', record_kenmerk: 'ZRP-2026',
                toelichting: 'A. de Vries is aangemeld als functionaris en tegelijk eigenaar van CBB-13; toezicht op het eigen werk is geen toezicht',
                grondslag: 'art. 38 lid 6 AVG', afwijking_tot: null },
              { regelcode: 'ZRP-04', niveau: 'signalerend', ontvanger: 'security officer',
                record_soort: 'zorgplicht', record_kenmerk: 'ZRP-2026',
                toelichting: '3 maatregelen zijn ingericht zonder bewijs van de uitvoering dat nu geldt: CBB-09, CBB-12, CBB-15',
                grondslag: 'art. 6 lid 4 Cyberbeveiligingsbesluit', afwijking_tot: uur(24 * 30) },
              { regelcode: 'VWO-02', niveau: 'signalerend', ontvanger: 'contracteigenaar',
                record_soort: 'leverancier', record_kenmerk: 'LEV-014',
                toelichting: '3 van de acht onderdelen van artikel 28 lid 3 hebben geen vindplaats in het contract: d, f, g',
                grondslag: 'art. 28 lid 3 AVG', afwijking_tot: null },
              { regelcode: 'REG-05', niveau: 'blokkerend', ontvanger: 'proceseigenaar',
                record_soort: 'verwerking', record_kenmerk: '0412-K',
                toelichting: 'er is geen bewaartermijn vastgelegd en geen gemotiveerde uitstelafspraak',
                grondslag: 'art. 30 lid 1 onder f AVG', afwijking_tot: null },
            ];
          case 'prognose':
            return [
              { eis: 'RIS-2025: een geldige risicobeoordeling over de hele organisatie',
                grondslag: 'art. 21 lid 1 Cyberbeveiligingswet', oorzaak: 'de risicobeoordeling verloopt',
                record_soort: 'risico', record_kenmerk: 'RIS-2025',
                eigenaar: 'de security officer', vervalt_op: uur(-24 * 12) },
              { eis: 'CBB-13: beleid en procedures voor het gebruik van cryptografie',
                grondslag: 'art. 21 lid 3 onder h Cyberbeveiligingswet', oorzaak: 'het bewijsstuk verloopt',
                record_soort: 'zorgplicht', record_kenmerk: 'ZRP-2026',
                eigenaar: 'de beheerder (J. Jansen)', vervalt_op: uur(24 * 41) },
              { eis: 'LEV-014: een nagelopen subverwerkerslijst',
                grondslag: 'art. 28 lid 2 en lid 4 AVG', oorzaak: 'de subverwerkerslijst is dan te lang niet nagelopen',
                record_soort: 'leverancier', record_kenmerk: 'LEV-014',
                eigenaar: null, vervalt_op: uur(24 * 63) },
            ];
          case 'fg_ontgrendel':
            if (args.wachtwoord !== 'juist') throw new Error('de wachtwoordzin klopt niet');
            return {
              pad: '/home/fg/.local/share/dpo-fg-tool/fg-persoonlijk.dpofg',
              ontgrendeld: true,
              adviezen: [
                { kenmerk: 'ADV-2026-014', onderwerp: 'de invoering van een aanwezigheidsregistratie',
                  uitgebracht_aan: 'de directie', uitgebracht_op: '2026-07-20T09:00:00Z',
                  tijdig_betrokken: 'niet tijdig betrokken', reactie: 'niet overgenomen',
                  escalatiestappen: 2, spiegelstand: 'gewijzigd' },
                { kenmerk: 'ADV-2026-021', onderwerp: 'de bewaartermijn van cameratoezicht',
                  uitgebracht_aan: 'de facilitair manager', uitgebracht_op: '2026-08-05T09:00:00Z',
                  tijdig_betrokken: 'naar behoren en tijdig betrokken', reactie: null,
                  escalatiestappen: 0, spiegelstand: 'sluitend' },
              ],
              gebeurtenissen: [
                { kenmerk: 'ONA-001', soort: 'met een sanctie gedreigd',
                  grondslag: 'art. 38 lid 3, tweede volzin AVG', datum: '2026-08-01T09:00:00Z',
                  van: 'de directeur bedrijfsvoering', opvolging: null, spiegelstand: 'nooit_gespiegeld' },
              ],
            };
          default:
            return null;
        }
      },
    };
  });
  await page.goto(pad);
}

test('schermafdrukken', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  const donker = process.env.DONKER === '1';
  if (donker) {
    await page.addInitScript(() => localStorage.setItem('dpofg.thema', 'donker'));
  }

  await metNabootsing(page);
  await page.screenshot({ path: `${MAP}/1-slot.png` });

  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await page.getByRole('heading', { name: 'Werkbak' }).waitFor();
  await page.screenshot({ path: `${MAP}/2-werkbak.png`, fullPage: true });

  await page.getByRole('button', { name: /open incident 2026-0041/i }).first().click();
  await page.getByText('11 van de 14 verplichte onderdelen').waitFor();
  await page.screenshot({ path: `${MAP}/3-dossier.png`, fullPage: true });

  await page.getByRole('button', { name: 'Controle' }).click();
  await page.getByRole('heading', { name: 'Controleronde' }).waitFor();
  await page.screenshot({ path: `${MAP}/4-controle.png`, fullPage: true });

  await page.getByRole('button', { name: 'Prognose' }).click();
  await page.getByRole('heading', { name: 'Vervalprognose' }).waitFor();
  await page.screenshot({ path: `${MAP}/5-prognose.png`, fullPage: true });

  await metNabootsing(page, '/fg.html');
  await page.getByLabel(/wachtwoordzin/i).fill('juist');
  await page.getByRole('button', { name: /openen/i }).click();
  await page.getByText('ADV-2026-014').waitFor();
  await page.screenshot({ path: `${MAP}/6-persoonlijk.png`, fullPage: true });
});
