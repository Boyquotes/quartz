use bevy::{
    prelude::*,
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    render::primitives::Aabb,
};
use bevy::prelude::shape::Circle as BevyCircle;

use fundsp::hacker32::*;

use crate::components::*;

pub fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: Local<f32>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let r = cursor.f.distance(cursor.i);
        let v = 8;
        let color = Color::hsla(300., 1., 0.5, 1.);
        let id = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(BevyCircle { radius: r, vertices: v} .into()).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(cursor.i.extend(*depth)),
                ..default()
            },
            Radius(r),
            Col(color),
            Visible, //otherwise it can't be selected til after mark_visible is updated
            Order(0),
            NetChanged(true),
            Network(Net32::new(0,1)),
            NetIns(Vec::new()),
            crate::components::Num(0.),
            Arr(vec!(42., 105., 420., 1729.)),
            Op("empty".to_string()),
            Vertices(v),
            Save,
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn((Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                    TextStyle { color: Color::BLACK, font_size: 18., ..default() },
                ),
                TextSection::new(
                    "order: 0\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "empty\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "0",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
            ]),
            transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
            ..default()
        },
        Save,
        )).id();
        commands.entity(id).add_child(text);

        *depth += 0.00001;
    }
}

pub fn highlight_selected(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    added: Query<(Entity, &Radius, &Vertices), Added<Selected>>,
    mut removed: RemovedComponents<Selected>,
    highlight_query: Query<(Entity, &Parent), With<Highlight>>,
    children_query: Query<&Children>,
    resized: Query<(&Vertices, &Radius), (With<Selected>, Or<(Changed<Vertices>, Changed<Radius>)>)>,
    mesh_ids: Query<&Mesh2dHandle>,
) {
    for (id, r, v) in added.iter() {
        let highlight = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(BevyCircle{ radius: r.0 + 5., vertices: v.0} .into()).into(),
                material: materials.add(ColorMaterial::from(Color::hsl(0.0,1.0,0.5))),
                transform: Transform::from_translation(Vec3{z:-0.0000001, ..default()}),
                ..default()
            },
            Highlight,
        )).id();
        commands.entity(id).add_child(highlight);
    }
    'circle: for id in removed.read() {
        if let Ok(children) = children_query.get(id) {
            for child in children {
                if highlight_query.contains(*child) {
                    if let Some(mut e) = commands.get_entity(*child) {
                        e.remove_parent();
                        e.despawn();
                    }
                    continue 'circle;
                }
            }
        }
    }
    for (id, parent) in highlight_query.iter() {
        if let Ok((v, r)) = resized.get(parent.get()) {
            if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(id) {
                let mesh = meshes.get_mut(mesh_id).unwrap();
                *mesh = BevyCircle { radius: r.0 + 5., vertices: v.0 }.into();
            }
        }
    }
}

// loop over the visible entities and give them a Visible component
// so we can query just the visible entities
pub fn mark_visible(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<Entity, With<Visible>>,
    visible: Query<&VisibleEntities>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for e in query.iter() {
            commands.entity(e).remove::<Visible>();
        }
        let vis = visible.single();
        for e in vis.iter() {
            commands.entity(*e).insert(Visible);
        }
    }
}

pub fn draw_drawing_circle(
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    if mouse_button_input.pressed(MouseButton::Left) 
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: cursor.i.distance(cursor.f), vertices: 8 }.into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: 0., vertices: 3 }.into();
    }
}

//optimize all those distance calls, use a distance squared instead
pub fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Radius, &GlobalTransform), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut top_clicked_circle: Local<Option<(Entity, f32)>>,
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
) {
    if keyboard_input.pressed(KeyCode::Space) { return; }
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, t) in query.iter() {
            if top_clicked_circle.is_some() {
                if t.translation().z > top_clicked_circle.unwrap().1 &&
                    cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
                }
            } else {
                if cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
                }
            }
        }
        if let Some(top) = *top_clicked_circle {
            if !selected_query.contains(top.0) {
                if shift { commands.entity(top.0).insert(Selected); }
                else {
                    for entity in selected.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }
                    commands.entity(top.0).insert(Selected);
                }
            }
        }
    } else if mouse_button_input.pressed(MouseButton::Left) && top_clicked_circle.is_none() {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: cursor.i.distance(cursor.f), vertices: 8 }.into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: 0., vertices: 3 }.into();
        if top_clicked_circle.is_none() {
            if !shift {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            // select those in the dragged area
            for (e, r, t) in query.iter() {
                if cursor.i.distance(cursor.f) + r.0 > cursor.i.distance(t.translation().xy()) {
                    commands.entity(e).insert(Selected);
                }
            }
        }
        *top_clicked_circle = None;
    }
}

