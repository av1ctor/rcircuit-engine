use super::vector3::Vector3;

#[derive(Clone)]
pub struct Box3 {
    pub min: Vector3,
    pub max: Vector3,
}

impl Default for Box3 {
    fn default() -> Self {
        Self { 
            min: Vector3::new(f32::MIN, f32::MIN, f32::MIN), 
            max: Vector3::new(f32::MAX, f32::MAX, f32::MAX)
        }
    }
}

impl Box3 {
    pub fn new(
        min: Vector3,
        max: Vector3
    ) -> Self {
        Self {
            min,
            max
        }
    }
}