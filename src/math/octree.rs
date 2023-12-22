use super::{triangle::Triangle, box3::Box3, vector3::Vector3};

#[derive(Clone)]
pub struct OctreeNode {
    pub bx: Box3,
    pub sub_trees: Vec<OctreeNode>,
    pub triangles: Vec<u32>,
}
pub struct Octree {
    pub root: OctreeNode,
    pub bounds: Box3,
    pub triangles_buffer: Vec<Triangle>,
}

impl Octree {
    pub fn new(
        bx: Box3
    ) -> Self {
        Self {
            root: OctreeNode::new(bx),
            bounds: Box3::default(),
            triangles_buffer: vec![],
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
        
        let index = self.triangles_buffer.len();
        self.root.add_triangle(index as u32);
        self.triangles_buffer.push(tri);
    }

    fn calc_box(
        &self
    ) -> Box3 {
        Box3::new(
            self.bounds.min.sub_scalar(0.01), 
            self.bounds.max.clone()
        )
	}

    pub fn build(
        &mut self
    ) {
        let bx = self.calc_box();

        self.root.build(bx, &self.triangles_buffer);
    }
    
}

impl OctreeNode {
    pub fn new(
        bx: Box3
    ) -> Self {
        Self {
            bx,
            sub_trees: vec![],
            triangles: vec![]
        }
    }

    pub fn add_triangle( 
        &mut self,
        index: u32
    ) {
        self.triangles.push(index);
	}

    fn split( 
        &mut self,
        level: usize,
        triangles_buffer: &Vec<Triangle>
    ) {
        if self.bx.is_empty() {
            return;
        }
        
        let mut sub_trees = vec![];
        let half_size = self.bx.max.sub(&self.bx.min).mul_scalar(0.5);

        for x in [0.0, 1.0] {
            for y in [0.0, 1.0] {
                for z in [0.0, 1.0] {
                    let v = Vector3::new(x, y, z);
                    let min = self.bx.min.add(&v.mul(&half_size));
                    let max = min.add(&half_size);
                    
                    sub_trees.push(Self::new(Box3::new(min, max)));
                }
            }    
        }

        while let Some(tri) = self.triangles.pop() {
            for sub in &mut sub_trees {
                let triangle = &triangles_buffer[tri as usize];
                if sub.bx.intersects_with_triangle(triangle) {
                    sub.triangles.push(tri);
                }
            }
        }

        while let Some(mut sub) = sub_trees.pop() {
            let len = sub.triangles.len();
            if len > 8 && level < 16 {
                sub.split(level + 1, triangles_buffer);
            }

            if len != 0 {
                self.sub_trees.push(sub);
            }
        }
    }

    fn _traverse(
        tree: &Self,
        level: usize,
        cb: &mut dyn FnMut(&Self, usize) -> ()
    ) {
        cb(tree, level);
        for sub in &tree.sub_trees {
            Self::_traverse(sub, level + 1, cb);
        }
    }

    pub fn build(
        &mut self,
        bx: Box3,
        triangles_buffer: &Vec<Triangle>
    ) {
        self.bx = bx;
        self.split(0, triangles_buffer);

        /*let mut trees = 0;
        let mut triangles = 0;
        Self::_traverse(
            &self, 
            0,
            &mut |tree, _level| -> () {
                trees += 1;
                triangles += tree.triangles.len();
            }
        );

        println!("trees: {}", trees);
        println!("triangles: {}", triangles);*/
	}
}