// Standard library
use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
};

// Local
use crate::{
    automaton::{AutomatonCell, CPUCell, GPUCell},
    universe::{CPUUniverse, GPUUniverse, Universe},
};

use super::{Coordinates2D, Neighbor2D, SCoordinates2D};

// Assumption : a cell in the default state whose neighborhood only consists of cells in the
//              default state will remain in the default state in the next generation

/// InfiniteGrid2D

#[derive(Clone)]
pub struct InfiniteGrid2D<C: AutomatonCell> {
    chunks: HashMap<SCoordinates2D, Chunk<C>>,
    chunk_size_pow2: usize,
    boundary_size: usize,
    gc_countdown: usize,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> InfiniteGrid2D<C> {
    pub fn new(chunk_size_pow2: usize) -> Self {
        let boundary_size = Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood());

        // Equivalent to (2 * boundary) > 2^chunk_size_pow2
        if (boundary_size << 1) > (1 << chunk_size_pow2) {
            panic!(ERR_CHUNK_TOO_SMALL);
        }

        Self {
            chunks: HashMap::new(),
            chunk_size_pow2,
            boundary_size,
            gc_countdown: GC_RATE,
        }
    }

    pub fn free_useless_chunks(&mut self) {
        // Look for chunks that can be freed
        let mut to_free = Vec::new();
        for (coords, chunk) in self.chunks.iter() {
            if chunk.is_safe_for_deletion(&self.chunks) {
                to_free.push(*coords);
            }
        }

        // Free all chunks that can be freed
        for coords in to_free {
            self.free_chunk(coords);
        }
    }

    #[inline]
    fn create_chunk(&self, coords: SCoordinates2D) -> Option<Chunk<C>> {
        // TODO We should never create a chunk near the isize underflow/overflow boundary
        Some(Chunk::new(coords, self.chunk_size_pow2, self.boundary_size))
    }

    #[inline]
    fn free_chunk(&mut self, coords: SCoordinates2D) {
        self.chunks.remove(&coords);
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Universe for InfiniteGrid2D<C> {
    type Cell = C;
    type Coordinates = SCoordinates2D;

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
                if let Some(mut new_chunk) = self.create_chunk(chunk_coords) {
                    new_chunk.set(coords_in_chunk, val);
                    self.chunks.insert(chunk_coords, new_chunk);
                }
            }
        }

        // Potentially add chunks near the modified chunk's boundary
        if val != C::default() {
            let chunk = self.chunks.get_mut(&chunk_coords).unwrap();
            let mut adjacent_chunks = HashSet::new();
            chunk.get_adjacent_chunks(coords_in_chunk, &mut adjacent_chunks);
            for adj_chunk_coords in adjacent_chunks {
                if !self.chunks.contains_key(&adj_chunk_coords) {
                    if let Some(new_chunk) = self.create_chunk(adj_chunk_coords) {
                        self.chunks.insert(adj_chunk_coords, new_chunk);
                    }
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
            coords.x() + isize::from(nbor.x()),
            coords.y() + isize::from(nbor.y()),
        ))
    }
}

