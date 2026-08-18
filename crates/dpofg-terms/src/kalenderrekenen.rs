//! Kalenderrekenkunde met maandeindeklem.
//!
//! `chrono` kent geen "tel een maand op", en met reden: het antwoord is niet
//! eenduidig. 31 januari plus één maand is 28 februari, 29 februari, 2 maart of
//! 3 maart, afhankelijk van de gekozen regel. Voor wettelijke termijnen is de
//! regel wél eenduidig vastgelegd:
//!
//! > Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 2 onder c: een in
//! > maanden omschreven termijn eindigt op de dag die in de laatste maand
//! > dezelfde cijferaanduiding draagt als de dag van de gebeurtenis. *Indien
//! > die dag in de laatste maand ontbreekt, eindigt de termijn met het
//! > verstrijken van de laatste dag van die maand.*
//!
//! Die laatste zin is de maandeindeklem, en die is de reden dat dit een eigen
//! module is met eigen tests: het is het rekenwerk waar het in de praktijk
//! misgaat.

use chrono::{Datelike, NaiveDate};

use crate::{Resultaat, TermijnFout};

/// Aantal dagen in een maand, met schrikkeljaar.
pub fn dagen_in_maand(jaar: i32, maand: u32) -> u32 {
    match maand {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_schrikkeljaar(jaar) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Of een jaar een schrikkeljaar is volgens de gregoriaanse kalender.
pub fn is_schrikkeljaar(jaar: i32) -> bool {
    (jaar % 4 == 0 && jaar % 100 != 0) || jaar % 400 == 0
}

/// Telt maanden op bij een datum, met maandeindeklem.
///
/// ```text
/// 15 januari  + 1 maand = 15 februari
/// 31 januari  + 1 maand = 28 februari  (29 februari in een schrikkeljaar)
/// 31 maart    + 1 maand = 30 april
/// 29 februari + 12 maand = 28 februari van het volgende jaar
/// ```
pub fn tel_maanden_op(datum: NaiveDate, aantal: u32) -> Resultaat<NaiveDate> {
    let totaal_maanden = datum.month0() as i64 + aantal as i64;
    let jaar = datum.year() as i64 + totaal_maanden.div_euclid(12);
    let maand = totaal_maanden.rem_euclid(12) as u32 + 1;

    let jaar = i32::try_from(jaar).map_err(|_| TermijnFout::DatumBuitenBereik)?;
    let laatste = dagen_in_maand(jaar, maand);
    // De klem: bestaat de dag niet in de doelmaand, dan de laatste dag daarvan.
    let dag = datum.day().min(laatste);

    NaiveDate::from_ymd_opt(jaar, maand, dag).ok_or(TermijnFout::DatumBuitenBereik)
}

/// Telt jaren op bij een datum, met klem op 29 februari.
///
/// 29 februari plus één jaar is 28 februari; er bestaat geen 29 februari in een
/// niet-schrikkeljaar en de termijn schuift dan niet stilzwijgend naar 1 maart.
pub fn tel_jaren_op(datum: NaiveDate, aantal: u32) -> Resultaat<NaiveDate> {
    tel_maanden_op(datum, aantal.saturating_mul(12))
}

/// Telt kalenderdagen op.
pub fn tel_dagen_op(datum: NaiveDate, aantal: u32) -> Resultaat<NaiveDate> {
    datum.checked_add_days(chrono::Days::new(aantal as u64)).ok_or(TermijnFout::DatumBuitenBereik)
}

/// Telt weken op, als zeven kalenderdagen per week.
///
/// De uitkomst valt daarmee altijd op dezelfde weekdag als de begindag, wat
/// aansluit bij "dezelfde naam" uit Verordening 1182/71, art. 3 lid 2 onder c.
pub fn tel_weken_op(datum: NaiveDate, aantal: u32) -> Resultaat<NaiveDate> {
    tel_dagen_op(datum, aantal.saturating_mul(7))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(j: i32, m: u32, dag: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(j, m, dag).unwrap()
    }

    #[test]
    fn schrikkeljaren() {
        assert!(is_schrikkeljaar(2024));
        assert!(is_schrikkeljaar(2000));
        assert!(!is_schrikkeljaar(2026));
        assert!(!is_schrikkeljaar(1900));
        assert!(!is_schrikkeljaar(2100));
        assert!(is_schrikkeljaar(2400));
    }

    #[test]
    fn dagen_per_maand() {
        assert_eq!(dagen_in_maand(2026, 1), 31);
        assert_eq!(dagen_in_maand(2026, 2), 28);
        assert_eq!(dagen_in_maand(2028, 2), 29);
        assert_eq!(dagen_in_maand(2026, 4), 30);
    }

    /// T-21: inzageverzoek 15 januari, maandtermijn.
    #[test]
    fn t21_gewone_maandtermijn() {
        assert_eq!(tel_maanden_op(d(2026, 1, 15), 1).unwrap(), d(2026, 2, 15));
    }

    /// T-22: inzageverzoek 31 januari. De klem grijpt in.
    #[test]
    fn t22_maandeindeklem() {
        assert_eq!(tel_maanden_op(d(2026, 1, 31), 1).unwrap(), d(2026, 2, 28));
        assert_eq!(tel_maanden_op(d(2028, 1, 31), 1).unwrap(), d(2028, 2, 29));
    }

    #[test]
    fn klem_bij_dertigdaagse_maanden() {
        assert_eq!(tel_maanden_op(d(2026, 3, 31), 1).unwrap(), d(2026, 4, 30));
        assert_eq!(tel_maanden_op(d(2026, 5, 31), 1).unwrap(), d(2026, 6, 30));
        assert_eq!(tel_maanden_op(d(2026, 8, 31), 1).unwrap(), d(2026, 9, 30));
    }

    #[test]
    fn maanden_over_de_jaarwisseling() {
        assert_eq!(tel_maanden_op(d(2026, 11, 15), 2).unwrap(), d(2027, 1, 15));
        assert_eq!(tel_maanden_op(d(2026, 12, 31), 2).unwrap(), d(2027, 2, 28));
        assert_eq!(tel_maanden_op(d(2026, 1, 15), 12).unwrap(), d(2027, 1, 15));
        assert_eq!(tel_maanden_op(d(2026, 1, 15), 25).unwrap(), d(2028, 2, 15));
    }

    #[test]
    fn nul_maanden_verandert_niets() {
        assert_eq!(tel_maanden_op(d(2026, 2, 28), 0).unwrap(), d(2026, 2, 28));
    }

    /// T-04: schrikkeljaar, 29 februari als anker.
    #[test]
    fn t04_negenentwintig_februari() {
        // 29 februari 2028 + 1 maand = 29 maart 2028.
        assert_eq!(tel_maanden_op(d(2028, 2, 29), 1).unwrap(), d(2028, 3, 29));
        // 29 februari 2028 + 1 jaar = 28 februari 2029; 29 februari bestaat niet.
        assert_eq!(tel_jaren_op(d(2028, 2, 29), 1).unwrap(), d(2029, 2, 28));
        // En + 4 jaar valt weer op 29 februari.
        assert_eq!(tel_jaren_op(d(2028, 2, 29), 4).unwrap(), d(2032, 2, 29));
    }

    #[test]
    fn jaren_optellen() {
        assert_eq!(tel_jaren_op(d(2026, 8, 18), 1).unwrap(), d(2027, 8, 18));
        assert_eq!(tel_jaren_op(d(2026, 8, 18), 4).unwrap(), d(2030, 8, 18));
    }

    #[test]
    fn weken_behouden_de_weekdag() {
        let start = d(2026, 9, 3); // donderdag
        let eind = tel_weken_op(start, 6).unwrap();
        assert_eq!(eind, d(2026, 10, 15));
        assert_eq!(start.weekday(), eind.weekday());
    }

    #[test]
    fn dagen_optellen_over_maandgrens() {
        assert_eq!(tel_dagen_op(d(2026, 1, 25), 10).unwrap(), d(2026, 2, 4));
        assert_eq!(tel_dagen_op(d(2026, 12, 28), 5).unwrap(), d(2027, 1, 2));
    }

    #[test]
    fn maandtermijn_is_geen_dertig_dagen() {
        // De reden dat de eenheid onderdeel van het type is: wie een maand als
        // dertig dagen doorrekent, komt hier twee dagen te laat uit.
        let via_maanden = tel_maanden_op(d(2026, 1, 31), 1).unwrap();
        let via_dagen = tel_dagen_op(d(2026, 1, 31), 30).unwrap();
        assert_eq!(via_maanden, d(2026, 2, 28));
        assert_eq!(via_dagen, d(2026, 3, 2));
        assert_ne!(via_maanden, via_dagen);
    }
}
