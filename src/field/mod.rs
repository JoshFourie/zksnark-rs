//! Defines the `Field` trait along with other utility functions for working
//! with fields.

extern crate itertools;

use self::itertools::unfold;
use std::ops::*;
use std::str::FromStr;

pub mod z251;

/// `FieldIdentity` only makes sense when defined with a Field. The reason
/// this trait is not a part of [`Field`] is to provide a "zero" element and a
/// "one" element to types that cannot define a multiplicative inverse to be a
/// `Field`. Currently this includes: `isize` and is used in `z251`.
///
/// As such `zero()` is the value that equals an element added to its additive
/// inverse and the `one()` is the value that equals an element multiplied by
/// its multiplicative inverse.
pub trait FieldIdentity {
    fn zero() -> Self;
    fn one() -> Self;
}

impl FieldIdentity for isize {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
}

/// A `Field` here has the same classical mathematical definition of a field.
pub trait Field:
    Sized
    + Add<Output = Self>
    + Neg<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + FieldIdentity
    + Copy
{
    fn mul_inv(self) -> Self;
    fn add_inv(self) -> Self {
        -self
    }
}

/// A line, `Polynomial`, represented as a vector of `Field` elements where the position in the
/// vector determines the power of the exponent.
///
/// For example: [1,2,0,4] is equivalent to f(x) = x^0 + 2x^1 + 4x^3
///
/// # Example
///
/// `degree` returns the highest exponent of the polynomial.
///
/// ```rust
/// use zksnark::field::z251::Z251;
/// use zksnark::field::*;
///
/// assert_eq!((1..5).count(),4);
/// assert_eq!((1..5).collect::<Vec<i32>>(),[1,2,3,4]);
///
/// assert_eq!(vec![1,2,0,4].into_iter().map(Z251::from).collect::<Vec<_>>().degree(), 3);
/// assert_eq!(vec![1,1,1,1,9].into_iter().map(Z251::from).collect::<Vec<_>>().degree(), 4);
/// ```
///
/// `evaluate` take the polynomial and evaluates it at the specified value.
///     For example: f(x) = 1 + x^2 + 3x^3 then f(1) = 1 + 1^2 + (3*1)^3
///
/// ```rust
/// use zksnark::field::z251::Z251;
/// use zksnark::field::*;
///
/// assert_eq!(vec![1,1,1].into_iter().map(Z251::from).collect::<Vec<_>>().evaluate(Z251::from(2)),
///     Z251::from(7));
/// assert_eq!(vec![1,1,4].into_iter().map(Z251::from).collect::<Vec<_>>().evaluate(Z251::from(2)),
///     Z251::from(19));
/// assert_eq!((1..5).map(Z251::from).collect::<Vec<_>>().evaluate(Z251::from(3)),
///     Z251::from(142));
///
/// ```
pub trait Polynomial<T>: From<Vec<T>>
where
    T: Field + PartialEq,
{
    fn coefficients(&self) -> Vec<T>;
    fn degree(&self) -> usize {
        let coeffs = self.coefficients();
        let mut degree = match coeffs.len() {
            0 => 0,
            d => d - 1,
        };

        for c in coeffs.iter().rev() {
            if *c == T::zero() && degree != 0 {
                degree -= 1;
            } else {
                return degree;
            }
        }

        degree
    }
    fn evaluate(&self, x: T) -> T {
        self.coefficients()
            .iter()
            .rev()
            .fold(T::zero(), |acc, y| (acc * x) + *y)
    }
    fn remove_leading_zeros(&mut self) {
        *self = self
            .coefficients()
            .into_iter()
            .rev()
            .skip_while(|&c| c == T::zero())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .into();
    }
}

impl<T> Polynomial<T> for Vec<T>
where
    T: Field + PartialEq,
{
    fn coefficients(&self) -> Vec<T> {
        self.clone()
    }
}

