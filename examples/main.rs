use amethyst::{
	prelude::*,
	renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow},
	utils::application_root_dir,
	window::DisplayConfig,
	core::{
		math::{
			Isometry, Isometry2, Isometry3, Quaternion, Translation, Translation3, UnitComplex, UnitQuaternion, Vector,
			Vector2, Vector3, Vector4,
		},
		Transform,
		Named,
	},
};

use amethyst_imgui::RenderImgui;
use amethyst_inspector::{inspector, InspectControl, Inspect};

#[derive(Default, Clone, Copy)]
pub struct DemoSystem;
impl<'s> amethyst::ecs::System<'s> for DemoSystem {
	type SystemData = ();
	fn run(&mut self, _: Self::SystemData) {
		amethyst_imgui::with(|ui| {
			ui.show_demo_window(&mut true);
		});
	}
}

struct Example;
impl SimpleState for Example {}

#[derive(Clone, InspectControl)]
pub struct Movement {
	#[inspect(null_to = 10., speed = 0.1)]
	pub speed: f32,
	pub direction: Vector2<f32>,
}

#[derive(Clone, Inspect)]
pub struct Player {
	pub movement: Movement,
	pub direction: Vector2<f32>,
}

impl Default for Player {
	fn default() -> Self {
		Self {
			direction: Vector2::zeros(),
			movement: Movement {
				speed: 10.,
				direction: Vector2::zeros(),
			},
		}
	}
}

impl Component for Player {
	type Storage = DenseVecStorage<Self>;
}

inspector![
	Named,
	Transform,
	Player,
];

fn main() -> amethyst::Result<()> {
	amethyst::start_logger(Default::default());
	let game_data = GameDataBuilder::default()
		.with_barrier()
		.with(DemoSystem::default(), "imgui_use", &[])
		.with_bundle(amethyst::core::transform::TransformBundle::new())?
		.with_bundle(
			RenderingBundle::<DefaultBackend>::new()
				.with_plugin(
					RenderToWindow::from_config(DisplayConfig::default())
						.with_clear([0., 0., 0., 1.]),
				)
				.with_plugin(RenderImgui::default()),
		)?
		.with(amethyst_inspector::InspectorHierarchy::default(), "", &[])
		.with(Inspector, "", &[])
	;

	Application::build("/", Example)?.build(game_data)?.run();

	Ok(())
}
