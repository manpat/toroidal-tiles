#![feature(generators, generator_trait, box_syntax)]
#![feature(associated_type_defaults)]
#![feature(inclusive_range_syntax)]
#![feature(specialization)]
#![feature(ord_max_min)]
#![feature(link_args)]
#![feature(const_fn)]

extern crate web_common;
pub use web_common::*;
use web_common::rendering::*;

mod world;
mod world_view;
mod tiles;

use world::*;
use world_view::*;

#[macro_export]
macro_rules! asset {
	($expr:expr) => {{
		include_str!(concat!("../assets/", $expr))
	}}
}

#[macro_export]
macro_rules! bin_asset {
	($expr:expr) => {{
		include_bytes!(concat!("../assets/", $expr))
	}}
}

pub fn screen_point_to_gl(screen_size: Vec2i, point: Vec2i) -> Vec2 {
	let sz = screen_size.to_vec2();
	let aspect = sz.x as f32 / sz.y as f32;

	let norm = point.to_vec2() / sz * 2.0 - Vec2::splat(1.0);
	norm * Vec2::new(aspect, -1.0)
}

use events::{Event, KeyCode};

pub const TEXELS_PER_TILE: u32 = 16;
pub const ATLAS_SIZE: u32 = 128;
pub const TEXEL_FACTOR: Vec2 = Vec2::splat(1.0 / ATLAS_SIZE as f32);

