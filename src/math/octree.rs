use super::{triangle::Triangle, box3::Box3, vector3::Vector3};

#[derive(Clone)]
pub struct Octree {
    pub bx: Option<Box3>,
    pub bounds: Box3,
    pub sub_trees: Vec<Octree>,
    pub triangles: Vec<Triangle>,
}

impl Octree {
    pub fn new(
        bx: Option<Box3>
    ) -> Self {
        Self {
            bx,
            bounds: Box3::default(),
            sub_trees: vec![],
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
        self.bx = Some(
            Box3::new(
                self.bounds.min.sub_scalar(0.01), 
                self.bounds.max.clone()
            )
        );
	}

    fn split( 
        &mut self,
        level: usize
    ) {
		if let Some(bx) = &self.bx {
            let mut sub_trees = vec![];
            let half_size = bx.max.sub(&bx.min).mul_scalar(0.5);

            for x in [0.0, 1.0] {
                for y in [0.0, 1.0] {
                    for z in [0.0, 1.0] {
                        let v = Vector3::new(x, y, z);
                        let min = bx.min.add(&v.mul(&half_size));
                        let max = min.add(&half_size);
                        
                        sub_trees.push(Octree::new(Some(Box3::new(min, max))));
                    }
                }    
            }

            while let Some(tri) = self.triangles.pop() {
                for sub in &mut sub_trees {
                    if let Some(bx) = &sub.bx {
                        if bx.intersects_with_triangle(&tri) {
                            sub.triangles.push(tri);
                        }
                    }
                }
            }

            for sub in &mut sub_trees {
                let len = sub.triangles.len();
                if len > 8 && level < 16 {
                    sub.split(level + 1);
                }

                if len != 0 {
                    self.sub_trees.push(sub.to_owned());
                }
            }
        }
    }

    pub fn build(
        &mut self
    ) {
		self.calc_box();
		self.split(0);
	}
}