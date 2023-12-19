
#[derive(Clone, Debug)]
pub struct Matrix4(pub [[f32; 4]; 4]);

impl Matrix4 {
    pub fn new(
        src: &[[f32; 4]; 4]
    ) -> Self {
        Self(src.clone())
    }

    pub fn multiply(
        &self, 
        b: &Self
    ) -> Self {
        let ae = &self.0;
		let be = &b.0;

		let a11 = ae[0][0];
        let a12 = ae[1][0];
        let a13 = ae[2][0];
        let a14 = ae[3][0];
		
        let a21 = ae[0][1];
        let a22 = ae[1][1];
        let a23 = ae[2][1];
        let a24 = ae[3][1];
		
        let a31 = ae[0][2];
        let a32 = ae[1][2];
        let a33 = ae[2][2];
        let a34 = ae[3][2];
		
        let a41 = ae[0][3];
        let a42 = ae[1][3];
        let a43 = ae[2][3];
        let a44 = ae[3][3];

		let b11 = be[0][0];
        let b12 = be[1][0];
        let b13 = be[2][0];
        let b14 = be[3][0];
		
        let b21 = be[0][1];
        let b22 = be[1][1];
        let b23 = be[2][1];
        let b24 = be[3][1];
		
        let b31 = be[0][2];
        let b32 = be[1][2];
        let b33 = be[2][2];
        let b34 = be[3][2];
		
        let b41 = be[0][3];
        let b42 = be[1][3];
        let b43 = be[2][3];
        let b44 = be[3][3];

		let mut te = [[0.0f32; 4]; 4];
        
        te[0][0] = a11 * b11 + a12 * b21 + a13 * b31 + a14 * b41;
		te[0][1] = a21 * b11 + a22 * b21 + a23 * b31 + a24 * b41;
		te[0][2] = a31 * b11 + a32 * b21 + a33 * b31 + a34 * b41;
		te[0][3] = a41 * b11 + a42 * b21 + a43 * b31 + a44 * b41;
		te[1][0] = a11 * b12 + a12 * b22 + a13 * b32 + a14 * b42;
		te[1][1] = a21 * b12 + a22 * b22 + a23 * b32 + a24 * b42;
		te[1][2] = a31 * b12 + a32 * b22 + a33 * b32 + a34 * b42;
		te[1][3] = a41 * b12 + a42 * b22 + a43 * b32 + a44 * b42;
		te[2][0] = a11 * b13 + a12 * b23 + a13 * b33 + a14 * b43;
		te[2][1] = a21 * b13 + a22 * b23 + a23 * b33 + a24 * b43;
		te[2][2] = a31 * b13 + a32 * b23 + a33 * b33 + a34 * b43;
		te[2][3] = a41 * b13 + a42 * b23 + a43 * b33 + a44 * b43;
		te[3][0] = a11 * b14 + a12 * b24 + a13 * b34 + a14 * b44;
		te[3][1] = a21 * b14 + a22 * b24 + a23 * b34 + a24 * b44;
		te[3][2] = a31 * b14 + a32 * b24 + a33 * b34 + a34 * b44;
		te[3][3] = a41 * b14 + a42 * b24 + a43 * b34 + a44 * b44;

        Self(te)
    }
}