fn ext_euc_alg<T>(a: T, b: T) -> (T, T, T)
where
    T: Div<Output = T> + Mul<Output = T> + Sub<Output = T> + Eq + FieldIdentity + Copy,
{
    let (ref mut r0, ref mut r1) = (a, b);
    let (ref mut s0, ref mut s1) = (T::one(), T::zero());
    let (ref mut t0, ref mut t1) = (T::zero(), T::one());

    let (mut r, mut s, mut t, mut q): (T, T, T, T);

    while *r1 != T::zero() {
        q = *r0 / *r1;
        r = *r0 - q * (*r1);
        s = *s0 - q * (*s1);
        t = *t0 - q * (*t1);

        *r0 = *r1;
        *r1 = r;
        *s0 = *s1;
        *s1 = s;
        *t0 = *t1;
        *t1 = t;
    }

    (*r0, *s0, *t0)
}

fn chinese_remainder<T>(rems: &[T], moduli: &[T]) -> T
where
    T: Div<Output = T>
        + Mul<Output = T>
        + Sub<Output = T>
        + Add<Output = T>
        + Eq
        + FieldIdentity
        + Copy,
{
    let prod = moduli.iter().fold(T::one(), |acc, x| acc * *x);

    moduli
        .iter()
        .map(|x| prod / *x)
        .zip(moduli)
        .map(|(x, a)| {
            let (_, m, _) = ext_euc_alg(x, *a);
            m * x
        }).zip(rems)
        .map(|(a, b)| a * *b)
        .fold(T::zero(), |acc, x| acc + x)
}

/// `polynomial_division` is the devision of two `Polynomial`
///
/// ```rust
/// use zksnark::field::z251::Z251;
/// use zksnark::field::*;
///
/// let poly: Vec<Z251> = vec![1,0,3,1].into_iter().map(Z251::from).collect();
/// let polyDividend: Vec<Z251> = vec![0,0,9,1].into_iter().map(Z251::from).collect();
///
/// let num: Vec<Z251> = vec![1].into_iter().map(Z251::from).collect();
/// let den: Vec<Z251> = vec![1,0,245].into_iter().map(Z251::from).collect();
///
/// assert_eq!(polynomial_division(poly, polyDividend), (num, den));
/// ```
///
pub fn polynomial_division<P, T>(mut poly: P, mut dividend: P) -> (P, P)
where
    P: Polynomial<T>,
    T: Field + PartialEq,
{
    if dividend
        .coefficients()
        .into_iter()
        .skip_while(|&c| c == T::zero())
        .count()
        == 0
    {
        panic!("Dividend must be non-zero");
    }

    if dividend.degree() > poly.degree() {
        return (P::from(vec![T::zero()]), P::from(vec![T::zero()]));
    }

    poly.remove_leading_zeros();
    dividend.remove_leading_zeros();

    let mut q = vec![T::zero(); poly.degree() + 1 - dividend.degree()];
    let mut r = poly.coefficients();
    let d = dividend.degree();
    let c = dividend.coefficients()[d];

    while r.degree() >= d && r.len() != 0 {
        let s = r[r.degree()] / c;
        q[r.degree() - d] = s;
        r.as_mut_slice()
            .iter_mut()
            .rev()
            .skip_while(|&&mut c| c == T::zero())
            .zip(dividend.coefficients().into_iter().map(|a| a * s).rev())
            .for_each(|(r, b)| *r = *r - b);

        r.remove_leading_zeros();
    }

    (q.into(), r.into())
}

/// Yields an infinite list of powers of x starting from x^0.
///
/// ```rust
/// use zksnark::field::z251::Z251;
/// use zksnark::field::*;
///
/// assert_eq!(powers(Z251::from(5)).take(3).collect::<Vec<_>>(),
///     vec![1,5,25].into_iter().map(Z251::from).collect::<Vec<_>>());
///
/// assert_eq!(powers(Z251::from(2)).take(5).collect::<Vec<_>>(),
///     [1,2,4,8,16].iter_mut().map(|x| Z251::from(*x)).collect::<Vec<_>>());
/// ```
pub fn powers<T>(x: T) -> impl Iterator<Item = T>
where
    T: Field + Copy,
{
    use std::iter::once;
    let identity = T::one();

    once(identity).chain(unfold(identity, move |state| {
        *state = *state * x;
        Some(*state)
    }))
}

