// Ported by Robert Borghese (SomeRanDev)
//
// Tested on Godot 4.3 dev6
//
// Ported from axilirate's gist:
// https://gist.github.com/axilirate/96a3e77d597c2527582dbc79aecbab70
//
// Which was a Godot 4 port of Oussama BOUKHELF's Godot 3 Trail System:
// https://github.com/OBKF/Godot-Trail-System
// https://github.com/OBKF/Godot-Trail-System/blob/master/addons/Trail/trail_3d.gd
// https://github.com/OBKF/Godot-Trail-System/blob/master/LICENSE

use godot::classes::{
	Curve, IMeshInstance3D, ImmediateMesh, MeshInstance3D,
	mesh::PrimitiveType
};

use godot::builtin::math::FloatExt;

use godot::prelude::*;

// ---

/// Used internally to configure `Trail3D.render_geometry` behavior.
enum PointesRenderType {
	Points,
	InternalPointsAndTempSegmentAndC,
}

/// Used internally for `Trail3D.render_geometry` to optimize iterating through `Point`s.
enum PointsIterator<'a> {
	Points(usize, std::slice::Iter<'a, Point>),
	Chained(usize, std::iter::Chain<std::iter::Chain<std::slice::Iter<'a, Point>, std::slice::Iter<'a, Point>>, std::iter::Once<&'a Point>>),
}

impl<'a> PointsIterator<'a> {
	fn len(&self) -> usize {
		match self {
			PointsIterator::Points(len, _) |
			PointsIterator::Chained(len, _) => *len,
		}
	}
}

impl<'a> std::iter::Iterator for PointsIterator<'a> {
	type Item = &'a Point;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			PointsIterator::Points(_, p) => p.next(),
			PointsIterator::Chained(_, c) => c.next()
		}
	}
}

/// Used internally to track the trail's points.
#[derive(Default, Copy, Clone)]
struct Point {
	id: i64,
	transform: Transform3D,
	age: f64,
}

impl Point {
	/// Generates a new point.
	/// The `id` should be a value unique to other points in the trail so
	/// this `Point` can be found and removed from any `Vec` it's contained in.
	pub fn new(id: i64, transform: Transform3D, age: f64) -> Self {
		Self { id, transform, age }
	}

	/// Returns `true` if its life is complete.
	/// If this occurs, the callee should properly dispose of this `Point`.
	pub fn update(&mut self, delta: f64) -> bool {
		self.age -= delta;
		self.age <= 0.0
	}
}

/// Input for `Trail3D` `alignment`
#[derive(GodotConvert, Var, Debug, Export)]
#[godot(via = i32)]
enum AlignmentType {
	View,
	Normal,
	Object,
}

/// Input for `Trail3D` `axe`
#[derive(GodotConvert, Var, Debug, Export)]
#[godot(via = i32)]
enum Axis {
	X,
	Y,
	Z,
}

/// Generates a 3D trail that follows its parent node.
/// This node should always have a parent.
#[derive(GodotClass)]
#[class(init, base = MeshInstance3D)]
pub struct Trail3D {
	base: Base<MeshInstance3D>,

	#[init(val = ImmediateMesh::new_gd())]
	immediate_mesh: Gd<ImmediateMesh>,

	points: Vec<Point>,
	always_update: bool,

	_target: Option<Gd<Node3D>>,
	_a: Option<Point>,
	_b: Option<Point>,
	_c: Option<Point>,
	_temp_segment: Vec<Point>,
	_points: Vec<Point>,

	_points_last_id: i64,

	#[export]
	#[init(val = true)]
	emit: bool,

	#[export]
	#[init(val = 0.1)]
	distance: f32,

	#[export(range = (0.0, 99999.0))]
	#[init(val = 20)]
	segments: u32,

	#[export]
	#[init(val = 0.5)]
	lifetime: f64,

	#[export(range = (0.0, 99999.0))]
	#[init(val = 0.5)]
	base_width: f32,

	#[export]
	tiled_texture: bool,

	#[export]
	tiling: i32,

	#[export]
	width_curve: Option<Gd<Curve>>,

	#[export(range = (0.0, 3.0))]
	smoothing_iterations: i32,

	#[export(range = (0.0, 0.5))]
	#[init(val = 0.25)]
	smoothing_ratio: f32,

	#[export(enum = (VIEW = 1, NORMAL = 2, OBJECT = 3))]
	#[init(val = AlignmentType::View)]
	alignment: AlignmentType,

