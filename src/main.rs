use three_rs::math::octree::Octree;

fn _traverse(
    node: &gltf::Node<'_>,
    cb: &dyn Fn (&gltf::Node<'_>)
) {
    cb(node);
    for child in node.children() {
        _traverse(&child, cb);
    }
}

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

    println!("triangles: {}", octree.triangles.len());
}
