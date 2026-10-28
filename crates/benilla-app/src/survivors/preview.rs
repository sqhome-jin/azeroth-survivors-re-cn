//! The launcher's hero preview: the live hero is moved onto its own render layer and filmed by a
//! second camera into an image, so the orbiting battleground behind the menu never shows it.

use bevy::app::Propagate;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use super::Phase;
use super::hero::Hero;

/// Clear of every booth layer in `portrait`.
const PREVIEW_LAYER: usize = 40;
const WIDTH: u32 = 440;
const HEIGHT: u32 = 712;

#[derive(Resource)]
pub(super) struct HeroPreview {
    image: Handle<Image>,
    camera: Entity,
}

/// The launcher node that shows the preview image.
#[derive(Component)]
pub(super) struct HeroPreviewView;

/// Carries the hero's layer down to every model part under it.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(bevy::app::HierarchyPropagatePlugin::<RenderLayers>::new(
        PostUpdate,
    ))
    .configure_sets(
        PostUpdate,
        bevy::app::PropagateSet::<RenderLayers>::default()
            .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
    );
}

pub(super) fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(crate::portrait::new_target_image_sized(WIDTH, HEIGHT));
    let camera = commands
        .spawn((
            crate::portrait::booth_view_shape(),
            Camera {
                order: -60,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            RenderTarget::Image(image.clone().into()),
            benilla_world::ffx_glow::FfxGlow::BOOTH,
            Projection::Perspective(PerspectiveProjection {
                fov: 0.6,
                near: 0.05,
                far: 60.0,
                ..default()
            }),
            RenderLayers::layer(PREVIEW_LAYER),
            Transform::default(),
        ))
        .id();
    commands.insert_resource(HeroPreview { image, camera });
}

/// The hero lives on the preview layer while the menu is up, and on the world's layer in play.
pub(super) fn assign_layer(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    heroes: Query<(Entity, Option<&Propagate<RenderLayers>>), With<Hero>>,
) {
    let want = if *phase.get() == Phase::Menu {
        RenderLayers::layer(PREVIEW_LAYER)
    } else {
        RenderLayers::layer(0)
    };
    for (hero, have) in &heroes {
        if have.is_none_or(|h| h.0 != want) {
            commands
                .entity(hero)
                .insert((Propagate(want.clone()), want.clone()));
        }
    }
}

/// Films the hero from the front while the menu is up.
pub(super) fn frame(
    phase: Res<State<Phase>>,
    preview: Option<Res<HeroPreview>>,
    heroes: Query<&Transform, (With<Hero>, Without<Camera>)>,
    mut cameras: Query<(&mut Camera, &mut Transform), Without<Hero>>,
) {
    let Some(preview) = preview else { return };
    let Ok((mut camera, mut at)) = cameras.get_mut(preview.camera) else {
        return;
    };
    let hero = heroes.single().ok();
    camera.is_active = *phase.get() == Phase::Menu && hero.is_some();
    if let Some(hero) = hero {
        let focus = hero.translation + Vec3::Y * 1.0;
        let eye = hero.translation + hero.forward().as_vec3() * 3.7 + Vec3::Y * 1.3;
        *at = Transform::from_translation(eye).looking_at(focus, Vec3::Y);
    }
}

/// Points freshly built launcher views at the preview image.
pub(super) fn fill_view(
    preview: Option<Res<HeroPreview>>,
    mut views: Query<&mut ImageNode, Added<HeroPreviewView>>,
) {
    let Some(preview) = preview else { return };
    for mut node in &mut views {
        node.image = preview.image.clone();
        node.color = Color::WHITE;
    }
}
