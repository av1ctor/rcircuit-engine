use std::cell::RefCell;
use math::{octree::Octree, matrix4::Matrix4};
use crate::math::{vector3::Vector3, triangle::Triangle};

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

fn traverse_meshes(
    node: &gltf::Node<'_>,
    cb: &dyn Fn (&gltf::Mesh<'_>)
) {
    if let Some(mesh) = node.mesh() {
        cb(&mesh);
        for child in node.children() {
            traverse_meshes(&child, cb);
        }
    }
}

fn main() {
    let (gltf, buffers, _) = gltf::import("./level.glb").unwrap();
    let scene = gltf.default_scene().unwrap();
    let octree = Box::new(RefCell::new(Octree::new()));
    let world_matrix: Box<RefCell<Option<Matrix4>>> = Box::default();

    for node in scene.nodes() {
        // for every node on scene..
        traverse_meshes(
            &node, 
            &|mesh| {
                let octree = &mut octree.as_ref().borrow_mut();

                let matrix = Matrix4::new(&node.transform().matrix());
                let mut wm = world_matrix.as_ref().borrow_mut();
                *wm = if let Some(m) = wm.as_ref() {
                    Some(m.multiply(&matrix))
                }
                else {
                        Some(matrix)
                };

                // for every primtive in a mesh..
                for primitive in mesh.primitives() {
                    match primitive.mode() {
                        // made of triangles?
                        gltf::mesh::Mode::Triangles => {
                            println!("Triangles primitive:");
                            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                            if let Some(mut iter) = reader.read_positions() {
                                // for each triange in a primitive..
                                for i in 0..iter.clone().count()/3 {
                                    println!("Triangle #{}:", i);
                                    let mut a = Vector3::from_slice(&iter.next().unwrap());
                                    let mut b = Vector3::from_slice(&iter.next().unwrap());
                                    let mut c = Vector3::from_slice(&iter.next().unwrap());

                                    if let Some(m) = wm.as_ref() {
                                        a.apply_matrix4(m);
                                        b.apply_matrix4(m);
                                        c.apply_matrix4(m);
                                    }

                                    //println!("v1:(x:{}, y:{}, z:{}), v2:(x:{}, y:{}, z:{}), v3:(x:{}, y:{}, z:{})", a.x, a.y, a.z, b.x, b.y, b.z, c.x, c.y, c.z);

                                    octree.add_triangle(Triangle::new(a, b, c));
                                }
                            }
                        },
                        _ => {
                            println!("Unsupported primitive: {:?}", primitive.mode());
                        }
                    }
                }
            }
        );
    }

    println!("{:?}", world_matrix.as_ref().borrow());

    octree.as_ref().borrow_mut().build();
}
