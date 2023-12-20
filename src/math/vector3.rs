use super::matrix4::Matrix4;

#[derive(Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(
        x: f32,
        y: f32,
        z: f32
    ) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    pub fn from_slice(
        src: &[f32; 3]
    ) -> Self {
        Self {
            x: src[0],
            y: src[1],
            z: src[2],
        }
    }

    pub fn apply_matrix4( 
        &mut self,
        m: &Matrix4
    ) {

		let x = self.x; 
        let y = self.y;
        let z = self.z;
		let e = &m.0;

		let w = 1.0 / (e[0][3] * x + e[1][3] * y + e[2][3] * z + e[3][3]);

		self.x = (e[0][0] * x + e[1][0] * y + e[2][0] * z + e[3][0]) * w;
		self.y = (e[0][1] * x + e[1][1] * y + e[2][1] * z + e[3][1]) * w;
		self.z = (e[0][2] * x + e[1][2] * y + e[2][2] * z + e[3][2]) * w;
    }

}