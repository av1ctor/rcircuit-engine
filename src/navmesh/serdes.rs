use std::{os::raw::c_int, mem::size_of, ptr::{null_mut, copy_nonoverlapping, addr_of_mut}};
use serde::{Deserialize, Serialize};
use recastnavigation_sys::{dtNavMeshParams, dtTileRef, dtNavMesh, dtAlloc, dtAllocHint_DT_ALLOC_PERM, dtTileFlags_DT_TILE_FREE_DATA, dtAllocNavMesh};

const NAVMESHSET_MAGIC: c_int = ('M' as c_int) << 24 | ('S' as c_int) << 16 | ('E' as c_int) << 8 | ('T' as c_int); //'MSET'
const NAVMESHSET_VERSION: c_int = 1;

#[repr(C)]
#[derive(Serialize, Deserialize)]
struct RecastHeader {
    magic: c_int,
    version: c_int,
    num_tiles: c_int,
}

#[repr(C)]
#[derive(Serialize, Deserialize)]
struct NavMeshTileHeader {
    tile_ref: dtTileRef,
    data_size: c_int
}

pub fn import(
    buf: Vec<u8>
) -> Result<*const dtNavMesh, String> {

    let mut i = 0;
    let recast_header: RecastHeader = bincode::deserialize(&buf[i..i+size_of::<RecastHeader>()]).unwrap();
    i += size_of::<RecastHeader>();

    if recast_header.magic != NAVMESHSET_MAGIC {
        return Err("Invalid magic number".to_string());
    }

    if recast_header.version != NAVMESHSET_VERSION {
        return Err("Unsupported version".to_string());
    }

    unsafe { 
        let mut params = dtNavMeshParams {
            orig: [0.0f32; 3],
            tileWidth: 0.0,
            tileHeight: 0.0,
            maxTiles: 0,
            maxPolys: 0,
        };
        copy_nonoverlapping(&buf[i] as *const u8, addr_of_mut!(params) as _, size_of::<dtNavMeshParams>());
        i += size_of::<dtNavMeshParams>();
        
        let nav_mesh = dtAllocNavMesh();
        nav_mesh.as_mut().unwrap().init(&params as *const dtNavMeshParams);

        for _ in 0..recast_header.num_tiles {
            let tile_header: NavMeshTileHeader = bincode::deserialize(&buf[i..i+size_of::<NavMeshTileHeader>()]).unwrap();
            i += size_of::<NavMeshTileHeader>();

            if tile_header.tile_ref == 0 || tile_header.data_size == 0 {
                break;
            }

            let data = dtAlloc(tile_header.data_size as usize, dtAllocHint_DT_ALLOC_PERM) as *mut u8;
            if data.is_null() {
                break;
            }

            data.copy_from(buf.as_ptr().add(i) as _, tile_header.data_size as usize);
            i += tile_header.data_size as usize;

            nav_mesh.as_mut().unwrap().addTile(data, tile_header.data_size, dtTileFlags_DT_TILE_FREE_DATA, tile_header.tile_ref, null_mut());
        }

        Ok(nav_mesh)
    }
}