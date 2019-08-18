use amethyst::{
	prelude::*,
	renderer::{bundle::RenderingBundle, types::DefaultBackend, RenderToWindow, Transparent, SpriteRender},
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

use amethyst_imgui::RenderImgui;
use amethyst_inspector::{inspector, InspectControl, Inspect};

type TextureList = std::collections::HashMap<String, amethyst::assets::Handle<amethyst::renderer::Texture>>;
type SpriteList = std::collections::HashMap<String, amethyst::assets::Handle<amethyst::renderer::SpriteSheet>>;

struct Example;
impl SimpleState for Example {
	fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
		let StateData { mut world, .. } = data;

		world.setup::<(
			Read<'_, SpriteList>,
			Read<'_, std::collections::HashMap<String, amethyst::ui::FontHandle>>,
			Read<'_, TextureList>,
		)>();

		let loader = world.read_resource::<amethyst::assets::Loader>();
		// TODO: man it became hard to load texture from raw pixel data
		let white = loader.load(
			"white.jpg",
			amethyst::renderer::ImageFormat::default(),
			(),
			&world.read_resource(),
		);

		let mut textures = world.write_resource::<TextureList>();
		textures.insert("white 1px".to_owned(), white.clone());

		let mut sprites = world.write_resource::<SpriteList>();
		sprites.insert(
			"white 1px".to_owned(),
			world.read_resource::<amethyst::assets::Loader>().load_from_data(
				amethyst::renderer::SpriteSheet { texture: white.clone(), sprites: vec![amethyst::renderer::sprite::Sprite::from_pixel_values(1, 1, 1, 1, 0, 0, [0., 0.], false, false)] },
				(),
				&world.read_resource::<amethyst::assets::AssetStorage<amethyst::renderer::SpriteSheet>>(),
			),
		);
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
	Transparent,
	UiTransform,
	UiText,
	SpriteRender,
	Hidden,
	HiddenPropagate,
];

fn main() -> amethyst::Result<()> {
	amethyst::start_logger(Default::default());
	let game_data = GameDataBuilder::default()
		.with_barrier()
		.with_bundle(amethyst::core::transform::TransformBundle::new())?
		.with_bundle(amethyst::ui::UiBundle::<amethyst::input::StringBindings>::new())?
		.with_bundle(
			RenderingBundle::<DefaultBackend>::new()
				.with_plugin(
					RenderToWindow::from_config(DisplayConfig::default())
						.with_clear([0., 0., 0., 1.]),
				)
				.with_plugin(amethyst::renderer::plugins::RenderFlat2D::default())
				.with_plugin(amethyst::ui::RenderUi::default())
				.with_plugin(RenderImgui::default()),
		)?
		.with(amethyst_inspector::InspectorHierarchy::default(), "", &[])
		.with(Inspector, "", &[])
	;

	Application::build(amethyst::utils::application_root_dir()?, Example)?.build(game_data)?.run();

	Ok(())
}
