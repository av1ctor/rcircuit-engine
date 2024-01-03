use navmesh::serdes;
use recastnavigation_sys::dtNavMesh;
use three_rs::math::octree::Octree;
mod navmesh;

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
    unsafe {
        println!("title.w: {}, title.h: {}, title.max: {}", (*nav_mesh).m_tileWidth, (*nav_mesh).m_tileHeight, (*nav_mesh).m_maxTiles);
        println!("origin: {:?}", (*nav_mesh).m_orig);
    }
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
) -> *const dtNavMesh {
    if let Ok(bytes) = std::fs::read("level-0-navmesh.bin") {
        serdes::import(bytes).unwrap()
    }
    else {
        panic!("Error loading level-0-navmesh.bin");
    }
}
 