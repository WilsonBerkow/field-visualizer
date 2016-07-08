use num::Zero;
use na::{ Point3, Vector3, Norm };

use std::ops::Add;

pub trait VectorField3 {
    fn field_data_at(&self, p: &Point3<f64>) -> FieldData;
}

pub struct FieldData {
    pub force_vec: Vector3<f64>, // direction and unscaled strength of field
    pub force_mag: f64, // cached norm of force_vec, updated manually
    pub potential: f64, // potential before scaling relative to field as a whole
}

impl FieldData {
    pub fn new(force_vec: Vector3<f64>, mag: f64, pot: f64) -> FieldData {
        FieldData {
            force_vec: force_vec,
            force_mag: mag,
            potential: pot,
        }
    }

    pub fn update_norm(&mut self) {
        self.force_mag = self.force_vec.norm();
    }
}

impl Zero for FieldData {
    fn zero() -> FieldData { FieldData::new(Zero::zero(), 0.0, 0.0) }
    fn is_zero(&self) -> bool {
        self.force_vec.is_zero()
            && self.potential.is_zero()
    }

}

impl Add<FieldData> for FieldData {
    type Output = FieldData;
    fn add(self, right: FieldData) -> FieldData {
        FieldData::new(
            self.force_vec + right.force_vec,
            self.force_mag, // not updated
            self.potential + right.potential
        )
    }
}
