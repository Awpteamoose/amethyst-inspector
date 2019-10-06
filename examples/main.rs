use amethyst::{
	prelude::*,
	renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow, Transparent, SpriteRender, resources::Tint},
	utils::application_root_dir,
	window::DisplayConfig,
	core::{
		Hidden,
		HiddenPropagate,
		math::{
			Isometry, Isometry2, Isometry3, Quaternion, Translation, Translation3, UnitComplex, UnitQuaternion, Vector,
			Vector2, Vector3, Vector4,
		},
		Transform,
		Named,
	},
	ui::{UiTransform, UiText},
};

use amethyst_inspector::{inspector, InspectControl, Inspect};

type TextureList = std::collections::HashMap<String, amethyst::assets::Handle<amethyst::renderer::Texture>>;
type SpriteList = std::collections::HashMap<String, amethyst::assets::Handle<amethyst::renderer::SpriteSheet>>;

struct Example;
impl SimpleState for Example {
	fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
		let StateData { world, .. } = data;

		world.setup::<(
			Read<'_, SpriteList>,
			Read<'_, std::collections::HashMap<String, amethyst::ui::FontHandle>>,
			Read<'_, TextureList>,
		)>();

		world
			.create_entity()
			.with(amethyst::ui::UiTransform::new(
				String::default(),
				amethyst::ui::Anchor::Middle,
				amethyst::ui::Anchor::Middle,
				0., 0., 0.,
				200., 50.,
			))
			.with(amethyst::ui::UiImage::SolidColor([1., 1., 1., 1.]))
			.build();
	}
}

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
	#[inspect(with_component = "Player")]
	pub maybe_player: Option<Entity>,
}

impl Default for Player {
	fn default() -> Self {
		Self {
			direction: Vector2::zeros(),
			movement: Movement {
				speed: 10.,
				direction: Vector2::zeros(),
			},
			maybe_player: None,
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
	Transparent,
	UiTransform,
	UiText,
	SpriteRender,
	Hidden,
	HiddenPropagate,
	Tint,
];

fn main() -> amethyst::Result<()> {
	amethyst::start_logger(Default::default());
	let game_data = GameDataBuilder::default()
		.with_barrier()
		.with_bundle(amethyst::core::transform::TransformBundle::new())?
		.with_bundle(amethyst::ui::UiBundle::<amethyst::input::StringBindings>::new())?
		.with_bundle(amethyst::input::InputBundle::<amethyst::input::StringBindings>::default())?
		.with_bundle(
			RenderingBundle::<DefaultBackend>::new()
				.with_plugin(
					RenderToWindow::from_config(DisplayConfig::default())
						.with_clear([0., 0., 0., 1.]),
				)
				.with_plugin(amethyst::renderer::plugins::RenderFlat2D::default())
				.with_plugin(amethyst::ui::RenderUi::default())
				.with_plugin(amethyst_imgui::RenderImgui::<amethyst::input::StringBindings>::default()),
		)?
		.with(amethyst_inspector::InspectorHierarchy::default(), "", &[])
		.with(Inspector, "", &[])
	;

	Application::build(amethyst::utils::application_root_dir()?, Example)?.build(game_data)?.run();

	Ok(())
}
