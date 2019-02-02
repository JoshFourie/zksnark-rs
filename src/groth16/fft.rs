use crate::groth16::coefficient_poly::CoefficientPoly;
use std::ops::{Add, Mul, Sub};
use std::iter::FromIterator;

pub struct PointWise<P> { points: Vec<Points<P>> }

pub struct Points<P> { degree: P, y: P }

impl<P> From<Vec<(P, P)>> for PointWise<P> {
    fn from(object: Vec<(P, P)>) -> Self {
        Self {
            points: 
                object.into_iter()
                .map( |(degree,y)| Points::from( (degree, y ) ) )
                .collect::<Vec<_>>()
        }   
    }
}

impl<P> From<(P, P)> for Points<P> { 
    fn from((degree, y): (P, P)) -> Self { Self { degree , y } }
}

impl<P> Add<Self> for PointWise<P> 
where
    P: Add<P, Output=P>,
{
    type Output = Self;    
    fn add(self, rhs: Self) -> Self {
        Self::from(
            self.points
                .into_iter()
                .zip(rhs.points.into_iter())
                .map(|(a, b)| {
                    (a.degree, a.y + b.y)
                })
                .collect::<Vec<_>>()
        )
    }
}


#[cfg(test)]
mod tests {
    use crate::groth16::fft::{Points, PointWise};

    fn pointwise_addition() {
        let Ax = PointWise::from(
            vec![
                Points::from( (0, 1) ),
                Points::from( (1, 0) ),
                Points::from( (2, 5) ),
                Points::from( (3, 22) ),
            ]
        );
        let Bx = PointWise::from(
            vec![
                Points::from( (0, 1) ),
                Points::from( (1, 3) ),
                Points::from( (2, 13) ),
                Points::from( (3, 37) ),
            ]
        );
        let Cx = PointWise::from(
            vec![
                Points::from( (0, 2) ),
                Points::from( (1, 3) ),
                Points::from( (2, 18) ),
                Points::from( (3, 59) ),
            ]
        );
        assert_eq!(Ax + Bx, Cx);
    }
}