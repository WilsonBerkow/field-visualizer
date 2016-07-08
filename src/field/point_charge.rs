use na;
use na::Translate;

use std::ops::Add;
use std::ops::AddAssign;

use field::field_data::*;

pub struct PointCharge {
    pub charge: f64,
    pub loc: na::Point3<f64>,
}

impl PointCharge {
    pub fn new(charge: f64, loc: na::Point3<f64>) -> PointCharge {
        PointCharge { charge: charge, loc: loc }
    }
}

const FIELD_SCALE_FACTOR: f64 = 10000.0;
impl VectorField3 for PointCharge {
    fn field_data_at(&self, p: &na::Point3<f64>) -> FieldData {
        let dist_squared = na::distance_squared(&self.loc, p);
        let dist = dist_squared.sqrt();
        let force_mag = FIELD_SCALE_FACTOR * self.charge / dist_squared;
        let potential = FIELD_SCALE_FACTOR * self.charge / dist;
        let unit_vec = (p.clone() - self.loc) / dist;
        FieldData {
            force_vec: unit_vec * force_mag,
            force_mag: force_mag,
            potential: potential,
        }
    }
}

impl Translate<PointCharge> for na::Vector3<f64> {
    fn translate(&self, chg: &PointCharge) -> PointCharge {
        PointCharge::new(chg.charge, self.translate(&chg.loc))
    }
    fn inverse_translate(&self, chg: &PointCharge) -> PointCharge {
        PointCharge::new(chg.charge, self.inverse_translate(&chg.loc))
    }
}

impl Add<na::Vector3<f64>> for PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: na::Vector3<f64>) -> PointCharge {
        rhs.translate(&self)
    }
}

impl<'a> Add<na::Vector3<f64>> for &'a PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: na::Vector3<f64>) -> PointCharge {
        rhs.translate(self)
    }
}

impl<'a> Add<&'a na::Vector3<f64>> for PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: &na::Vector3<f64>) -> PointCharge {
        rhs.translate(&self)
    }
}

impl<'a, 'b> Add<&'a na::Vector3<f64>> for &'b PointCharge {
    type Output = PointCharge;
    fn add(self, rhs: &na::Vector3<f64>) -> PointCharge {
        rhs.translate(self)
    }
}

impl<'a> AddAssign<na::Vector3<f64>> for PointCharge {
    fn add_assign(&mut self, rhs: na::Vector3<f64>) {
        self.loc += rhs;
    }
}

impl<'a, 'b> AddAssign<&'a na::Vector3<f64>> for PointCharge {
    fn add_assign(&mut self, rhs: &na::Vector3<f64>) {
        // nalgebra does not implement AddAssign<&Vector3> for Point3,
        // thus this is done field by field
        self.loc.x = self.loc.x + rhs.x;
        self.loc.y = self.loc.y + rhs.y;
        self.loc.z = self.loc.z + rhs.z;
    }
}