	#[export(enum = (X = 1, Y = 2, Z = 3))]
	#[init(val = Axis::Y)]
	axe: Axis,

	#[export]
	#[init(val = Color::from_rgba(1.0, 1.0, 1.0, 1.0))]
	color_start: Color,

	#[export]
	#[init(val = Color::from_rgba(1.0, 1.0, 1.0, 0.0))]
	color_end: Color,

	#[export]
	color_curve: Option<Gd<Curve>>,
}

impl Trail3D {
	fn make_point(&mut self, transform: Transform3D, lifetime: f64) -> Point {
		let result = Point::new(self._points_last_id, transform, lifetime);
		self._points_last_id += 1;
		result
	}

	/// MUST run ```self._points_last_id += 1;``` manually after calling this.
	fn make_point_immutable(&self, transform: Transform3D, lifetime: f64) -> Point {
		let result = Point::new(self._points_last_id, transform, lifetime);
		result
	}

	fn remove_point(&mut self, id: i64) {
		if let Some(index) = self._points.iter().position(|p| p.id == id) {
			self._points.remove(index);
		}
	}

	fn prepare_geometry(&self, point_prev: Point, point: Point, half_width: real, factor: real) -> (Vector3, Vector3) {
		let mut normal = Vector3::default();

		match self.alignment {
			AlignmentType::View => {
				let camera = if let Some(viewport) = self.base().get_viewport() {
					viewport.get_camera_3d()
				} else {
					godot_error!("Trail3D: Could not get viewport.");
					None
				};
				if let Some(camera) = camera {
					let cam_pos = camera.get_global_transform().origin;
					let path_direction = (point.transform.origin - point_prev.transform.origin).normalized();
					normal = (cam_pos - (point.transform.origin + point_prev.transform.origin) / 2.0).cross(path_direction).normalized();
				} else {
					godot_error!("Trail3D: There is no camera in the scene.");
				}
			},
			AlignmentType::Normal => {
				let basis = point.transform.basis;
				normal = match self.axe {
					Axis::X => basis.col_a().normalized(),
					Axis::Y => basis.col_b().normalized(),
					Axis::Z => basis.col_c().normalized()
				}
			},
			AlignmentType::Object => {
				if let Some(target) = &self._target {
					let basis = target.get_transform().basis;
					normal = match self.axe {
						Axis::X => basis.col_a().normalized(),
						Axis::Y => basis.col_b().normalized(),
						Axis::Z => basis.col_c().normalized()
					}
				} else {
					godot_error!("Trail3D: No parent found for trail.");
				}
			}
		}

		let mut width = half_width;
		if let Some(width_curve) = &self.width_curve {
			width = half_width * width_curve.sample(factor);
		}

		let normal_times_width = normal * width;
		let p1 = point.transform.origin - normal_times_width;
		let p2 = point.transform.origin + normal_times_width;
		return (p1, p2);
	}

	fn render_realtime(&mut self) {
		self.render_geometry(PointesRenderType::InternalPointsAndTempSegmentAndC);
	}

	/// Renders the provided points.
	///
	/// Originally this was given an array of points to use, but now it uses a
	/// custom-made enum to handle special iterator cases to avoid copying data.
	fn render_geometry<'a>(&mut self, render_type: PointesRenderType) {
		let mut points_iter = match render_type {
			PointesRenderType::Points => PointsIterator::Points(self.points.len(), self.points.iter()),
			PointesRenderType::InternalPointsAndTempSegmentAndC => {
				// Ensure `self._c` exists. It should, but let's just be sure.
				assert!(self._c.is_some());
				PointsIterator::Chained(
					self._points.len() + self._temp_segment.len() + 1,
					self._points.iter()
						.chain(self._temp_segment.iter())
						.chain(std::iter::once(self._c.as_ref().unwrap()))
				)
			}
		};

		let points_count = points_iter.len();
		if points_count < 2 {
			return;
		}

		let first = points_iter.next().unwrap();
		let second = points_iter.next().unwrap();

		// Comment from the original GDScript version:
		// # The following section is a hack to make orientation "View" work.
		// # However, it may cause an artifact at the end of the trail;
		// # you can use transparency in the gradient to hide it for now.
		let _d = first.transform.origin - second.transform.origin;
		let mut _t = first.transform;
		_t.origin = _t.origin + _d;

		let half_width = self.base_width / 2.0;
		let mut u = 0.0;

		self.immediate_mesh.clear_surfaces();