/// discrete fourier transformation
///
pub fn dft<T>(seq: &[T], root: T) -> Vec<T>
where
    T: Field,
{
    powers(root)
        .take(seq.len())
        .map(|ri| {
            seq.iter()
                .zip(powers(ri))
                .map(|(&a, r)| a * r)
                .fold(T::zero(), |acc, x| acc + x)
        }).collect::<Vec<_>>()
}

/// inverse discrete fourier transformation
///
pub fn idft<T>(seq: &[T], root: T) -> Vec<T>
where
    T: Field + From<usize>,
{
    powers(root.mul_inv())
        .take(seq.len())
        .map(|ri| {
            seq.iter()
                .zip(powers(ri))
                .map(|(&a, r)| a * r)
                .fold(T::zero(), |acc, x| acc + x)
                * T::from(seq.len()).mul_inv()
        }).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::z251::*;
    use super::*;

    extern crate quickcheck;
    use self::quickcheck::quickcheck;

    quickcheck! {
        fn prop_polynomial_evaluate(vec: Vec<usize>, eval_at: usize) -> bool {
            let poly: Vec<Z251> = vec.into_iter().map(|x| Z251::from(x % 251)).collect();
            let x: Z251 = Z251::from(eval_at);
            poly.evaluate(x) == poly
                .coefficients()
                .as_slice()
                .iter()
                .zip(powers(x))
                .fold(Z251::zero(), |acc, (&c, x)| acc + c * x)
        }
    }

    #[test]
    fn powers_test() {
        let root = Z251 { inner: 9 };
        assert_eq!(
            powers(root).take(5).collect::<Vec<_>>(),
            vec![
                Z251 { inner: 1 },
                Z251 { inner: 9 },
                Z251 { inner: 81 },
                Z251 { inner: 227 },
                Z251 { inner: 35 },
            ]
        );
    }

    #[test]
    fn dft_test() {
        // 25 divies 251 - 1 and 5 has order 25 in Z251
        let mut seq = [Z251::zero(); 25];
        seq[0] = 1.into();
        seq[1] = 2.into();
        seq[2] = 3.into();
        let root = 5.into();

        let result = vec![
            6, 86, 169, 189, 203, 131, 237, 118, 115, 91, 248, 177, 8, 48, 34, 136, 177, 203, 125,
            57, 237, 81, 9, 30, 122,
        ].into_iter()
        .map(Z251::from)
        .collect::<Vec<_>>();

        assert_eq!(dft(&seq[..], root), result);
    }

    #[test]
    fn idft_test() {
        // 25 divies 251 - 1 and 5 has order 25 in Z251
        let mut seq = [Z251::zero(); 25];
        seq[0] = 1.into();
        seq[1] = 2.into();
        seq[2] = 3.into();
        let root = 5.into();

        assert_eq!(idft(&dft(&seq[..], root)[..], root), seq.to_vec());
    }

    #[test]
    fn degree_test() {
        let a = [3, 0, 0, 0, 179, 0, 0, 6]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let b = [29, 112, 68]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let c = [3, 0, 0, 0, 179, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();

        assert_eq!(a.degree(), 7);
        assert_eq!(b.degree(), 2);
        assert_eq!(c.degree(), 7);
    }

    #[test]
    fn polynomial_division_test() {
        let a = [3, 0, 0, 0, 179, 0, 0, 6]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let b = [29, 112, 68]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let q = [209, 207, 78, 1, 131, 37]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let r = [217, 207]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();

        assert_eq!((q, r), polynomial_division(a, b));
    }

    #[test]
    #[should_panic]
    fn polynomial_divisionby0_test() {
        let a = [3, 0, 0, 0, 179, 0, 0, 6]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();
        let b = [0, 0, 0, 0, 0, 0, 0, 0]
            .iter()
            .map(|&c| Z251::from(c))
            .collect::<Vec<_>>();

        polynomial_division(a, b);
    }
}
