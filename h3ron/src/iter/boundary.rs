use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

use geo_types::{Coordinate, LineString, Polygon};

use h3ron_h3_sys::{h3ToGeoBoundary, GeoBoundary};

use crate::{H3Cell, Index};

pub struct GeoBoundaryBuilder {
    geo_boundary: GeoBoundary,
}

impl GeoBoundaryBuilder {
    pub fn new() -> Self {
        let geo_boundary = unsafe {
            let mut mu = MaybeUninit::<GeoBoundary>::uninit();
            (*mu.as_mut_ptr()).numVerts = 0;
            mu.assume_init()
        };
        Self { geo_boundary }
    }

    /// iterate over the coordinates of the boundary vertices.
    ///
    /// The order of the vertices is preserved as generated by the H3 library
    pub fn iter_cell_boundary_vertices(
        &mut self,
        cell: &H3Cell,
        close_ring: bool,
    ) -> GeoBoundaryIter {
        unsafe {
            h3ToGeoBoundary(cell.h3index(), addr_of_mut!(self.geo_boundary));
        };
        GeoBoundaryIter::new(&self.geo_boundary, close_ring)
    }
}

impl Default for GeoBoundaryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GeoBoundaryIter<'b> {
    geo_boundary: &'b GeoBoundary,
    close_ring: bool,
    pos: usize,
}

impl<'b> GeoBoundaryIter<'b> {
    /// the `geo_boundary` must be initialized
    pub const fn new(geo_boundary: &'b GeoBoundary, close_ring: bool) -> Self {
        Self {
            geo_boundary,
            close_ring,
            pos: 0,
        }
    }

    /// number of vertices in the boundary
    ///
    /// Does not include the extra vertex which is returned when `close_ring` is set
    /// to true.
    pub const fn num_verts(&self) -> usize {
        self.geo_boundary.numVerts as usize
    }

    #[inline(always)]
    fn get_coordinate(&self, pos: usize) -> Coordinate<f64> {
        assert!(pos < self.num_verts());
        Coordinate::from((
            (self.geo_boundary.verts[pos].lon as f64).to_degrees(),
            (self.geo_boundary.verts[pos].lat as f64).to_degrees(),
        ))
    }
}

impl<'b> Iterator for GeoBoundaryIter<'b> {
    type Item = Coordinate<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let num_verts = self.num_verts();
        let value = if self.pos < num_verts {
            Some(self.get_coordinate(self.pos))
        } else if self.pos == num_verts && self.close_ring && num_verts != 0 {
            Some(self.get_coordinate(0))
        } else {
            None
        };
        self.pos += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.num_verts() + if self.close_ring { 1 } else { 0 };
        (len.saturating_sub(self.pos), None)
    }
}

impl<'b> From<GeoBoundaryIter<'b>> for Polygon<f64> {
    fn from(mut gb_iter: GeoBoundaryIter<'b>) -> Self {
        gb_iter.close_ring = true;
        gb_iter.pos = 0; // rewind
        let mut exterior = Vec::with_capacity(gb_iter.num_verts() + 1);
        for coord in gb_iter {
            exterior.push(coord);
        }
        Self::new(LineString::from(exterior), Vec::with_capacity(0))
    }
}
