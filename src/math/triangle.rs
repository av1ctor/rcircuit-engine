use super::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vector3,
    pub b: Vector3,
    pub c: Vector3,
}

impl Triangle {
    pub fn new(
        a: Vector3,
        b: Vector3,
        c: Vector3
    ) -> Self {
        Self {
            a,
            b,
            c,
        }
    }
}