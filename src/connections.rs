use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inputs>();
        app.register_type::<Outputs>();
        app.add_systems(Update, connect);
    }
}

// they mirro each other
// use codes for input types (0 = color, 1 = ...)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Inputs(Vec<(usize, i8, i8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Outputs(Vec<(usize, i8, i8)>);

//operations as marker components
#[derive(Component)]
struct Add; //"inputA" + "inputB" (if an input is block, block output)
#[derive(Component)]
struct Mult;
#[derive(Component)]
struct Get; //takes a vector to "input" and a num index to "index"

// a Ready component for entities
// query all with no inputs and set them to ready
// then loop until all are ready

fn connect(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Pos), With<Visible>>,
    index_query: Query<&Index>,
    mut inputs_query: Query<&mut Inputs>,
    mut outputs_query: Query<&mut Outputs>,
    cursor: Res<CursorInfo>,
    entity_indices: Res<EntityIndices>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_released(MouseButton::Left) {
        let mut source_entity: Option<Entity> = None;
        let mut sink_entity: Option<Entity> = None;
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.value.xy()) < r.value { source_entity = Some(e) };
            if cursor.f.distance(p.value.xy()) < r.value { sink_entity = Some(e) };
        }

        if let Some(src) = source_entity {
            if let Some(snk) = sink_entity {
                let src_index = index_query.get(src).unwrap().0;
                let snk_index = index_query.get(snk).unwrap().0;
                // source has outputs (we push to its outputs vector)
                if outputs_query.contains(src) {
                    if let Ok(mut outputs) = outputs_query.get_mut(src) {
                        outputs.0.push((src_index, 0, 0));
                    }
                }
                else {
                    commands.entity(src).insert(Outputs(vec![(src_index, 0, 0)]));
                }
                if inputs_query.contains(snk) {
                    if let Ok(mut inputs) = inputs_query.get_mut(snk) {
                        inputs.0.push((snk_index, 0, 0));
                    }
                }
                else {
                    commands.entity(snk).insert(Inputs(vec![(snk_index, 0, 0)]));
                }
            }
        }
    }
}

