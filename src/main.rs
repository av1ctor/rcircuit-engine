use std::cell::RefCell;
use gltf::mesh::Mode;
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
    world_matrix: Option<&Matrix4>,
    cb: &dyn Fn (&gltf::Mesh<'_>, &Matrix4)
) {
    if let Some(mesh) = node.mesh() {
        let matrix = Matrix4::new(&node.transform().matrix());
        let world_matrix = if let Some(m) = world_matrix {
            m.multiply(&matrix)
        }
        else {
            matrix
        };
        cb(&mesh, &world_matrix);
        for child in node.children() {
            traverse_meshes(&child, Some(&world_matrix), cb);
        }
    }
    else {
        for child in node.children() {
            traverse_meshes(&child, world_matrix, cb);
        }
    }
}

fn main() {
    let (gltf, buffers, _) = gltf::import("./level.glb").unwrap();
    let scene = gltf.default_scene().unwrap();
    let octree = Box::new(RefCell::new(Octree::new()));

    for node in scene.nodes() {
        // for every node on scene..
        traverse_meshes(
            &node,
            None, 
            &|mesh, world_matrix| {
                let octree = &mut octree.as_ref().borrow_mut();

                // for every primitive in a mesh..
                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                            
                    if let Some(iter) = reader.read_positions() {
                        let vertices = iter.collect::<Vec<_>>();
                        let indices: Vec<u32> = reader.read_indices()
                            .map(|ind| ind.into_u32().collect())
                            .unwrap_or((0..vertices.len() as u32).collect());

                        let mut triangles = vec![];
                
                        match primitive.mode() {
                            Mode::Triangles => {
                                for i in (0..indices.len()).step_by(3) {
                                    triangles.push([
                                        vertices[indices[i] as usize],
                                        vertices[indices[i + 1] as usize],
                                        vertices[indices[i + 2] as usize],
                                    ]);
                                }
                            }
                            Mode::TriangleStrip => {
                                for i in 0..(indices.len() - 2) {
                                    triangles.push([
                                        vertices[indices[i] as usize + i % 2],
                                        vertices[indices[i + 1 - i % 2] as usize],
                                        vertices[indices[i + 2] as usize],
                                    ]);
                                }
                            }
                            Mode::TriangleFan => {
                                for i in 1..(indices.len() - 1) {
                                    triangles.push([
                                        vertices[indices[0] as usize],
                                        vertices[indices[i] as usize],
                                        vertices[indices[i + 1] as usize],
                                    ]);
                                }
                            }
                            _ => panic!("Unsupported mode"),
                        }

                        for tri in triangles {
                            //println!("Triangle:");
                            let mut a = Vector3::from_slice(&tri[0]);
                            let mut b = Vector3::from_slice(&tri[1]);
                            let mut c = Vector3::from_slice(&tri[2]);

                            a.apply_matrix4(world_matrix);
                            b.apply_matrix4(world_matrix);
                            c.apply_matrix4(world_matrix);

                            //println!("v1:(x:{}, y:{}, z:{}), v2:(x:{}, y:{}, z:{}), v3:(x:{}, y:{}, z:{})", a.x, a.y, a.z, b.x, b.y, b.z, c.x, c.y, c.z);

                            octree.add_triangle(Triangle::new(a, b, c));
                        }
                    }
                }
            }
        );
    }

    octree.as_ref().borrow_mut().build();
}
