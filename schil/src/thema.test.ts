import { beforeEach, describe, expect, it } from 'vitest';
import { gekozenThema, pasToe, volgende, zetThema } from './thema';

beforeEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute('data-thema');
});

describe('het thema', () => {
  it('volgt standaard het systeem en zet dan geen attribuut', () => {
    expect(gekozenThema()).toBe('systeem');
    pasToe();
    expect(document.documentElement.hasAttribute('data-thema')).toBe(false);
  });

  // Wie licht wil op een donker systeem moet dat kunnen kiezen; anders wint de
  // systeemvoorkeur altijd en is de knop versiering.
  it('zet de eigen keuze in beide richtingen', () => {
    zetThema('donker');
    expect(document.documentElement.getAttribute('data-thema')).toBe('donker');
    zetThema('licht');
    expect(document.documentElement.getAttribute('data-thema')).toBe('licht');
    zetThema('systeem');
    expect(document.documentElement.hasAttribute('data-thema')).toBe(false);
  });

  it('onthoudt de keuze', () => {
    zetThema('donker');
    expect(gekozenThema()).toBe('donker');
  });

  it('gaat rond langs de drie standen', () => {
    expect(volgende('systeem')).toBe('licht');
    expect(volgende('licht')).toBe('donker');
    expect(volgende('donker')).toBe('systeem');
  });

  it('valt terug op het systeem bij onzin in de opslag', () => {
    localStorage.setItem('dpofg.thema', 'paars');
    expect(gekozenThema()).toBe('systeem');
  });
});
