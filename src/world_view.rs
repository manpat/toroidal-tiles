use web_common::*;
use web_common::rendering::*;

use MeshBuilderTileExtension;

use tiles::TileInfo;
use world::World;

pub struct WorldView {
	mesh: Mesh,
	builder: MeshBuilder<::Vert2D>,

	pub screen_size: Vec2i,
	pub camera_zoom: f32,
	pub camera_pos: Vec2,
}

impl WorldView {
	pub fn new() -> Self {
		WorldView {
			mesh: Mesh::new(),
			builder: MeshBuilder::new(),

			screen_size: Vec2i::splat(1),
			camera_zoom: 6.0,
			camera_pos: Vec2::splat(2.5),
		}
	}

	pub fn get_view_matrix(&self) -> Mat4 {
		Mat4::scale(Vec3::splat(1.0/self.camera_zoom))
			* Mat4::translate(-self.camera_pos.extend(0.0))
	}

	pub fn transform_screen_coord_to_tile(&self, point: Vec2i) -> Vec2 {
		let pos = :: screen_point_to_gl(self.screen_size, point);
		pos * Vec2::splat(self.camera_zoom) + self.camera_pos
	}

	pub fn draw(&mut self, world: &World) {
		self.camera_pos = (1.0/60.0).ease_linear(self.camera_pos, world.player_pos);

		let aspect = self.screen_size.x as f32 / self.screen_size.y as f32;
		let extent = Vec2::new(self.camera_zoom*aspect + 1.0, self.camera_zoom + 1.0);

		let bottom = (self.camera_pos - extent).to_vec2i();
		let top = (self.camera_pos + extent).to_vec2i();

		let player_info = world.tile_set.get_tile_info_by_name("player_idle")
			.expect("missing player sprite");
		
		let tile_set_lookup = |idx| world.tile_set.get_tile_info(idx as usize);

		for y in bottom.y..top.y {
			for x in bottom.x..top.x {
				let pos = Vec2i::new(x, y);

				if let Some(Some(info)) = world.get_tile(pos).map(&tile_set_lookup) {
					self.draw_tile(info, pos);
				}

				if world.wrap_position(pos) == world.player_pos.to_vec2i() {
					self.draw_tile(player_info, pos);
				}
			}
		}

		self.builder.upload_to(&mut self.mesh);
		self.builder.clear();

		self.mesh.bind();
		self.mesh.draw(gl::TRIANGLES);
	}

	pub fn draw_tile_rotated(&mut self, tile_info: &TileInfo, pos: Vec2i, rot: u32) {
		self.builder.draw_tiled_rotated(tile_info.texel_offset, tile_info.texel_size, pos.to_vec2(), rot);
	}

	pub fn draw_tile(&mut self, tile_info: &TileInfo, pos: Vec2i) {
		self.draw_tile_rotated(tile_info, pos, 0);
	}
}