impl<C: CPUCell<Neighbor = Neighbor2D>> CPUUniverse for InfiniteGrid2D<C> {
    fn cpu_evolve_once(mut self) -> Self {
        let mut all_adjacent_chunks = HashSet::new();
        for (_coords, chunk) in self.chunks.iter() {
            // Ask each chunk to compute its next generation and collect set of adjacent
            // chunks that need to be added to the universe
            for adjacent_chunk_coords in chunk.compute_next_gen(&self) {
                all_adjacent_chunks.insert(adjacent_chunk_coords);
            }
        }

        // Actually update each chunk
        for (_coords, chunk) in self.chunks.iter() {
            chunk.swap_next_gen();
        }

        // Add all collected adjacent chunks to the universe
        for chunk_coords in all_adjacent_chunks {
            if !self.chunks.contains_key(&chunk_coords) {
                if let Some(new_chunk) = self.create_chunk(chunk_coords) {
                    self.chunks.insert(chunk_coords, new_chunk);
                }
            }
        }

        // Trigger garbage collection procedure at a fixed rate
        self.gc_countdown -= 1;
        if self.gc_countdown == 0 {
            self.free_useless_chunks();
            self.gc_countdown = GC_RATE;
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
    size_pow2: usize,
    boundary_size: usize,
    inner_swap: RefCell<Option<ChunkInner<C>>>,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Chunk<C> {
    pub fn get(&self, coord: Coordinates2D) -> C {
        self.inner.borrow().data[coord.0 + (1 << self.size_pow2) * coord.1]
    }

    #[inline]
    pub fn iter(&self) -> ChunkIterator<C> {
        ChunkIterator::new(self)
    }

    fn new(coordinates: SCoordinates2D, size_pow2: usize, boundary_size: usize) -> Self {
        Self {
            inner: RefCell::new(ChunkInner::new(size_pow2)),
            coordinates,
            size_pow2,
            boundary_size,
            inner_swap: RefCell::new(None),
        }
    }

    fn set(&mut self, local_coords: Coordinates2D, val: C) {
        let mut inner = self.inner.borrow_mut();
        inner.data[local_coords.x() + (1 << self.size_pow2) * local_coords.y()] = val;
        if val != C::default() {
            inner.is_empty = false;
        }
    }

    fn is_safe_for_deletion(&self, chunks: &HashMap<SCoordinates2D, Chunk<C>>) -> bool {
        let inner = self.inner.borrow();

        // A chunk is safe for deletion if it's empty and all surrounding chunks are also empty
        if inner.is_empty {
            let x = self.coordinates.x();
            let y = self.coordinates.y();

            // Check that all surrounding chunks are empty
            for rel_coords in &NEIGHBORS {
                let nbor_coords = SCoordinates2D(x + rel_coords.x(), y + rel_coords.y());
                if let Some(nbor_chunk) = chunks.get(&nbor_coords) {
                    if !nbor_chunk.inner.borrow().is_empty {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn get_adjacent_chunks(
        &self,
        local_coords: Coordinates2D,
        chunk_coordinates: &mut HashSet<SCoordinates2D>,
    ) {
        let b = (1 << self.size_pow2) - self.boundary_size;
        let left = local_coords.x() < self.boundary_size;
        let right = local_coords.x() >= b;
        let bottom = local_coords.y() < self.boundary_size;
        let top = local_coords.y() >= b;

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
    fn compute_next_gen(&self, grid: &InfiniteGrid2D<C>) -> HashSet<SCoordinates2D> {
        let world_coords = self.coordinates.to_universe_coordinates(self.size_pow2);
        let default_cell = C::default();

        let size = 1 << self.size_pow2;
        let mut data = Vec::with_capacity(size * size);
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (usize::MAX, 0usize, usize::MAX, 0usize);
        let mut is_empty = true;

        // Update each cell in the chunk
        for line in self.iter() {
            for (coords, cell) in line {
                // Compute cell's world coordinates and update it
                let (x, y) = (coords.x(), coords.y());
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
                data.push(new_cell);
            }
        }

        // Compute the set of adjacent chunks that the universe might need to create
        let mut adjacent_chunks = HashSet::new();
        if !is_empty {
            self.get_adjacent_chunks(Coordinates2D(min_x, min_y), &mut adjacent_chunks);
            self.get_adjacent_chunks(Coordinates2D(max_x, max_y), &mut adjacent_chunks);
        }

        // Store new data in the swap and return
        *self.inner_swap.borrow_mut() = Some(ChunkInner { data, is_empty });
        adjacent_chunks
    }

    fn swap_next_gen(&self) {
        let swap = self.inner_swap.replace(None).expect(ERR_SWAP_EMPTY);
        *self.inner.borrow_mut() = swap;
    }
}

/// ChunkInner

#[derive(Clone)]
struct ChunkInner<C: AutomatonCell> {
    data: Vec<C>,
    is_empty: bool,
}

impl<C: AutomatonCell> ChunkInner<C> {
    fn new(size_pow2: usize) -> Self {
        let size = 1 << size_pow2;
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
        if self.line_idx < (1 << self.chunk.size_pow2) {
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
        let line_size = 1 << chunk.size_pow2;
        Self {
            chunk: chunk.inner.borrow(),
            size: line_size,
            coords: Coordinates2D(0, line_idx),
            idx: line_idx * line_size,
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

const GC_RATE: usize = 100;
const NEIGHBORS: [SCoordinates2D; 8] = [
    SCoordinates2D(0, -1),
    SCoordinates2D(1, -1),
    SCoordinates2D(1, 0),
    SCoordinates2D(1, 1),
    SCoordinates2D(0, 1),
    SCoordinates2D(-1, 1),
    SCoordinates2D(-1, 0),
    SCoordinates2D(-1, -1),
];

const ERR_CHUNK_TOO_SMALL: &str =
    "The boundary size must be at least twice as big as the chunk size.";
const ERR_SWAP_EMPTY: &str = "Tried to swap generation without computing a new one first.";

#[cfg(test)]
mod tests {
    use super::{CPUUniverse, InfiniteGrid2D};
    use crate::{automaton::game_of_life, universe::grid2d::SCoordinates2D};

    #[test]
    fn cpu_evolution() {
        // Create LWSS
        let base_coords = SCoordinates2D(0, 0);
        let mut grid = InfiniteGrid2D::new(3);
        game_of_life::create_lwss(&mut grid, base_coords);
        assert!(game_of_life::check_lwss(&grid, base_coords, 0));

        // Start LWSS
        for n in 1..100 {
            grid = grid.cpu_evolve_once();
            println!("Testing {:?}", n);
            assert!(game_of_life::check_lwss(&grid, base_coords, n));
        }
    }
}
