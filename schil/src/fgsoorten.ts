// De vormen van het persoonlijke dossier.
//
// Een eigen bestand en geen hergebruik van `soorten.ts`. Dat is met opzet: de
// scheiding tussen de twee kluizen is geen weergavekwestie maar een grens, en
// die grens hoort ook in de bouw te zitten.

export type Spiegelstand = 'nooit_gespiegeld' | 'sluitend' | 'gewijzigd';

export interface Advies {
  kenmerk: string;
  onderwerp: string;
  uitgebracht_aan: string;
  uitgebracht_op: string;
  tijdig_betrokken: string;
  reactie: string | null;
  escalatiestappen: number;
  spiegelstand: Spiegelstand;
}

export interface Onafhankelijkheid {
  kenmerk: string;
  soort: string;
  grondslag: string;
  datum: string;
  van: string;
  opvolging: string | null;
  spiegelstand: Spiegelstand;
}

export interface Persoonlijkdossier {
  pad: string;
  ontgrendeld: boolean;
  adviezen: Advies[];
  gebeurtenissen: Onafhankelijkheid[];
}

export const SPIEGELNAAM: Record<Spiegelstand, string> = {
  nooit_gespiegeld: 'nooit gespiegeld — het tijdstip berust op uw eigen opgave',
  sluitend: 'gespiegeld en sindsdien niet gewijzigd',
  gewijzigd: 'sinds de laatste spiegeling gewijzigd',
};