		// surface_begin must be called this way since the normal method is unimplemented.
		// See https://github.com/godot-rust/gdext/issues/156
		self.immediate_mesh.call("surface_begin", &[PrimitiveType::TRIANGLE_STRIP.to_variant(), Variant::nil().to()]);

		let mut index = 0;
		let new_front_point = self.make_point_immutable(_t, first.age);
		self._points_last_id += 1; // Must run here since called `self.make_point_immutable`.

		// Store reference to "previous" point instead of accessing with [i-1]
		// Since the original code started at 1 and placed the new Point at the front of the array,
		// we simply start at 0 without adding the new Point then reference the new Point in "previous".
		let mut previous = &new_front_point;

		// Add first and second back into the iterator...
		let points_iter = std::iter::once(first).chain(std::iter::once(second)).chain(points_iter);
		for item in points_iter {
			let mut factor = index as f32 / points_count as f32;
			let vertices = self.prepare_geometry(previous.clone(), item.clone(), half_width, 1.0-factor);
			if self.tiled_texture {
				if self.tiling > 0 {
					factor *= self.tiling as f32;
				} else {
					let travel = (previous.transform.origin - item.transform.origin).length();
					u += travel / self.base_width;
					factor = u;
				}
			}

			previous = item;

			let mut r = 1.0 - factor;
			if let Some(color_curve) = &self.color_curve {
				r = color_curve.sample(r);
			}
			let color = self.color_start.lerp(self.color_end, r as f64);

			let m = &mut self.immediate_mesh;
			m.surface_set_color(color);
			m.surface_set_uv(Vector2::new(factor, 0.0));
			m.surface_add_vertex(vertices.0);
			m.surface_set_uv(Vector2::new(factor, 1.0));
			m.surface_add_vertex(vertices.1);

			index += 1;
		}

		self.immediate_mesh.surface_end();
	}

	/// This function is only called when self._a, self._b, and self._c are guarenteed to be Some.
	fn update_points(&mut self) -> Option<()> {
		assert!(self._a.is_some() && self._b.is_some() && self._c.is_some());

		let delta = self.base().get_process_delta_time();

		if self._a?.update(delta) {
			self.remove_point(self._a?.id);
		}
		if self._b?.update(delta) {
			self.remove_point(self._b?.id);
		}
		if self._c?.update(delta) {
			self.remove_point(self._c?.id);
		}

		let mut ids_to_be_removed = vec![];
		for point in &mut self._points {
			if point.update(delta) {
				ids_to_be_removed.push(point.id);
			}
		}
		for id in ids_to_be_removed {
			self.remove_point(id);
		}

		// Seemingly arbitrary values from GDScript version.
		// Maybe this could be configured better?
		let size_multiplier: usize = match self.smoothing_iterations {
			0 => 1,
			1 => 2,
			2 => 4,
			3 => 6,
			_ => 1
		};

		let max_points_count: usize = (self.segments as usize) * size_multiplier;
		if self._points.len() > max_points_count {
			// How GDScript version truncated the front...
			// self._points.reverse();
			// self._points.truncate(max_points_count as usize);
			// self._points.reverse();

			self._points.drain(0..(self._points.len() - max_points_count));
		}

		Some(())
	}

	fn chaikin(&mut self, a: Point, b: Point, c: Point) -> Vec<Point> {
		if self.smoothing_iterations == 0 {
			return vec![b];
		}

		let mut out = vec![];
		let x: f32 = self.smoothing_ratio;

		// Pre-calculate some parameters to improve performance
		let xi: f32 = 1.0 - x;
		let xpa: f32 = (x * x) - (2.0 * x) + 1.0;
		let xpb: f32 = ((-x) * x) + (2.0 * x);
		// transforms
		let a1_t: Transform3D = a.transform.interpolate_with(&b.transform, xi);
		let b1_t: Transform3D = b.transform.interpolate_with(&c.transform, x);
		// ages
		let a1_a = a.age.lerp(b.age, xi as f64);
		let b1_a = b.age.lerp(c.age, x as f64);

		if self.smoothing_iterations == 1 {
			out = vec![
				self.make_point(a1_t, a1_a),
				self.make_point(b1_t, b1_a)
			];
		} else {
			// transforms
			let a2_t  = a.transform.interpolate_with(&b.transform, xpa);
			let b2_t  = b.transform.interpolate_with(&c.transform, xpb);
			let a11_t = a1_t.interpolate_with(&b1_t, x);
			let b11_t = a1_t.interpolate_with(&b1_t, xi);
			// ages
			let a2_a  = a.age.lerp(b.age, xpa as f64);
			let b2_a  = b.age.lerp(c.age, xpb as f64);
			let a11_a = a1_a.lerp(b1_a, x as f64);
			let b11_a = a1_a.lerp(b1_a, xi as f64);

			if self.smoothing_iterations == 2 {
				out.append(&mut vec![
					self.make_point(a2_t, a2_a),
					self.make_point(a11_t, a11_a),
					self.make_point(b11_t, b11_a),
					self.make_point(b2_t, b2_a),
				]);
			} else if self.smoothing_iterations == 3 {
				// transforms
				let a12_t  = a1_t.interpolate_with(&b1_t, xpb);
				let b12_t  = a1_t.interpolate_with(&b1_t, xpa);
				let a121_t = a11_t.interpolate_with(&a2_t, x);
				let b121_t = b11_t.interpolate_with(&b2_t, x);
				// ages
				let a12_a  = a1_a.lerp(b1_a, xpb as f64);
				let b12_a  = a1_a.lerp(b1_a, xpa as f64);
				let a121_a = a11_a.lerp(a2_a, x as f64);
				let b121_a = b11_a.lerp(b2_a, x as f64);
				out.append(&mut vec![
					self.make_point(a2_t, a2_a),
					self.make_point(a121_t, a121_a),
					self.make_point(a12_t, a12_a),
					self.make_point(b12_t, b12_a),
					self.make_point(b121_t, b121_a),
					self.make_point(b2_t, b2_a),
				]);
			}
		}

		return out;
	}

	fn update_emit(&mut self, delta: f64) -> Option<()> {
		let _transform = self._target.as_ref()?.get_global_transform();

		let point = self.make_point(_transform, self.lifetime);
		if self._a.is_none() {
			self._a = point.into();
			return Some(());
		} else if self._b.is_none() {
			if self._a?.update(delta) {
				self.remove_point(self._a?.id);
			}
			self._b = point.into();
			return Some(());
		}

		if self._b?.transform.origin.distance_squared_to(_transform.origin) >= (self.distance*self.distance) {
			self._a = self._b;
			self._b = point.into();
			self._points.append(&mut self._temp_segment);
		}

		self._c = point.into();

		self.update_points()?;
		self._temp_segment = self.chaikin(self._a?, self._b?, self._c?);
		self.render_realtime();

		Some(())
	}
}

