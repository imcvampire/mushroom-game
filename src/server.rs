use std::f32::consts::PI;

use ambient_api::core::physics::components::cube_collider;
use ambient_api::core::primitives::components::cube;
use ambient_api::prelude::*;
use ambient_api::{
    rand,
    animation::{AnimationPlayerRef, PlayClipFromUrlNodeRef},
    core::{
        animation::components::apply_animation_player,
        app::components::main_scene,
        camera::{
            components::fog,
            concepts::{
                PerspectiveInfiniteReverseCamera, PerspectiveInfiniteReverseCameraOptional,
            },
        },
        hierarchy::components::parent,
        model::components::model_from_url,
        physics::components::{dynamic, physics_controlled, plane_collider},
        player::components::{is_player, user_id},
        prefab::components::prefab_from_url,
        primitives::components::quad,
        rendering::components::{
            color, fog_color, fog_density, fog_height_falloff, light_ambient, light_diffuse,
            pbr_material_from_url, sky, sun,
        },
        transform::{
            components::{local_to_world, lookat_target, rotation, scale, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    entity::{
        add_child, add_component, add_components, get_component, has_component, remove_component,
        set_component,
    },
    prelude::*,
};
use packages::{
    character_animation::components::basic_character_animations,
    character_movement::concepts::{CharacterMovement, CharacterMovementOptional},
    dead_meets_lead_content::assets as dmlc_assets,
    game_object::components::health,
    this::{
        components::{run_to,gacha_result},
        messages::{Click,StartGacha,GachaResult},
        types::{GachaKind,GachaRewardKind},
    },
    unit::components::{is_on_ground, jumping, run_direction, running, speed, vertical_velocity},
};

#[main]
async fn main() {
    PerspectiveInfiniteReverseCamera {
        optional: PerspectiveInfiniteReverseCameraOptional {
            aspect_ratio_from_window: Some(entity::resources()),
            main_scene: Some(()),
            translation: Some(vec3(1., 0., 1.) * 5.),
            ..default()
        },
        ..PerspectiveInfiniteReverseCamera::suggested()
    }
    .make()
    .with(lookat_target(), vec3(0., 0., 0.))
    // .with(fog(), ())
    .spawn();

    // ground
    let ground = Entity::new()
        .with(local_to_world(), Mat4::IDENTITY)
        .with(plane_collider(), ())
        .spawn();

    for y in 0..100 {
        for x in 0..100 {
            Entity::new()
                .with(local_to_world(), Mat4::IDENTITY)
                .with(parent(), ground)
                .with(translation(), ivec3(x, y, 0).as_vec3() - vec3(50., 50., 0.))
                .with(quad(), ())
                .with(
                    pbr_material_from_url(),
                    dmlc_assets::url("pipeline.toml/114/mat.json"),
                )
                .spawn();
        }
    }

    let trees = [
        "Swamptree1.x",
        "Swamptree2.x",
        // "Palmtree1.x",
        "Palmtree2.x",
        "Rottentree1.x",
    ];
    let foliage = [
        "Fern1.x",
        "Fern2.x",
        "FieldPlant1.x",
        "Flower1.x",
        "Forestplants1.x",
        "Forestplants2.x",
        "Leaf1.x",
        "Leaf2.x",
        "Shrub1.x",
        "Shrub2.x",
        "Shrubbery1.x",
        "Tallgrass1.x",
        "Tallgrass2.x",
        "Tallgrass3.x",
    ];
    let stones = [
        // "GravelStone1.x",
        "MudPebble1.x",
        "MudPebble2.x",
        "MudPebble3.x",
        // "Pebble4.x",
        // "Stone1.x",
        // "Stone3.x",
        // "Stone4.x",
    ];

    for i in 0..50 {
        for tree in trees {
            spawn_big_world_object(tree)
        }
    }
    for i in 0..500 {
        for tree in foliage {
            spawn_small_world_object(tree)
        }
    }
    for i in 0..10 {
        for tree in stones {
            spawn_big_world_object(tree)
        }
    }

    for i in 0..10 {
        Entity::new()
            .with(
                model_from_url(),
                packages::this::assets::url("boletus_edulis.fbx"),
            )
            .with_merge(Transformable {
                local_to_world: Default::default(),
                optional: TransformableOptional {
                    scale: Some(Vec3::ONE * 0.05),
                    translation: Some(random::<Vec2>().extend(0.) * 5.),
                    ..Default::default()
                },
            })
            .spawn();
    }

    spawn_query(user_id())
        .requires(is_player())
        .bind(move |players| {
            for (id, _) in players {
                entity::add_components(
                    id,
                    Entity::new()
                        .with(
                            prefab_from_url(),
                            packages::base_assets::assets::url("Y Bot.fbx"),
                        )
                        .with(basic_character_animations(), id)
                        .with_merge(Transformable {
                            local_to_world: Default::default(),
                            optional: TransformableOptional {
                                // translation: Some(Vec3::Z * 1.),
                                rotation: Some(Quat::IDENTITY),
                                scale: None,
                                ..Default::default()
                            },
                        })
                        .with_merge(CharacterMovement {
                            ..CharacterMovement::suggested()
                        })
                        .with(physics_controlled(), ())
                        .with(dynamic(), true)
                        .with(health(), 100.),
                );
            }
        });

    Click::subscribe(|cx, ev| {
        if let Some(hit) = physics::raycast_first(ev.orig, ev.dir) {
            println!("hit: {:?}", hit);
            let id = cx.client_entity_id().unwrap();
            if has_component(hit.entity, health()) {
                remove_component(id, run_to());
            } else {
                add_component(id, run_to(), hit.position);
            }
        }
    });

    query((run_to(), translation())).each_frame(|entities| {
        for (id, (target, pos)) in entities {
            let delta = target - pos;
            if delta.length() < 0.1 {
                remove_component(id, run_to());
                set_component(id, run_direction(), Vec2::ZERO);
            } else {
                let dir = delta.normalize();
                let rot = dir.y.atan2(dir.x);
                set_component(id, run_direction(), Vec2::X);
                set_component(id, rotation(), Quat::from_rotation_z(rot));
            }
        }
    });

    StartGacha::subscribe(|ctx, msg| {
        println!("Starting Gacha {:?}", msg);

        let possible_rewards = vec![GachaRewardKind::Normal, GachaRewardKind::Rare];
        let reward = possible_rewards.choose(&mut rand::thread_rng()).unwrap();

        GachaResult {
            kind: msg.kind,
            reward: *reward,
        }.send_client_targeted_reliable(ctx.source.client_user_id().unwrap());

        let text = match *reward {
            GachaRewardKind::None => "",
            GachaRewardKind::Normal => "Normal",
            GachaRewardKind::Rare => "Rare",
        };

        let gacha_result_entity = Entity::new().with(gacha_result(), text.to_string()).spawn();

        sleep(Duration::from_secs(10).as_secs_f32());

        entity::despawn(gacha_result_entity);
    });
}

fn spawn_small_world_object(name: &str) {
    Entity::new()
        .with(
            model_from_url(),
            dmlc_assets::url(&format!("Data/Models/Props/{name}")),
        )
        .with(
            translation(),
            random::<Vec2>().extend(0.) * 100. - vec3(50., 50., 0.),
        )
        .with(rotation(), Quat::from_rotation_z(random::<f32>() * 2. * PI))
        .with(scale(), Vec3::ONE * 0.6 + Vec3::ONE * random::<f32>() * 0.8)
        .spawn();
}

fn spawn_big_world_object(name: &str) {
    Entity::new()
        .with(
            prefab_from_url(),
            dmlc_assets::url(&format!("Data/Models/Props/{name}")),
        )
        .with(
            translation(),
            random::<Vec2>().extend(0.) * 100. - vec3(50., 50., 0.),
        )
        .with(rotation(), Quat::from_rotation_z(random::<f32>() * 2. * PI))
        .with(scale(), Vec3::ONE * 0.6 + Vec3::ONE * random::<f32>() * 0.8)
        .spawn();
}
