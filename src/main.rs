use std::{fs::File, io::Write};

use math::octree::Octree;
use postcard::to_allocvec;

mod math;

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
    let (gltf, buffers, _) = gltf::import("./level-0.glb").unwrap();
    let scene = gltf.default_scene().unwrap();

    let octree = Octree::from_gltf(scene, buffers);
    let octree_serialized = to_allocvec(&octree).unwrap();
    let mut file = File::create("level-0-octree.bin").expect("Error creating level-0-octree.bin");
    file.write_all(&octree_serialized).expect("Error writing to level-0-octree.bin");
}
