use gltf::{Gltf, Node};
use navmesh_rs::{navmesh::NavMesh, query::NavMeshQuery};
use serde::{Serialize, Deserialize};
use three_rs::math::{
    vector3::Vector3, 
    quaternion::Quaternion, 
    path3::Path3
};

const MAX_NAVMESH_PATH_LEN: usize = 4;

#[derive(Clone)]
pub struct NodeTransform {
    pub translation: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

#[derive(Debug)]
pub struct MapGltf {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    #[serde(skip)]
    gltf: Option<MapGltf>,
    #[serde(skip)]
    nav_mesh: Option<NavMesh>,
    #[serde(skip)]
    nav_query: Option<NavMeshQuery>,
    level_bytes: Vec<u8>, 
    navmesh_bytes: Vec<u8>,
}

impl Clone for Map {
    fn clone(&self) -> Self {
        Map::new(
            &self.level_bytes, 
            &self.navmesh_bytes
        )
    }
}

impl Map {
    pub fn new(
        level_bytes: &[u8],
        navmesh_bytes: &[u8]
    ) -> Self {
        Self {
            gltf: None,
            nav_mesh: None,
            nav_query: None,
            level_bytes: level_bytes.to_vec(),
            navmesh_bytes: navmesh_bytes.to_vec(),
        }
    }

    pub fn get_gltf(
        &mut self
    ) -> &Option<MapGltf> {
        if self.gltf.is_none() {
            let gltf = Gltf::from_slice(&self.level_bytes).unwrap();
            let buffers = gltf::import_buffers(
                &gltf.document, None, gltf.blob).unwrap();
            self.gltf = Some(MapGltf{
                doc: gltf.document,
                buffers,
            });
        }

        &self.gltf
    }

    pub fn get_nav_mesh(
        &mut self
    ) -> &Option<NavMesh> {
        if self.nav_mesh.is_none() {
            self.nav_mesh = Some(
                NavMesh::from_bytes(&self.navmesh_bytes)
            );
        }

        &self.nav_mesh
    }

    pub fn get_nav_query(
        &mut self
    ) -> &Option<NavMeshQuery> {
        if self.nav_query.is_none() {
            let nav_mesh = self.get_nav_mesh().as_ref().unwrap();
            self.nav_query = Some(
                NavMeshQuery::new(nav_mesh, MAX_NAVMESH_PATH_LEN)
            );
        }

        &self.nav_query
    }

    pub fn find_nodes_by_partial_name(
        &mut self,
        node_name: &str
    ) -> Vec<Node> {
        let scene = self.get_gltf().as_ref().unwrap()
            .doc.default_scene().unwrap();
        
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
        &mut self, 
        from: &Vector3,
        to: &Vector3
    ) -> Path3 {
        Path3::new(self.get_nav_query().as_ref().unwrap()
            .find_path(from, to)
        )
    }

    pub fn move_along_surface(
        &mut self,
        from: &Vector3,
        to: &Vector3
    ) -> Option<Vector3> {
        self.get_nav_query().as_ref().unwrap()
            .move_along_surface(&from, &to)
    }

    pub fn calc_distance_to_closest_wall(
        &mut self,
        from: &Vector3,
        to: &Vector3
    ) -> Option<f32> {
        if let Some(hit) = self.get_nav_query().as_ref()
            .unwrap().raycast(from, to) {
            Some(hit.0)
        }
        else {
            None
        }
    }
}
