//! Geheimhoudende typen die hun inhoud bij het opruimen overschrijven.
//!
//! Uitgangspunt uit het dreigingsmodel: sleutelmateriaal mag niet langer in het
//! werkgeheugen staan dan strikt nodig, en mag nooit in een logregel, een
//! foutmelding of een geheugendump herkenbaar terechtkomen. Daarom:
//!
//! * elk geheim type overschrijft zijn buffer bij `Drop` (`ZeroizeOnDrop`);
//! * `Debug` en `Display` tonen nooit inhoud;
//! * vergelijken gebeurt in constante tijd, zodat de vergelijkingstijd niets
//!   verraadt over hoeveel bytes overeenkwamen.

use std::fmt;

use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Een wachtwoord of wachtwoordzin zoals de gebruiker die invoert.
///
/// Wordt nooit opgeslagen; hij bestaat alleen tussen invoer en sleutelafleiding.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Wachtwoordzin(Vec<u8>);

impl Wachtwoordzin {
    pub fn nieuw(tekst: impl Into<String>) -> Self {
        let mut s = tekst.into();
        let bytes = s.as_bytes().to_vec();
        s.zeroize();
        Self(bytes)
    }

    pub fn uit_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_leeg(&self) -> bool {
        self.0.is_empty()
    }

    pub fn lengte(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for Wachtwoordzin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Wachtwoordzin(<verborgen>)")
    }
}

/// Ruw sleutelmateriaal van vaste lengte.
///
/// `N` is een compile-time lengte, zodat een sleutel van de verkeerde lengte
/// niet compileert in plaats van pas bij gebruik te falen.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Geheim<const N: usize>([u8; N]);

impl<const N: usize> Geheim<N> {
    pub fn uit_array(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    pub fn uit_slice(bytes: &[u8]) -> crate::Resultaat<Self> {
        if bytes.len() != N {
            return Err(crate::CryptoFout::OngeldigeLengte {
                veld: "geheim",
                verwacht: N,
                gekregen: bytes.len(),
            });
        }
        let mut buf = [0u8; N];
        buf.copy_from_slice(bytes);
        Ok(Self(buf))
    }

    /// Een geheim dat volledig uit nullen bestaat, om in te vullen.
    pub fn nul() -> Self {
        Self([0u8; N])
    }

    pub fn bytes(&self) -> &[u8; N] {
        &self.0
    }

    pub fn bytes_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }

    pub const fn lengte() -> usize {
        N
    }
}

impl<const N: usize> fmt::Debug for Geheim<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Geheim<{N}>(<verborgen>)")
    }
}

impl<const N: usize> PartialEq for Geheim<N> {
    /// Vergelijkt in constante tijd.
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl<const N: usize> Eq for Geheim<N> {}

/// Vergelijkt twee reeksen in constante tijd.
///
/// Reeksen van verschillende lengte zijn altijd ongelijk; de lengte zelf is
/// geen geheim, de inhoud wel.
#[must_use]
pub fn gelijk_in_constante_tijd(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wachtwoordzin_toont_geen_inhoud() {
        let w = Wachtwoordzin::nieuw("geheim wachtwoord");
        let weergave = format!("{w:?}");
        assert!(!weergave.contains("geheim"));
        assert_eq!(weergave, "Wachtwoordzin(<verborgen>)");
    }

    #[test]
    fn geheim_toont_geen_inhoud() {
        let g = Geheim::<32>::uit_array([7u8; 32]);
        let weergave = format!("{g:?}");
        assert_eq!(weergave, "Geheim<32>(<verborgen>)");
    }

    #[test]
    fn geheim_weigert_verkeerde_lengte() {
        let fout = Geheim::<32>::uit_slice(&[0u8; 16]).unwrap_err();
        assert_eq!(
            fout,
            crate::CryptoFout::OngeldigeLengte { veld: "geheim", verwacht: 32, gekregen: 16 }
        );
    }

    #[test]
    fn geheimen_vergelijken_op_inhoud() {
        let a = Geheim::<32>::uit_array([1u8; 32]);
        let b = Geheim::<32>::uit_array([1u8; 32]);
        let c = Geheim::<32>::uit_array([2u8; 32]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn constante_tijd_vergelijking() {
        assert!(gelijk_in_constante_tijd(b"abc", b"abc"));
        assert!(!gelijk_in_constante_tijd(b"abc", b"abd"));
        assert!(!gelijk_in_constante_tijd(b"abc", b"abcd"));
        assert!(gelijk_in_constante_tijd(b"", b""));
    }
}
