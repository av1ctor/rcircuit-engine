use navmesh_rs::query::NavMeshQuery;
use three_rs::math::{octree::Octree, vector3::Vector3};

use crate::map::Map;

pub mod env;
pub mod renderer;
pub mod object;
pub mod player;
pub mod enemy;
pub mod spawn_loc;
pub mod map;
pub mod game;

fn main() {
    let map = if let Ok(level_bytes) = std::fs::read("./level-0.glb") {
        if let Ok(navmesh_bytes) = std::fs::read("level-0-navmesh.bin") {
            Map::new(&level_bytes, &navmesh_bytes)
        }
        else {
            panic!("Error loading level-0-navmesh.bin");
        }
    }
    else {
        panic!("Error loading level-0.glb");
    };
    
    //let scene = gltf.default_scene().unwrap();
    //let octree = load_or_gen_octree(scene, buffers);
    //println!("triangles: {}", octree.triangles.len());

    let nav_query = NavMeshQuery::new(&map.nav_mesh);
    let path = nav_query.find_path(
        &nav_query.find_closest_point(&Vector3::new(2.0, 0.0, 0.0)), 
        &nav_query.find_closest_point(&Vector3::new(27.0, 0.0, 17.0)), 
        256
    );
    println!("{:?}", path);
}

fn _load_or_gen_octree(
    scene: gltf::Scene<'_>, 
    buffers: Vec<gltf::buffer::Data>
) -> Octree {
    let octree = if let Ok(bytes) = std::fs::read("level-0-octree.bin") {
        postcard::from_bytes(&bytes).unwrap()
    }
    else {
        let octree = Octree::from_gltf(scene, buffers);
        let octree_serialized = postcard::to_allocvec(&octree).unwrap();
        let mut file = std::fs::File::create("level-0-octree.bin").expect("Error creating level-0-octree.bin");
        std::io::Write::write_all(&mut file, &octree_serialized).expect("Error writing to level-0-octree.bin");
        octree
    };

    octree
}