#[godot_api]
impl Trail3D {
	#[func]
	fn add_trail_point(&mut self, transform: Transform3D) {
		let point = self.make_point(transform, self.lifetime);
		self.points.push(point);
	}

	#[func]
	fn clear_trail_points(&mut self) {
		self.points.clear();
	}

	/// Not really sure what this function does since it isn't used internally,
	/// so I've exposed it for use in GDScript.
	#[func]
	fn smooth(&mut self) {
		if self.points.len() < 3 {
			return;
		}

		let mut output = vec![self.points[0]];
		for i in 1..(self.points.len() - 1) {
			output.append(&mut self.chaikin(self.points[i - 1], self.points[i], self.points[i + 1]));
		}

		// Guarenteed to have `last()` point because `len()` checked at start.
		output.push(self.points.last().unwrap().clone());

		self.points = output;
	}

	/// Not really sure what this function does since it isn't used internally,
	/// so I've exposed it for use in GDScript.
	#[func]
	fn render(&mut self, update: bool) {
		if update {
			self.always_update = true;
		} else {
			self.render_geometry(PointesRenderType::Points);
		}
	}
}

#[godot_api]
impl IMeshInstance3D for Trail3D {
	fn ready(&mut self) {
		self.base_mut().set_global_transform(Transform3D::IDENTITY);

		let mesh = self.immediate_mesh.clone();
		self.base_mut().set_mesh(&mesh);

		self._target = self.base().get_parent_node_3d();
		if self._target.is_none() {
			godot_error!("Trail3D: Trail should have Node3D parent.");
		}

		self.base_mut().set_as_top_level(true);
	}

	fn process(&mut self, delta: f64) {
		if self.emit {
			if self.update_emit(delta).is_none() {
				godot_error!("Trail3D: update_emit failure");
			}
		} else if self.always_update {
			// Required for self.alignment == View, so the perspective is updated even when self.emit == false
			self.render_geometry(PointesRenderType::Points);
		}
	}
}
