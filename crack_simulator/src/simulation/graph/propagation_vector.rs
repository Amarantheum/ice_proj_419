use std::ops::{Mul, Add};


/// Propogation vector in edges
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct PVec {
    v: [f32; 2],
}

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