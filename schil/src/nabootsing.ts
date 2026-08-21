// Een nagebootste brug, voor de tests en voor het bekijken van de schil
// zonder kluis.
//
// De gegevens zijn met opzet uitgesproken: een verstreken meldtermijn, een
// dossier dat op een anker wacht, en een lijst die niet leeg is. Een
// nabootsing met alleen vrolijke gevallen laat de schermen nooit doen waarvoor
// ze zijn gebouwd.

import type { Brug } from './brug';
import type {
  Bevinding,
  Buitenbeeld,
  Dossier,
  Kluisstand,
  Vervalpunt,
  Werkbakregel,
} from './soorten';

export interface Nabootsing extends Brug {
  /** Hoe vaak welk commando is aangeroepen; de tests kijken hiernaar. */
  aanroepen: string[];
}

export function nabootsing(nu = new Date('2026-08-19T09:00:00Z')): Nabootsing {
  const aanroepen: string[] = [];
  const uur = (n: number) => new Date(nu.getTime() + n * 3_600_000).toISOString();

  const regels: Werkbakregel[] = [
    {
      record_soort: 'incident',
      record_kenmerk: '2026-0041',
      wat: 'melding van een inbreuk aan de toezichthouder',
      grondslag: 'art. 33 lid 1 AVG',
      anker: 'kennisname door de organisatie — de klok loopt vanaf kennisname',
      deadline: uur(-6),
      band: 'onherstelbaar_verstreken',
      onherstelbaar: true,
      eigenaar: null,
      spoor: { nummer: 1, totaal: 3 },
    },
    {
      record_soort: 'verzoek',
      record_kenmerk: 'BTR-2026-012',
      wat: 'het verzoek afhandelen: inzage in het personeelsdossier',
      grondslag: 'art. 12 lid 3 AVG',
      anker: 'ontvangst van het verzoek',
      deadline: uur(18),
      band: 'onherstelbaar_vandaag',
      onherstelbaar: true,
      eigenaar: 'de behandelaar (J. Jansen)',
      spoor: null,
    },
    {
      record_soort: 'correctie',
      record_kenmerk: 'COR-001',
      wat: 'herstel: ZRP-04 op zorgplicht ZRP-2026',
      grondslag: 'interne norm; de correctieplicht volgt uit de verantwoordingsplicht',
      anker: 'de afgesproken einddatum',
      deadline: uur(24 * 40),
      band: 'loopt',
      onherstelbaar: false,
      eigenaar: 'de security officer (K. de Wit)',
      spoor: null,
    },
    {
      record_soort: 'incident',
      record_kenmerk: '2026-0042',
      wat: 'vastlegging van de inbreuk in het interne register',
      grondslag: 'art. 33 lid 5 AVG',
      anker: 'kennisname door de organisatie — het anker ontbreekt nog',
      deadline: null,
      band: 'wacht_op_anker',
      onherstelbaar: false,
      eigenaar: null,
      spoor: { nummer: 1, totaal: 1 },
    },
  ];

  const buiten: Buitenbeeld[] = [
    {
      wat: 'wat verloopt zonder dat er iets moet',
      waar: "bewijs met een geldigheidsvenster staat in 'dpofg prognose'",
    },
    {
      wat: 'de meldketen van de zorgplicht',
      waar: 'die geldt alleen voor aangewezen entiteiten, en er is nog geen entiteitrecord',
    },
  ];

  const dossiers: Record<string, Dossier> = {
    'incident/2026-0041': {
      kop: {
        id: 'aaaa',
        soort: 'incident',
        kenmerk: '2026-0041',
        status: 'concept',
        gewijzigd_op: nu.toISOString(),
      },
      volledigheid: {
        soort: 'incident',
        verplicht: 14,
        compleet: 11,
        ontbreekt: [
          {
            veld: 'incident.oorzaak',
            omschrijving: 'kies de oorzaakcategorie uit het kennispakket',
            grondslag: 'art. 33 lid 5 AVG',
            blokkeert_vaststelling: true,
          },
          {
            veld: 'incident.maatregelen',
            omschrijving: 'noem ten minste één genomen of voorgestelde maatregel',
            grondslag: 'art. 33 lid 3 onder d AVG',
            blokkeert_vaststelling: true,
          },
          {
            veld: 'incident.aantal_betrokkenen',
            omschrijving: 'schat hoeveel betrokkenen het betreft',
            grondslag: 'art. 33 lid 3 onder a AVG',
            blokkeert_vaststelling: false,
          },
        ],
      },
      velden: [
        { naam: 'kenmerk', waarde: '2026-0041', is_tijdstip: false, herkomst: null },
        {
          naam: 'omschrijving',
          waarde: 'verkeerd geadresseerde brief',
          is_tijdstip: false,
          herkomst: null,
        },
        {
          naam: 'kennisname op',
          waarde: '2026-08-19T07:00:00Z',
          is_tijdstip: true,
          herkomst: null,
        },
        {
          naam: 'aantasting',
          waarde: 'vertrouwelijkheid',
          is_tijdstip: false,
          herkomst: null,
        },
        {
          naam: 'meldreferentie',
          waarde: '—',
          is_tijdstip: false,
          herkomst: 'verschijnt zodra de melding is verzonden',
        },
      ],
    },
  };

  // Eén onberekenbare termijn hoort in de nabootsing te zitten. Dit is precies
  // het geval dat het makkelijkst stilzwijgend wegvalt, en een scherm dat het
  // niet toont ziet er compleet uit terwijl het dat niet is.
  const nietBeoordeeld: string[] = [
    '2026-0041 — de termijn is met dit kennispakket niet te berekenen',
  ];

  const bevindingen: Bevinding[] = [
    {
      regelcode: 'ZRP-02',
      niveau: 'blokkerend',
      ontvanger: 'directie',
      record_soort: 'zorgplicht',
      record_kenmerk: 'ZRP-2026',
      toelichting: 'A. de Vries is aangemeld als functionaris en tegelijk eigenaar van CBB-13',
      grondslag: 'art. 38 lid 6 AVG',
      afwijking_tot: null,
    },
    {
      regelcode: 'ZRP-04',
      niveau: 'signalerend',
      ontvanger: 'security officer',
      record_soort: 'zorgplicht',
      record_kenmerk: 'ZRP-2026',
      toelichting: '3 maatregelen zijn ingericht zonder bewijs van de uitvoering dat nu geldt',
      grondslag: 'art. 6 lid 4 Cyberbeveiligingsbesluit',
      afwijking_tot: uur(24 * 30),
    },
  ];

  const vervalpunten: Vervalpunt[] = [
    {
      eis: 'CBB-13: beleid en procedures voor het gebruik van cryptografie',
      grondslag: 'art. 21 lid 3 onder h Cyberbeveiligingswet',
      oorzaak: 'het bewijsstuk verloopt',
      record_soort: 'zorgplicht',
      record_kenmerk: 'ZRP-2026',
      eigenaar: 'de beheerder (J. Jansen)',
      vervalt_op: uur(24 * 45),
    },
    {
      eis: 'RIS-2025: een geldige risicobeoordeling over de hele organisatie',
      grondslag: 'art. 21 lid 1 Cyberbeveiligingswet',
      oorzaak: 'de risicobeoordeling verloopt',
      record_soort: 'risico',
      record_kenmerk: 'RIS-2025',
      eigenaar: 'de security officer',
      vervalt_op: uur(-24 * 30),
    },
  ];

  const stand: Kluisstand = {
    pad: '/home/proef/dossier.dpofg',
    ontgrendeld: true,
    kennispakket: 'nl-start 0.3-start',
    consolidatiedatum: '2026-08-19',
    ketenreikwijdte: 'De keten van 42 regels is intern samenhangend.',
    keten_in_orde: true,
  };

  return {
    aanroepen,
    async ontgrendel(_pad, wachtwoord) {
      aanroepen.push('ontgrendel');
      if (wachtwoord !== 'juist') throw new Error('de wachtwoordzin klopt niet');
      return stand;
    },
    async vergrendel() {
      aanroepen.push('vergrendel');
    },
    async stand() {
      aanroepen.push('stand');
      return stand;
    },
    async werkbak() {
      aanroepen.push('werkbak');
      return { peilmoment: nu.toISOString(), regels };
    },
    async buitenbeeld() {
      aanroepen.push('buitenbeeld');
      return buiten;
    },
    async dossier(soort, kenmerk) {
      aanroepen.push(`dossier:${soort}/${kenmerk}`);
      const d = dossiers[`${soort}/${kenmerk}`];
      if (!d) throw new Error(`geen ${soort} met kenmerk '${kenmerk}'`);
      return d;
    },
    async controle() {
      aanroepen.push('controle');
      return {
        peilmoment: nu.toISOString(),
        bevindingen,
        beoordeeld: 7,
        niet_beoordeeld: nietBeoordeeld,
      };
    },
    async prognose(dagen) {
      aanroepen.push(`prognose:${dagen}`);
      return vervalpunten;
    },
    async toonPersoonlijkVenster() {
      aanroepen.push('toonPersoonlijkVenster');
    },
  };
}
