// Standard library
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

// CELL
use crate::{
    automaton::{AutomatonCell, CPUCell, GPUCell},
    universe::{CPUUniverse, GPUUniverse, Universe, UniverseDiff},
};

use super::{Coordinates2D, Neighbor2D, SCoordinates2D};

// ! Assumption : a cell in the default state whose neighborhood only consists of cells in the
// ! default state will remain in the deault state in the next generation

// ! Assumption: the chunk size is larger than the maximum one-axis manhattan distance of the
// ! automaton's neighborhood

/// InfiniteGrid2D

#[derive(Clone)]
pub struct InfiniteGrid2D<C: AutomatonCell> {
    chunks: HashMap<SCoordinates2D, Chunk<C>>,
    chunk_size_pow2: usize,
    boundary_size: usize,
    default_cell: C,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> InfiniteGrid2D<C> {
    pub fn new(chunk_size_pow2: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size_pow2,
            boundary_size: Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood()),
            default_cell: C::default(),
        }
    }

    #[inline]
    fn create_chunk(&self, coords: SCoordinates2D) -> Chunk<C> {
        // * TODO We should never create a chunk near the isize underflow/overflow boundary
        Chunk::new(coords, 1 << self.chunk_size_pow2, self.boundary_size)
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Universe for InfiniteGrid2D<C> {
    type Cell = C;
    type Coordinates = SCoordinates2D;
    type Diff = InfiniteGridDiff<C>;

    fn get(&self, coords: Self::Coordinates) -> &Self::Cell {
        let chunk_coords = coords.to_chunk_coordinates(self.chunk_size_pow2);
        match self.chunks.get(&chunk_coords) {
            Some(chunk) => chunk.get(coords.to_coordinates_in_chunk(self.chunk_size_pow2)),
            None => &self.default_cell,
        }
    }

    fn set(&mut self, coords: Self::Coordinates, val: Self::Cell) {
        let chunk_coords = coords.to_chunk_coordinates(self.chunk_size_pow2);
        let coords_in_chunk = coords.to_coordinates_in_chunk(self.chunk_size_pow2);

        // Set the cell (allocate a new chunk if necessary)
        match self.chunks.get_mut(&chunk_coords) {
            Some(chunk) => {
                chunk.set(coords_in_chunk, val);
            }
            None => {
                let mut new_chunk = self.create_chunk(chunk_coords);
                new_chunk.set(coords_in_chunk, val);
                self.chunks.insert(chunk_coords, new_chunk);
            }
        }

        // Potentially add chunks near the modified chunk's boundary
        let chunk = self.chunks.get_mut(&chunk_coords).unwrap();
        if val != C::default() {
            let mut adjacent_chunks = HashSet::new();
            chunk.get_boundary_chunks(coords_in_chunk, &mut adjacent_chunks);
            for adj_chunk_coords in adjacent_chunks {
                if !self.chunks.contains_key(&adj_chunk_coords) {
                    let new_chunk = self.create_chunk(adj_chunk_coords);
                    self.chunks.insert(adj_chunk_coords, new_chunk);
                }
            }
        }
    }

    fn neighbor(
        &self,
        coords: &Self::Coordinates,
        nbor: &<Self::Cell as AutomatonCell>::Neighbor,
    ) -> &Self::Cell {
        self.get(SCoordinates2D(
            coords.0 + nbor.0 as isize,
            coords.1 + nbor.1 as isize,
        ))
    }

    fn diff(&self, other: &Self) -> Self::Diff {
        todo!()
    }

    fn apply_diff(self, diff: &Self::Diff) -> Self {
        todo!()
    }
}

impl<C: CPUCell<Neighbor = Neighbor2D>> CPUUniverse for InfiniteGrid2D<C> {
    fn cpu_evolve_once(self) -> Self {
        todo!()
    }
}
impl<C: GPUCell<Neighbor = Neighbor2D>> GPUUniverse for InfiniteGrid2D<C> {}

/// Chunk

#[derive(Clone)]
pub struct Chunk<C: AutomatonCell> {
    data: Vec<C>,
    coordinates: SCoordinates2D,
    size: usize,
    boundary_size: usize,
    is_empty: bool,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Chunk<C> {
    pub fn get(&self, coord: Coordinates2D) -> &C {
        &self.data[coord.0 + self.size * coord.1]
    }

    fn new(coordinates: SCoordinates2D, size: usize, boundary_size: usize) -> Self {
        Self {
            data: vec![C::default(); size],
            coordinates,
            size,
            boundary_size,
            is_empty: true,
        }
    }

    fn set(&mut self, local_coords: Coordinates2D, val: C) {
        self.data[local_coords.x() + self.size * local_coords.y()] = val;
        if val != C::default() {
            self.is_empty = false;
        }
    }

    fn evolve(&mut self) -> HashSet<SCoordinates2D> {
        let mut new_data = vec![C::default(); self.size];
        self.data = new_data;
        todo!()
    }

    fn get_boundary_chunks(
        &self,
        local_coords: Coordinates2D,
        chunk_coordinates: &mut HashSet<SCoordinates2D>,
    ) {
        let left = local_coords.x() < self.boundary_size;
        let right = local_coords.x() >= self.size - self.boundary_size;
        let bottom = local_coords.y() < self.boundary_size;
        let top = local_coords.y() >= self.size - self.boundary_size;

        let x = self.coordinates.x();
        let y = self.coordinates.y();

        if left {
            chunk_coordinates.insert(SCoordinates2D(x - 1, y));
            if bottom {
                chunk_coordinates.insert(SCoordinates2D(x - 1, y - 1));
                chunk_coordinates.insert(SCoordinates2D(x, y - 1));
            } else if top {
                chunk_coordinates.insert(SCoordinates2D(x - 1, y + 1));
                chunk_coordinates.insert(SCoordinates2D(x, y + 1));
            }
        } else if right {
            chunk_coordinates.insert(SCoordinates2D(x + 1, y));
            if bottom {
                chunk_coordinates.insert(SCoordinates2D(x + 1, y - 1));
                chunk_coordinates.insert(SCoordinates2D(x, y - 1));
            } else if top {
                chunk_coordinates.insert(SCoordinates2D(x + 1, y + 1));
                chunk_coordinates.insert(SCoordinates2D(x, y + 1));
            }
        } else if bottom {
            chunk_coordinates.insert(SCoordinates2D(x, y - 1));
        } else if top {
            chunk_coordinates.insert(SCoordinates2D(x, y + 1));
        }
    }
}

/// InfiniteGridDiff

#[derive(Clone)]
pub struct InfiniteGridDiff<C: AutomatonCell> {
    _marker: PhantomData<C>,
}

impl<C: AutomatonCell> UniverseDiff for InfiniteGridDiff<C> {
    fn no_diff() -> Self {
        todo!()
    }

    fn stack(&mut self, other: &Self) {
        todo!()
    }
}
