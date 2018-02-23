use web_common::*;

pub mod tile_flags {
	pub const ALLOWS_Z_MOVE: u32 = 1<<0;
	pub const BLOCKS_MOVE: u32 = 1<<1;
}

pub trait TileMap {
	type TileType;
	fn set_tile(&mut self, pos: Vec2i, value: Self::TileType);
	fn get_tile(&self, pos: Vec2i) -> Option<&Self::TileType>;

	fn get_size(&self) -> Vec2i;

	fn pos_in_bounds(&self, pos: Vec2i) -> bool {
		let sz = self.get_size();

		pos.x >= 0 && pos.x < sz.x
		&& pos.y >= 0 && pos.y < sz.y
	}

	fn wrap_position(&self, pos: Vec2i) -> Vec2i {
		let sz = self.get_size();

		Vec2i::new(
			(pos.x%sz.x + sz.x) % sz.x,
			(pos.y%sz.y + sz.y) % sz.y)
	}
}

#[derive(Clone, Debug)]
pub struct TileInfo {
	pub name: &'static str,
	pub texel_offset: Vec2i,
	pub texel_size: Vec2i,
	pub flags: u32,
}

#[derive(Debug)]
pub struct TileSet {
	tile_infos: Vec<TileInfo>,
}

pub struct FlatTileMap<T> {
	data: Vec<T>,
	size: Vec2i,
}

pub struct ItemizedTileMap<T> {
	data: Vec<(Vec2i, T)>,
	size: Vec2i,
}
// TODO: Chunked tile map


impl TileInfo {
	pub fn allows_z_move(&self) -> bool {
		self.flags & tile_flags::ALLOWS_Z_MOVE > 0
	}
}


impl TileSet {
	pub fn new(info: &[TileInfo]) -> Self {
		TileSet { tile_infos: info.into() }
	}

	pub fn get_tile_info(&self, index: usize) -> Option<&TileInfo> {
		if index == 0 {
			None
		} else {
			self.tile_infos.get(index-1)
		}
	}

	pub fn get_tile_info_by_name(&self, name: &str) -> Option<&TileInfo> {
		self.tile_infos.iter()
			.find(|ti| ti.name == name)
	}
}

impl<T> FlatTileMap<T> where T: Clone + Default {
	pub fn new(size: Vec2i) -> Self {
		FlatTileMap {
			data: vec![Default::default(); (size.x * size.y) as usize],
			size,
		}
	}

	pub fn set_tiles_from<F>(&mut self, f: F) where F: Fn(Vec2i) -> T {
		for y in 0..self.size.y {
			for x in 0..self.size.x {
				let index = x + self.size.x * y;
				self.data[index as usize] = f(Vec2i::new(x, y));
			}
		}
	}
}

impl<T> TileMap for FlatTileMap<T> {
	type TileType = T;

	fn get_size(&self) -> Vec2i { self.size }

	fn set_tile(&mut self, pos: Vec2i, value: T) {
		if !self.pos_in_bounds(pos) { return }

		let index = pos.x + self.size.x * pos.y;
		self.data[index as usize] = value;
	}

	fn get_tile(&self, pos: Vec2i) -> Option<&T> {
		if self.pos_in_bounds(pos) {
			let index = pos.x + self.size.x * pos.y;
			Some(&self.data[index as usize])
		} else {
			None
		}
	}
}


impl<T> ItemizedTileMap<T> {
	pub fn new(size: Vec2i) -> Self {
		ItemizedTileMap {
			data: Vec::new(),
			size,
		}
	}

	fn hash_pos_with(p: Vec2i, s: Vec2i) -> i32 { p.x + p.y * s.x }
	fn hash_pos(&self, p: Vec2i) -> i32 { Self::hash_pos_with(p, self.size) }
}

impl<T> TileMap for ItemizedTileMap<T> {
	type TileType = T;

	fn get_size(&self) -> Vec2i { self.size }

	fn set_tile(&mut self, pos: Vec2i, value: T) {
		if !self.pos_in_bounds(pos) { return }

		let size = self.size;
		let hash_func = |t: &(Vec2i, _)| Self::hash_pos_with(t.0, size);

		if let Ok(idx) = self.data.binary_search_by_key(&self.hash_pos(pos), &hash_func) {
			self.data[idx] = (pos, value);
		} else {
			self.data.push((pos, value));
			self.data.sort_by_key(hash_func);
		}
	}

	fn get_tile(&self, pos: Vec2i) -> Option<&T> {
		if self.pos_in_bounds(pos) {
			self.data.binary_search_by_key(&self.hash_pos(pos), |t| self.hash_pos(t.0))
				.map(|idx| &self.data[idx].1)
				.ok()
		} else {
			None
		}
	}
}
