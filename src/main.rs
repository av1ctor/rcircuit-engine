use navmesh_rs::{navmesh::NavMesh, query::NavMeshQuery};
use three_rs::math::{octree::Octree, vector3::Vector3};

fn main() {
    let (gltf, buffers) = if let Ok(bytes) = std::fs::read("./level-0.glb") {
        let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
        let buffers = gltf::import_buffers(&gltf.document, None, gltf.blob).unwrap();
        (gltf.document, buffers)
    }
    else {
        panic!("Error loading level-0.glb");
    };
    
    let scene = gltf.default_scene().unwrap();

    let octree = load_or_gen_octree(scene, buffers);
    println!("triangles: {}", octree.triangles.len());

    let nav_mesh = load_navmesh();
    let nav_query = NavMeshQuery::new(&nav_mesh);
    let path = nav_query.find_path(
        &nav_query.find_closest_point(&Vector3::new(2.0, 0.0, 0.0)), 
        &nav_query.find_closest_point(&Vector3::new(27.0, 0.0, 17.0)), 
        256
    );
    println!("{:?}", path);
}

fn load_or_gen_octree(
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

fn load_navmesh(
) -> NavMesh {
    if let Ok(bytes) = std::fs::read("level-0-navmesh.bin") {
        NavMesh::from_array(bytes)
    }
    else {
        panic!("Error loading level-0-navmesh.bin");
    }
}
 