// Standard library
use std::{
    cell::{Ref, RefCell},
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
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> InfiniteGrid2D<C> {
    pub fn new(chunk_size_pow2: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size_pow2,
            boundary_size: Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood()),
        }
    }

    #[inline]
    fn create_chunk(&self, coords: SCoordinates2D) -> Chunk<C> {
        // TODO We should never create a chunk near the isize underflow/overflow boundary
        Chunk::new(coords, 1 << self.chunk_size_pow2, self.boundary_size)
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Universe for InfiniteGrid2D<C> {
    type Cell = C;
    type Coordinates = SCoordinates2D;
    type Diff = InfiniteGridDiff<C>;

    fn get(&self, coords: Self::Coordinates) -> Self::Cell {
        let chunk_coords = coords.to_chunk_coordinates(self.chunk_size_pow2);
        match self.chunks.get(&chunk_coords) {
            Some(chunk) => chunk.get(coords.to_coordinates_in_chunk(self.chunk_size_pow2)),
            None => C::default(),
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
            chunk.get_adjacent_chunks(coords_in_chunk, &mut adjacent_chunks);
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
        coords: Self::Coordinates,
        nbor: <Self::Cell as AutomatonCell>::Neighbor,
    ) -> Self::Cell {
        self.get(SCoordinates2D(
            coords.0 + nbor.0 as isize,
            coords.1 + nbor.1 as isize,
        ))
    }

    fn diff(&self, _other: &Self) -> Self::Diff {
        todo!()
    }

    fn apply_diff(self, _diff: &Self::Diff) -> Self {
        todo!()
    }
}

impl<C: CPUCell<Neighbor = Neighbor2D>> CPUUniverse for InfiniteGrid2D<C> {
    fn cpu_evolve_once(mut self) -> Self {
        let mut all_chunks = HashSet::new();
        for (_coords, chunk) in self.chunks.iter() {
            // Update the chunk and collect set of adjacent chunks that need to be added to the universe
            for adjacent_chunk_coords in chunk.evolve(&self) {
                all_chunks.insert(adjacent_chunk_coords);
            }
        }

        // Add all collected adjacent chunks to the universe
        for chunk_coords in all_chunks {
            if !self.chunks.contains_key(&chunk_coords) {
                let new_chunk = self.create_chunk(chunk_coords);
                self.chunks.insert(chunk_coords, new_chunk);
            }
        }

        // Return the updated universe
        self
    }
}

impl<C: GPUCell<Neighbor = Neighbor2D>> GPUUniverse for InfiniteGrid2D<C> {}

/// Chunk

#[derive(Clone)]
pub struct Chunk<C: AutomatonCell> {
    inner: RefCell<ChunkInner<C>>,
    coordinates: SCoordinates2D,
    size: usize,
    boundary_size: usize,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Chunk<C> {
    pub fn get(&self, coord: Coordinates2D) -> C {
        self.inner.borrow().data[coord.0 + self.size * coord.1]
    }

    #[inline]
    pub fn iter(&self) -> ChunkIterator<C> {
        ChunkIterator::new(self)
    }

    fn new(coordinates: SCoordinates2D, size: usize, boundary_size: usize) -> Self {
        Self {
            inner: RefCell::new(ChunkInner::new(size)),
            coordinates,
            size,
            boundary_size,
        }
    }

    fn set(&mut self, local_coords: Coordinates2D, val: C) {
        let mut inner = self.inner.borrow_mut();
        inner.data[local_coords.x() + self.size * local_coords.y()] = val;
        if val != C::default() {
            inner.is_empty = false;
        }
    }

    fn get_adjacent_chunks(
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

impl<C: CPUCell<Neighbor = Neighbor2D>> Chunk<C> {
    fn evolve(&self, grid: &InfiniteGrid2D<C>) -> HashSet<SCoordinates2D> {
        let world_coords = self.coordinates.to_universe_coordinates(self.size);
        let default_cell = C::default();

        let mut new_data = Vec::with_capacity(self.size * self.size);
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (0usize, usize::MAX, 0usize, usize::MAX);
        let mut is_empty = true;

        // Update each cell in the chunk
        for line in self.iter() {
            for (coords, cell) in line {
                // Compute cell's world coordinates and update it
                let x = coords.x();
                let y = coords.y();
                let cell_world_coords =
                    SCoordinates2D(world_coords.x() + x as isize, world_coords.y() + y as isize);
                let new_cell = cell.update(grid, cell_world_coords);

                if new_cell != default_cell {
                    // Update min/max coordinates of updated cells
                    if x < min_x {
                        min_x = x;
                    } else if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    } else if y > max_y {
                        max_y = y;
                    }

                    // Mark the chunk non-empty
                    is_empty = false;
                }

                // Append cell to new data vector
                new_data.push(new_cell);
            }
        }

        // Compute the set of adjacent chunks that the universe might need to create
        let mut adjacent_chunks = HashSet::new();
        if !is_empty {
            self.get_adjacent_chunks(Coordinates2D(min_x, min_y), &mut adjacent_chunks);
            self.get_adjacent_chunks(Coordinates2D(max_x, max_y), &mut adjacent_chunks);
        }

        // * Modify interior cell and return
        let mut inner = self.inner.borrow_mut();
        inner.data = new_data;
        inner.is_empty = is_empty;
        adjacent_chunks
    }
}

/// ChunkInner

#[derive(Clone)]
struct ChunkInner<C: AutomatonCell> {
    data: Vec<C>,
    is_empty: bool,
}

impl<C: AutomatonCell> ChunkInner<C> {
    fn new(size: usize) -> Self {
        Self {
            data: vec![C::default(); size * size],
            is_empty: true,
        }
    }
}

/// ChunkIterator

pub struct ChunkIterator<'a, C: AutomatonCell> {
    chunk: &'a Chunk<C>,
    line_idx: usize,
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> ChunkIterator<'a, C> {
    fn new(chunk: &'a Chunk<C>) -> Self {
        Self { chunk, line_idx: 0 }
    }
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Iterator for ChunkIterator<'a, C> {
    type Item = ChunkLineIterator<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line_idx < self.chunk.size {
            let col_iterator = ChunkLineIterator::new(self.chunk, self.line_idx);
            self.line_idx += 1;
            Some(col_iterator)
        } else {
            None
        }
    }
}

/// ChunkLineIterator

pub struct ChunkLineIterator<'a, C: AutomatonCell> {
    chunk: Ref<'a, ChunkInner<C>>,
    size: usize,
    coords: Coordinates2D,
    idx: usize,
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> ChunkLineIterator<'a, C> {
    fn new(chunk: &'a Chunk<C>, line_idx: usize) -> Self {
        Self {
            chunk: chunk.inner.borrow(),
            size: chunk.size,
            coords: Coordinates2D(0, line_idx),
            idx: line_idx * chunk.size,
        }
    }
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Iterator for ChunkLineIterator<'a, C> {
    type Item = (Coordinates2D, C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords.x() < self.size {
            let ret_coords = self.coords;
            let cell = self.chunk.data[self.idx];
            self.coords.0 += 1;
            self.idx += 1;
            Some((ret_coords, cell))
        } else {
            None
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

    fn stack(&mut self, _other: &Self) {
        todo!()
    }
}