pub fn select_all(
    mut commands: Commands,
    order_query: Query<Entity, With<Order>>,
    connection_query: Query<Entity, Or<(With<BlackHole>, With<WhiteHole>)>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if ctrl && keyboard_input.pressed(KeyCode::A) {
        if shift {
            for e in connection_query.iter() { commands.entity(e).insert(Selected); }
        } else {
            for e in order_query.iter() { commands.entity(e).insert(Selected); }
        }
    }
}

// TODO(amy): use scenes
pub fn duplicate_selected() {}

pub fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.t {
        if mouse_button_input.pressed(MouseButton::Left) &&
        //lol because the update to entities isn't read until the next frame
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for mut t in query.iter_mut() {
                t.translation.y += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for mut t in query.iter_mut() {
                t.translation.y -= 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for mut t in query.iter_mut() {
                t.translation.x += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for mut t in query.iter_mut() {
                t.translation.x -= 1.;
            }
        }
    }
}

pub fn rotate_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.o {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.rotate_z(cursor.d.y / 100.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::Right]) {
            for mut t in query.iter_mut() {
                t.rotate_z(0.01);
            }
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::Left]) {
            for mut t in query.iter_mut() {
                t.rotate_z(-0.01);
            }
        }
    }
}

pub fn update_color(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Col, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        if drag_modes.h {
            for mut c in query.iter_mut() {
                let h = (c.0.h() + cursor.d.x).clamp(0., 360.);
                c.0.set_h(h);
            }
        }
        if drag_modes.s {
            for mut c in query.iter_mut() {
                let s = (c.0.s() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_s(s);
            }
        }
        if drag_modes.l {
            for mut c in query.iter_mut() {
                let l = (c.0.l() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_l(l);
            }
        }
        if drag_modes.a {
            for mut c in query.iter_mut() {
                let a = (c.0.a() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::Left, KeyCode::Down]) {
        for mut c in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() - 1.).clamp(0., 360.);
                c.0.set_h(h);
            }
            if drag_modes.s {
                let s = (c.0.s() - 0.01).clamp(0., 1.);
                c.0.set_s(s);
            }
            if drag_modes.l {
                let l = (c.0.l() - 0.01).clamp(0., 1.);
                c.0.set_l(l);
            }
            if drag_modes.a {
                let a = (c.0.a() - 0.01).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::Right, KeyCode::Up]) {
        for mut c in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() + 1.).clamp(0., 360.);
                c.0.set_h(h);
            }
            if drag_modes.s {
                let s = (c.0.s() + 0.01).clamp(0., 1.);
                c.0.set_s(s);
            }
            if drag_modes.l {
                let l = (c.0.l() + 0.01).clamp(0., 1.);
                c.0.set_l(l);
            }
            if drag_modes.a {
                let a = (c.0.a() + 0.01).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
}

pub fn update_mat(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    color_query: Query<(Entity, &Col), Changed<Col>>,
) {
    for (id, c) in color_query.iter() {
        if let Ok(mat_id) = material_ids.get(id) {
            let mat = mats.get_mut(mat_id).unwrap();
            mat.color = c.0;
        }
    }
}

pub fn update_radius(
    mut query: Query<&mut Radius, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.r {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut r in query.iter_mut() {
                r.0 += cursor.d.y;
                r.0 = r.0.max(0.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::Right]) {
            for mut r in query.iter_mut() {
                r.0 += 1.;
            }
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::Left]) {
            for mut r in query.iter_mut() {
                r.0 = (r.0 - 1.).max(0.);
            }
        }
    }
}

pub fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut query: Query<(Entity, &Vertices, &Radius, &mut Aabb), Or<(Changed<Vertices>, Changed<Radius>)>>,
) {
    for (id, v, r, mut aabb) in query.iter_mut() {
        if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(id) {
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = BevyCircle { radius: r.0, vertices: v.0 }.into();
            *aabb = mesh.compute_aabb().unwrap();
        }
    }
}

