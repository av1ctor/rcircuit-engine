use super::{triangle::Triangle, box3::Box3};

#[derive(Clone)]
pub struct Octree {
    pub bx: Option<Box3>,
    pub bounds: Box3,
    pub children: Vec<Octree>,
    pub triangles: Vec<Triangle>,
}

impl Octree {
    pub fn new(
    ) -> Self {
        Self {
            bx: None,
            bounds: Box3::default(),
            children: vec![],
            triangles: vec![],
        }
    }

    pub fn add_triangle( 
        &mut self,
        tri: Triangle
    ) {
		self.bounds.min.x = f32::min(f32::min(f32::min(self.bounds.min.x, tri.a.x), tri.b.x), tri.c.x);
		self.bounds.min.y = f32::min(f32::min(f32::min(self.bounds.min.y, tri.a.y), tri.b.y), tri.c.y);
		self.bounds.min.z = f32::min(f32::min(f32::min(self.bounds.min.z, tri.a.z), tri.b.z), tri.c.z);
		self.bounds.max.x = f32::max(f32::max(f32::max(self.bounds.max.x, tri.a.x), tri.b.x), tri.c.x);
		self.bounds.max.y = f32::max(f32::max(f32::max(self.bounds.max.y, tri.a.y), tri.b.y), tri.c.y);
		self.bounds.max.z = f32::max(f32::max(f32::max(self.bounds.max.z, tri.a.z), tri.b.z), tri.c.z);

		self.triangles.push(tri);
	}

    fn calc_box(
        &mut self
    ) {
		let mut bx = self.bounds.clone();

		// offset small amount to account for regular grid
		bx.min.x -= 0.01;
		bx.min.y -= 0.01;
		bx.min.z -= 0.01;

        self.bx = Some(bx);
	}

    fn split( 
        &mut self,
        level: usize
    ) {
		if let Some(bx) = &self.bx {

        }
    }

    pub fn build(
        &mut self
    ) {
		self.calc_box();
		self.split(0);
	}
}