fn main() {
	std::env::set_var("RUST_BACKTRACE", "1");

	set_coro_as_main_loop(|| {
		let webgl = WebGLContext::new(false);
		webgl.set_background(Color::grey(0.2));

		let mut events = Vec::new();

		unsafe {
			events::initialise_ems_event_queue(&mut events);

			gl::Enable(gl::BLEND);
			gl::BlendEquation(gl::FUNC_ADD);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		}

		let shader = ShaderBuilder::new()
			.use_highp()
			.use_proj()
			.use_view()
			.frag_attribute("uv", "vec2")
			.uniform("color", "sampler2D")
			.output("texture2D(u_color, v_uv)")
			.finalize()
			.unwrap();

		shader.use_program();

		let tex = Texture::from_png(bin_asset!("tileset.png"));
		if tex.size != Vec2i::splat(ATLAS_SIZE as i32) {
			panic!("Tileset atlas has invalid size");
		}

		let tile_set = {
			use tiles::{TileSet, TileInfo};
			use tiles::tile_flags::*;

			let texel_size = Vec2i::splat(16);
			TileSet::new(&[
				TileInfo {
					name: "ground",
					texel_offset: Vec2i::zero(),
					texel_size,
					flags: 0,
				},

				TileInfo {
					name: "ladder",
					texel_offset: Vec2i::new(16, 0),
					texel_size,
					flags: ALLOWS_Z_MOVE,
				},

				TileInfo {
					name: "player_idle",
					texel_offset: Vec2i::new(32, 0),
					texel_size,
					flags: 0,
				},

				TileInfo {
					name: "cursor",
					texel_offset: Vec2i::new(48, 0),
					texel_size,
					flags: 0,
				},

				TileInfo {
					name: "generator",
					texel_offset: Vec2i::new(32, 0),
					texel_size,
					flags: 0,
				},

				TileInfo {
					name: "storage",
					texel_offset: Vec2i::new(32, 0),
					texel_size,
					flags: 0,
				},

				TileInfo {
					name: "pipe_base",
					texel_offset: Vec2i::new(32, 0),
					texel_size,
					flags: 0,
				},
				TileInfo {
					name: "pipe_connector",
					texel_offset: Vec2i::new(32, 0),
					texel_size,
					flags: 0,
				},
			])
		};

		let mut world_view = WorldView::new();
		let mut world = World::new(tile_set);

		let mut show_menu = false;
		let mut mouse_pos = Vec2i::zero();

		let mut ui_mesh = Mesh::new();
		let mut ui_builder = MeshBuilder::new();

		loop {
			for e in events.iter() {
				match *e {
					Event::Resize(sz) => {
						world_view.screen_size = sz;

						webgl.set_viewport(sz);

						let aspect = sz.x as f32 / sz.y as f32;
						shader.set_proj(&Mat4::scale(Vec3::new(1.0/aspect, 1.0, 1.0)));
					}

					Event::Down(pos) => {
						let Vec2{x, y} = world_view.transform_screen_coord_to_tile(pos);
						let tile_pos = Vec2i::new(x.floor() as i32, y.floor() as i32);

						if let Some(v) = world.get_tile(tile_pos) {
							world.set_tile(tile_pos, (v+1) % 3);
						}
					}

					Event::Move(pos) => {
						mouse_pos = pos; 
					}

					Event::KeyDown(k) => {
						let prev_pos = world.player_pos;

						let mut did_warp = false;
						let mut did_shift = false;

						match k {
							KeyCode::Alpha('W') => if world.move_player(Vec2i::new(0, 1)) { did_warp = true }
							KeyCode::Alpha('S') => if world.move_player(Vec2i::new(0,-1)) { did_warp = true }
							KeyCode::Alpha('D') => if world.move_player(Vec2i::new( 1, 0)) { did_warp = true }
							KeyCode::Alpha('A') => if world.move_player(Vec2i::new(-1, 0)) { did_warp = true }
	
							KeyCode::Alpha('Q') => {
								let z_move_allowed = world.get_tile_info(prev_pos.to_vec2i())
									.map(|ti| ti.allows_z_move())
									.unwrap_or(false);

								if z_move_allowed {
									did_shift |= world.shift_layer(-1)
								}
							}

							KeyCode::Alpha('E') => {
								let z_move_allowed = world.get_tile_info(prev_pos.to_vec2i())
									.map(|ti| ti.allows_z_move())
									.unwrap_or(false);

								if z_move_allowed {
									did_shift |= world.shift_layer( 1)
								}
							}
	
							KeyCode::Tab => {
								show_menu ^= true;
							}
							_ => {}
						}

						if did_warp {
							let dir = (world.player_pos - prev_pos).normalize();

							world_view.camera_pos = world_view.camera_pos 
								+ dir * Vec2::splat(World::layer_size(world.player_layer) as f32);

						} else if did_shift {
							let cam_diff = world_view.camera_pos - prev_pos.to_vec2i().to_vec2();
							world_view.camera_pos = world.player_pos.to_vec2i().to_vec2() + cam_diff;
						}
					}

					_ => {}
				}
			}

			let bg_colors: [Color; world::NUM_LAYERS as usize] = [
				Color::hsv(0.0, 0.6, 0.1),
				Color::hsv(10.0, 0.5, 0.1),
				Color::hsv(20.0, 0.3, 0.1),
				Color::grey(0.1),
				Color::hsv(90.0, 0.3, 0.1),
				Color::hsv(130.0, 0.5, 0.1),
			];

			webgl.set_background(bg_colors[world.player_layer as usize]);
			webgl.clear();

			events.clear();

			tex.bind_to_slot(0);

			shader.set_view(&world_view.get_view_matrix());
			shader.set_uniform_i32("u_color", 0);

			world_view.draw(&world);

			{	let cursor = world.tile_set.get_tile_info_by_name("cursor").unwrap();

				let Vec2{x, y} = world_view.transform_screen_coord_to_tile(mouse_pos);
				let mouse_tile = Vec2::new(x.floor(), y.floor());

				ui_builder.draw_tiled(cursor.texel_offset, cursor.texel_size, mouse_tile);
	
				ui_builder.upload_to(&mut ui_mesh);
				ui_builder.clear();

				ui_mesh.bind();
				ui_mesh.draw(gl::TRIANGLES);
			}

			yield;
		}
	});
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vert2D(Vec2, Vec2);

impl Vertex for Vert2D {
	fn get_layout() -> VertexLayout {
		VertexLayout::new::<Self>()
			.add_binding(0, 2, 0)
			.add_binding(1, 2, 8)
	}
}


pub trait MeshBuilderTileExtension {
	fn draw_tiled_rotated(&mut self, texel_offset: Vec2i, texel_size: Vec2i, pos: Vec2, rot: u32);
	fn draw_tiled(&mut self, texel_offset: Vec2i, texel_size: Vec2i, pos: Vec2);
}

impl MeshBuilderTileExtension for MeshBuilder<Vert2D> {
	fn draw_tiled_rotated(&mut self, texel_offset: Vec2i, texel_size: Vec2i, pos: Vec2, rot: u32) {
		let base_uv = texel_offset.to_vec2() * TEXEL_FACTOR;
		let uv_size = texel_size.to_vec2() * TEXEL_FACTOR;
		let size = texel_size.to_vec2() / Vec2::splat(TEXELS_PER_TILE as f32);

		let rot = rot as usize;

		let uvs = [
			base_uv + Vec2::new(0.01, 0.98) * uv_size,
			base_uv + Vec2::new(0.01, 0.01) * uv_size,
			base_uv + Vec2::new(0.98, 0.01) * uv_size,
			base_uv + Vec2::new(0.98, 0.98) * uv_size,
		];

		self.add_quad(&[
			Vert2D(Vec2::new(0.0, 0.0)*size + pos, uvs[(rot+0)%4]),
			Vert2D(Vec2::new(0.0, 1.0)*size + pos, uvs[(rot+1)%4]),
			Vert2D(Vec2::new(1.0, 1.0)*size + pos, uvs[(rot+2)%4]),
			Vert2D(Vec2::new(1.0, 0.0)*size + pos, uvs[(rot+3)%4]),
		]);
	}

	fn draw_tiled(&mut self, texel_offset: Vec2i, texel_size: Vec2i, pos: Vec2) {
		self.draw_tiled_rotated(texel_offset, texel_size, pos, 0);
	}
}