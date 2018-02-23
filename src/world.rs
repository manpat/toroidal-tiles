use web_common::*;
use tiles::{FlatTileMap, TileMap, TileSet, TileInfo};

pub const LAYER_FACTOR: i32 = 2;

pub struct World {
	layers: Vec<FlatTileMap<u8>>,
	pub tile_set: TileSet,

	pub player_pos: Vec2,
	pub player_layer: i32,
}

impl World {
	pub fn new(tile_set: TileSet) -> Self {
		let mut layers = Vec::new();

		for i in 0..6 {
			let world_size = Vec2i::splat(World::layer_size(i));

			layers.push(FlatTileMap::new(world_size));
		}

		let player_layer = layers.len() as i32 - 1;

		World {
			layers,
			tile_set,

			player_pos: Vec2::new(2.5, 2.5),
			player_layer,
		}
	}

	pub fn layer_size(layer: i32) -> i32 { LAYER_FACTOR.pow(layer as u32 + 2) }

	pub fn shift_layer(&mut self, dir: i32) -> bool {
		let new_layer = self.player_layer + dir;
		if new_layer < 0 || new_layer >= self.layers.len() as _ { return false }

		self.player_layer = new_layer;
		self.player_pos = self.player_pos * Vec2::splat((LAYER_FACTOR as f32).powi(dir));
		true
	}

	pub fn move_player(&mut self, dir: Vec2i) -> bool {
		self.player_pos = self.player_pos + dir.to_vec2();
		let layer_size = World::layer_size(self.player_layer) as f32;

		let mut did_warp = false;

		if self.player_pos.x < 0.0 { self.player_pos.x += layer_size; did_warp = true; }
		if self.player_pos.x >= layer_size { self.player_pos.x -= layer_size; did_warp = true; }

		if self.player_pos.y < 0.0 { self.player_pos.y += layer_size; did_warp = true; }
		if self.player_pos.y >= layer_size { self.player_pos.y -= layer_size; did_warp = true; }

		did_warp
	}

	pub fn set_tile(&mut self, pos: Vec2i, value: u8) {
		let pos = self.wrap_position(pos);
		self.layers[self.player_layer as usize].set_tile(pos, value);
	}

	pub fn get_tile(&self, pos: Vec2i) -> Option<u8> {
		let pos = self.wrap_position(pos);
		self.layers[self.player_layer as usize].get_tile(pos).cloned()
	}

	pub fn get_tile_below(&self, pos: Vec2i) -> Option<u8> {
		if self.player_layer <= 0 {
			return None
		}

		let pos = self.wrap_position(pos);
		let pos = Vec2i::new(pos.x/LAYER_FACTOR, pos.y/LAYER_FACTOR);

		let next_layer = (self.player_layer-1) as usize;
		let layer = &self.layers[next_layer];

		layer.get_tile(pos).cloned()
	}

	pub fn get_tiles_above(&self, pos: Vec2i) -> Option<[u8; (LAYER_FACTOR*LAYER_FACTOR) as usize]> {
		if self.player_layer + 1 >= self.layers.len() as _ {
			return None
		}

		let pos = self.wrap_position(pos);
		let pos = Vec2i::new(pos.x*LAYER_FACTOR, pos.y*LAYER_FACTOR);

		let layer = &self.layers[(self.player_layer+1) as usize];

		// TODO: Update this for non-2 LAYER_FACTOR
		let mut tiles = [0u8; (LAYER_FACTOR*LAYER_FACTOR) as usize];
		let poss = [
			pos + Vec2i::new(0, 0),
			pos + Vec2i::new(0, 1),
			pos + Vec2i::new(1, 1),
			pos + Vec2i::new(1, 0),
		];

		for (p, t) in poss.iter().zip(tiles.iter_mut()) {
			*t = layer.get_tile(*p).cloned()?;
		}

		Some(tiles)
	}

	pub fn get_tile_info(&self, pos: Vec2i) -> Option<&TileInfo> {
		self.get_tile(pos).iter()
			.filter_map(|&idx| self.tile_set.get_tile_info(idx as usize))
			.next()
	}
	
	pub fn wrap_position(&self, pos: Vec2i) -> Vec2i {
		self.layers[self.player_layer as usize].wrap_position(pos)
	}
}
