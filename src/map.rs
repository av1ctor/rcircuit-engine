use gltf::{Gltf, buffer::Data, Document, Node};
use navmesh_rs::{navmesh::NavMesh, query::NavMeshQuery};
use three_rs::math::{
    vector3::Vector3, 
    quaternion::Quaternion, path3::Path3
};

const MAX_NAVMESH_PATH_LEN: usize = 64;

#[derive(Clone)]
pub struct NodeTransform {
    pub translation: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

pub struct Map {
    pub gltf_doc: Document,
    pub buffers: Vec<Data>,
    pub nav_mesh: NavMesh,
    pub query: NavMeshQuery,
}

impl Map {
    pub fn new(
        level_bytes: &[u8],
        navmesh_bytes: &[u8]
    ) -> Self {
        let gltf = Gltf::from_slice(&level_bytes).unwrap();
        let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob).unwrap();
        let nav_mesh = NavMesh::from_bytes(navmesh_bytes);
        let query = NavMeshQuery::new(&nav_mesh, MAX_NAVMESH_PATH_LEN);
        Self {
            gltf_doc: gltf.document,
            buffers,
            nav_mesh,
            query
        }
    }

    pub fn find_nodes_by_partial_name(
        &self,
        node_name: &str
    ) -> Vec<Node> {
        let scene = self.gltf_doc.default_scene().unwrap();
        
        scene.nodes()
            .filter(|n| {
                if let Some(name) = n.name() {
                    name.starts_with(node_name)
                }
                else {
                    false
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_nodes_transform(
        nodes: Vec<Node>
    ) -> Vec<NodeTransform> {
        nodes.iter()
            .map(|n| {
                let dec = n.transform().decomposed();
                NodeTransform {
                    translation: Vector3::from_slice(&dec.0),
                    rotation: Quaternion::from_slice(&dec.1),
                    scale: Vector3::from_slice(&dec.2),
                }
                
            })
            .collect::<Vec<_>>()
    }

    pub fn find_path_to_target(
        &self, 
        from: &Vector3,
        to: &Vector3
    ) -> Path3 {
        Path3::new(self.query.find_path(from, to))
    }

    pub fn move_along_surface(
        &self,
        from: &Vector3,
        to: &Vector3
    ) -> Option<Vector3> {
        self.query.move_along_surface(&from, &to)
    }

    pub fn calc_distance_to_closest_wall(
        &self,
        from: &Vector3,
        to: &Vector3
    ) -> Option<f32> {
        if let Some(hit) = self.query.raycast(from, to) {
            Some(hit.0)
        }
        else {
            None
        }
    }
}