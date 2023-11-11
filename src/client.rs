use ambient_api::core::text::components::text;
use ambient_api::{
    camera::screen_position_to_world_ray,
    core::messages::Frame,
    element::{use_query, use_state},
    prelude::*,
};
use packages::this::{
    components::gacha_result,
    messages::{Click, GachaResult, StartGacha},
    types::{GachaKind, GachaRewardKind},
};

#[main]
pub fn main() {
    App.el().spawn_interactive();

    Frame::subscribe(|_| {
        let (delta, cur) = input::get_delta();
        if delta.mouse_buttons.contains(&MouseButton::Left) {
            let ray =
                screen_position_to_world_ray(camera::get_active().unwrap(), cur.mouse_position);
            Click::new(ray.origin, ray.dir).send_server_reliable();
        }
    });

    GachaResult::subscribe(|ctx, msg| {
       println!("Gacha result: {:?}", msg);
    });
}

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([Button::new("Start Gacha", |_| {
        StartGacha {
            kind: GachaKind::Knife,
            amount: 1,
        }
        .send_server_reliable()
    })
    .el(), GachaResultEl.el()])
    .with_padding_even(STREET)
    .with(space_between_items(), 10.)
}

#[element_component]
fn GachaResultEl(hooks: &mut Hooks) -> Element {
    let mut gachas = use_query(hooks, gacha_result());

    match gachas.last() {
        None => Text::el(""),
        Some((_, text)) => {
            Text::el(text)
        }
    }

    // let mut items = use_query(hooks, (todo_item(), todo_time()));
    // items.sort_by_key(|(_, (_, time))| *time);
    // FlowColumn::el(items.into_iter().map(|(id, (description, _))| {
    //     FlowRow::el([
    //         Button::new(COLLECTION_DELETE_ICON, move |_| {
    //             DeleteItem::new(id).send_server_reliable()
    //         })
    //             .style(ButtonStyle::Flat)
    //             .el(),
    //         Text::el(description),
    //     ])
    // }))
    //     .with(space_between_items(), 10.)
}
