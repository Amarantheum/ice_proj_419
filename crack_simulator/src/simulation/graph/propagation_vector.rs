use std::ops::{Mul, Add, Neg};


/// Propogation vector in edges
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct PVec {
    v: [f32; 2],
}

#[allow(unused)]
impl PVec {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            v: [x, y]
        }
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.v[0]
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.v[1]
    }

    #[inline]
    pub fn modulus(&self) -> f32 {
        (*self * *self).sqrt()
    }
    #[inline]
    pub fn norm(&self) -> Self {
        let modulus = self.modulus();
        if modulus != 0_f32 {
            Self {
                v: [self.v[0] / modulus, self.v[1] / modulus]
            }
        } else {
            Self::default()
        }
    }

    #[inline]
    pub fn norm_mut(&mut self) {
        let modulus = self.modulus();
        if modulus != 0_f32 {
            self.v[0] /= modulus;
            self.v[1] /= modulus;
        }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.v[0] == 0_f32 && self.v[1] == 0_f32
    }

    #[inline]
    pub fn scale(self, s: f32) -> Self {
        Self {
            v: [self.x() * s, self.y() * s]
        }
    }
}


impl Mul for PVec {
    type Output = f32;

    #[inline]
    fn mul(self, v: Self) -> Self::Output {
        self.x() * v.x() + self.y() * v.y()
    }
}

impl Add for PVec {
    type Output = Self;

    #[inline]
    fn add(self, v: Self) -> Self::Output {
        PVec { v: [self.x() + v.x(), self.y() + v.y()] }
    }
}

impl Neg for PVec {
    type Output = Self;
    
    #[inline]
    fn neg(self) -> Self::Output {
        PVec { v: [-self.x(), -self.y() ]}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvec() {
        let mut v: PVec = Default::default();

        assert!(v.modulus() == 0.0);
        assert!(v.is_zero());

        v = v + PVec::new(10_f32, 10_f32);
        assert!(!v.is_zero());
        assert!(v.modulus() == 10.0 * 2_f32.sqrt());
        
        let vv = v.norm();
        v.norm_mut();
        assert!(v == vv);

        let dot = v * PVec::new(-10_f32, -10_f32);
        assert!(dot == -10_f32 * 2_f32.sqrt());
    }
}