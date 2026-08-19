// De vormen die over de brug komen. Zij komen één op één overeen met de
// Rust-kant; wijkt er iets af, dan faalt de brugtest en niet het scherm.

export type Band =
  | 'onherstelbaar_verstreken'
  | 'onherstelbaar_vandaag'
  | 'onherstelbaar_deze_week'
  | 'verstreken'
  | 'loopt'
  | 'wacht_op_anker';

/** De vaste volgorde. Niet om te draaien, en niet door de gebruiker te kiezen. */
export const BANDVOLGORDE: Band[] = [
  'onherstelbaar_verstreken',
  'onherstelbaar_vandaag',
  'onherstelbaar_deze_week',
  'verstreken',
  'loopt',
  'wacht_op_anker',
];

export const BANDNAAM: Record<Band, string> = {
  onherstelbaar_verstreken: 'onherstelbaar en verstreken',
  onherstelbaar_vandaag: 'onherstelbaar, verloopt vandaag',
  onherstelbaar_deze_week: 'onherstelbaar, verloopt deze week',
  verstreken: 'verstreken',
  loopt: 'loopt',
  wacht_op_anker: 'wacht op een anker',
};

export interface Spoor {
  nummer: number;
  totaal: number;
}

export interface Werkbakregel {
  record_soort: string;
  record_kenmerk: string;
  wat: string;
  grondslag: string;
  anker: string;
  deadline: string | null;
  band: Band;
  onherstelbaar: boolean;
  eigenaar: string | null;
  spoor: Spoor | null;
}

export interface Ontbrekend {
  veld: string;
  omschrijving: string;
  grondslag: string;
  blokkeert_vaststelling: boolean;
}

export interface Volledigheid {
  soort: string;
  verplicht: number;
  compleet: number;
  ontbreekt: Ontbrekend[];
}

export interface Recordkop {
  id: string;
  soort: string;
  kenmerk: string | null;
  status: string;
  gewijzigd_op: string;
}

export interface Dossier {
  kop: Recordkop;
  volledigheid: Volledigheid;
  /** De velden zoals ze in beeld horen te komen, in vaste volgorde. */
  velden: Veld[];
}

export interface Veld {
  naam: string;
  waarde: string;
  /** Waarom dit veld er is, wanneer het uit een eerdere keuze volgt. */
  herkomst: string | null;
}

export interface Weggevallen {
  veld: string;
  reden: string;
  waarde: string | null;
}

export interface Kluisstand {
  pad: string;
  ontgrendeld: boolean;
  kennispakket: string;
  consolidatiedatum: string;
  ketenreikwijdte: string;
  keten_in_orde: boolean;
}

export interface Bevinding {
  regelcode: string;
  niveau: 'blokkerend' | 'signalerend' | 'rapporterend';
  ontvanger: string;
  record_soort: string;
  record_kenmerk: string | null;
  toelichting: string;
  grondslag: string;
  afwijking_tot: string | null;
}

export interface Vervalpunt {
  eis: string;
  grondslag: string;
  oorzaak: string;
  record_soort: string;
  record_kenmerk: string;
  eigenaar: string | null;
  vervalt_op: string;
}

/** Wat er niet in de werkbak staat, en waar het dan wel staat. */
export interface Buitenbeeld {
  wat: string;
  waar: string;
}