pub fn update_num(
    mut query: Query<&mut crate::components::Num, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.n {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut n in query.iter_mut() {
                n.0 += cursor.d.y / 10.;
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for mut n in query.iter_mut() {
                n.0 += 0.01;
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for mut n in query.iter_mut() {
                n.0 -= 0.01;
            }
        }
    }
}

pub fn update_net_from_op(
    mut query: Query<(&Op, &mut NetChanged, &mut Network, &mut NetIns), Changed<Op>>,
) {
    for (op, mut net_changed, mut n, mut inputs) in query.iter_mut() {
        net_changed.0 = true;
        match op.0.as_str() {
            "Var" => {
                let input = shared(0.);
                n.0 = Net32::wrap(Box::new(var(&input)));
                inputs.0.clear();
                inputs.0.push(input);
            },
            // testing
            "0outs" => {
                n.0 = Net32::new(0,0);
            },
            "1outs" => {
                n.0 = Net32::new(0,1);
            },
            "2outs" => {
                n.0 = Net32::new(0,2);
            },
            "3outs" => {
                n.0 = Net32::new(0,3);
            },
            "4outs" => {
                n.0 = Net32::new(0,4);
            },
            _ => {
                n.0 = Net32::wrap(Box::new(dc(0.)));
                inputs.0.clear();
            },
        }
    }
}

pub fn update_order (
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Order, With<Selected>>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketRight) {
        for mut order in query.iter_mut() {
            order.0 += 1;
            order_change.send_default();
        }
    }
    if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        for mut order in query.iter_mut() {
            if order.0 > 0 {
                order.0 -= 1;
                order_change.send_default();
            }
        }
    }
}

pub fn shake_order (
    keyboard_input: Res<Input<KeyCode>>,
    changed_order: Query<(Entity, &Children), With<Order>>,
    mut order_query: Query<&mut Order>,
    white_hole_query: Query<&WhiteHole>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.just_pressed(KeyCode::Key0) {
        for (e, children) in changed_order.iter() {
            for child in children {
                if let Ok(wh) = white_hole_query.get(*child) {
                    let this = order_query.get(e).unwrap().0;
                    let previous = order_query.get(wh.bh_parent).unwrap().0;
                    if this <= previous {
                        order_query.get_mut(e).unwrap().0 = previous + 1;
                    }
                }
            }
        }
        order_change.send_default();
    }
}

// FIXME(amy): you can loop the Visible (view vis)
// that'd avoid the just-created glitch
pub fn update_circle_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    order_query: Query<&Order, Changed<Order>>,
    num_query: Query<&crate::components::Num, Changed<crate::components::Num>>,
    op_query: Query<&Op, Changed<Op>>,
) {
    for (mut text, parent) in query.iter_mut() {
        if let Ok(order) = order_query.get(**parent) {
            text.sections[1].value = "order: ".to_string() + &order.0.to_string() + "\n";
        }
        if let Ok(op) = op_query.get(**parent) {
            text.sections[2].value = op.0.clone() + "\n";
        }
        if let Ok(num) = num_query.get(**parent) {
            text.sections[3].value = num.0.to_string();
        }
    }
}

pub fn delete_selected_circles(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Children), (With<Selected>, With<Order>)>,
    bh_query: Query<&BlackHole, Without<Selected>>,
    wh_query: Query<&WhiteHole, Without<Selected>>,
    arrow_query: Query<&ConnectionArrow>,
    text_query: Query<Entity, With<Text>>,
    highlight_query: Query<Entity, With<Highlight>>,
    mut commands: Commands,
    mut order_change: EventWriter<OrderChange>,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keyboard_input.just_pressed(KeyCode::Delete) && !shift {
        for (e, children) in query.iter() {
            for child in children {
                // TODO(amy): do we need to remove parent?
                if let Ok(bh) = bh_query.get(*child) {
                    if wh_query.contains(bh.wh) {
                        let arrow = arrow_query.get(bh.wh).unwrap().0;
                        commands.entity(arrow).despawn();
                        commands.entity(*child).remove_parent();
                        commands.entity(*child).despawn_recursive();
                        commands.entity(bh.wh).remove_parent();
                        commands.entity(bh.wh).despawn_recursive();
                    }
                } else if let Ok(wh) = wh_query.get(*child) {
                    if bh_query.contains(wh.bh) {
                        // don't remove things that will get removed later
                        if query.contains(wh.bh_parent) { continue; }
                        let arrow = arrow_query.get(*child).unwrap().0;
                        commands.entity(arrow).despawn();
                        commands.entity(wh.bh).remove_parent();
                        commands.entity(wh.bh).despawn_recursive();
                        commands.entity(*child).remove_parent();
                        commands.entity(*child).despawn_recursive();
                    }
                } else if text_query.contains(*child) || highlight_query.contains(*child) {
                    commands.entity(*child).remove_parent();
                    commands.entity(*child).despawn();
                }
            }
            commands.entity(e).despawn();
            order_change.send_default();
        }
    }
}

pub fn mark_children_change(
    query: Query<&Children, (With<Order>, Changed<Transform>)>,
    mut trans_query: Query<&mut Transform, Without<Order>>,
) {
    for children in query.iter() {
        for child in children {
            trans_query.get_mut(*child).unwrap().set_changed();
        }
    }
}

