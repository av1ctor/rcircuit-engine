pub struct Map {
}

impl Map {
    pub fn new(
        _level_bytes: &[u8],
        _navmesh_bytes: &[u8]
    ) -> Self {
        Self {}
    }
}

impl Clone for Map {
    fn clone(
        &self
    ) -> Self {
        Self {
        }
    }
}

 