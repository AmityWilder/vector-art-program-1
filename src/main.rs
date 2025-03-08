#![feature(let_chains, if_let_guard, arbitrary_self_types)]
#![warn(arithmetic_overflow, clippy::arithmetic_side_effects)]

use std::sync::Arc;
use document::{Artboard, Document};
use editor::{Editor, MaybeNew, Tool};
use engine::{Engine, EngineTab, EngineTabData, EngineTheme};
use layer::{Layer, LayerContent};
use raylib::prelude::{KeyboardKey::*, MouseButton::*, *};
use style::{Style, WidthProfile};

/// Vector path
mod curve;

/// Serializeable artwork
mod document;

/// [Document][`crate::document::Document`] editing and tools
mod editor;

/// Organizer for all open [editor][`crate::editor::Editor`]
mod engine;

/// [Document][`crate::document::Document`] element
mod layer;

/// Layer appearance modification
mod style;

#[allow(clippy::cognitive_complexity, reason = "you always overcomplicate everything when you listen to this about the main function, Amy.")]
fn main() {
    let (mut rl, thread) = init()
        .title("Amity Vector Art")
        .size(1280, 720)
        .resizable()
        .build();

    rl.set_target_fps(60);
    rl.set_window_state(WindowState::set_window_maximized(rl.get_window_state(), true));

    // initialize engine
    let mut engine = Engine::new(EngineTheme::default_theme());

    // new/open file arent implemented yet, but I still want to make sure documents work right
    #[cfg(debug_assertions)]
    {
        engine.create_editor({
            let mut document = Document::new("untitled".to_owned());
            let profile = Arc::downgrade(document.create_width_profile(WidthProfile::default_width_profile()));
            let mut editor = Editor::new(document, MaybeNew::New(Style::default_style(profile)));
            let style = editor.upgrade_current_style().clone();
            editor.document.artboards.push({
                Artboard::new("artboard 1".to_owned(), Rectangle::new(0.0, 0.0, 512.0, 512.0))
            });
            let content = LayerContent::Curve(Arc::downgrade(
                editor.document.create_curve(make_curve!((60,60)[10,0]->[0,-10](80,80)[0,-10]->[-10,0](100,60)))
            ));
            editor.document.layers.push(Layer {
                name: "new layer".to_owned(),
                content,
                style,
            });
            editor
        });
    }

    while !rl.window_should_close() {
        // editor tabs
        {
            if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
                let mouse_pos = rl.get_mouse_position();
                if let Some(EngineTab { data, .. }) = engine.tab_iter().find(|tab| tab.rect.check_collision_point_rec(mouse_pos)) {
                    match data {
                        EngineTabData::Editor { index, close_button_rect, .. } => {
                            if close_button_rect.check_collision_point_rec(mouse_pos) {
                                engine.remove_editor(index);
                            } else {
                                engine.focus_editor(index);
                            }
                        }

                        EngineTabData::New => {
                            engine.create_editor({
                                let mut document = Document::new("untitled".to_owned());
                                let profile = Arc::downgrade(document.create_width_profile(WidthProfile::default_width_profile()));
                                Editor::new(document, MaybeNew::New(Style::default_style(profile)))
                            });
                        }

                        EngineTabData::Open => {
                            todo!("open file dialogue not yet implemented");
                        }
                    }
                }
            }
        }

        // tick editor
        if let Some(editor) = engine.focused_editor_mut() {
            // editor inputs
            {
                if rl.is_key_pressed(KEY_P) {
                    editor.current_tool = Tool::PointSelect;
                } else if rl.is_key_pressed(KEY_B) {
                    editor.current_tool =
                        if rl.is_key_down(KEY_LEFT_SHIFT) {
                            Tool::VectorBrush
                        } else {
                            Tool::RasterBrush
                        }
                } else if rl.is_key_pressed(KEY_V) {
                    editor.current_tool = Tool::PointSelect;
                }
            }

            // zoom and pan
            {
                let mut pan = Vector2::zero();

                let mut scroll = Vector2::from(rl.get_mouse_wheel_move_v());
                if rl.is_key_down(KEY_LEFT_ALT) {
                    const ZOOM_SPEED: f32 = 1.5;
                    const MIN_ZOOM: f32 = 0.125;
                    const MAX_ZOOM: f32 = 64.0;
                    let zoom = if scroll.x.abs() < scroll.y.abs() { scroll.y } else { scroll.x };
                    if zoom > 0.0 && editor.camera.zoom < MAX_ZOOM {
                        editor.camera.zoom *= ZOOM_SPEED;
                    } else if zoom < 0.0 && editor.camera.zoom > MIN_ZOOM {
                        editor.camera.zoom /= ZOOM_SPEED;
                    }
                } else {
                    if rl.is_key_down(KEY_LEFT_SHIFT) {
                        std::mem::swap(&mut scroll.x, &mut scroll.y);
                    }
                    pan += scroll * 20.0;
                }
                if rl.is_mouse_button_down(MOUSE_BUTTON_MIDDLE) {
                    let drag = rl.get_mouse_delta();
                    pan += drag;
                }

                editor.camera.target += (rl.get_mouse_delta() - pan) / editor.camera.zoom;
                editor.camera.offset += rl.get_mouse_delta(); // equivalent to `rl.get_mouse_position()` when loading a file
            }

            // tick current tool
            match editor.current_tool {
                Tool::PointSelect => {

                }

                Tool::VectorBrush => {

                }

                Tool::VectorPen => {

                }

                Tool::RasterBrush => {

                }
            }
        }

        // draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(engine.theme.color_background);

        // draw focused editor
        if let Some(editor) = engine.focused_editor() {
            // draw viewport 2D
            {
                let mut d = d.begin_mode2D(editor.camera);

                // draw artboard backgrounds
                for artboard in &editor.document.artboards {
                    d.draw_rectangle_rec(artboard.rect, editor.document.paper_color);
                }

                // draw artwork
                for layer in &editor.document.layers {
                    match &layer.content {
                        // draw curve
                        LayerContent::Curve(curve) => {
                            let strong_curve = curve.upgrade().expect("should not hold onto dead layer");
                            let curve_lock = strong_curve.lock();
                            let curve_borrow = curve_lock.borrow();

                            let iter = curve_borrow
                                .pos_vel_iter::<40>()
                                .flat_map(|(i, t, p, v)| {
                                    const ROTATE_90DEG: na::Matrix2<f32> = na::Matrix2::new(
                                        0.0, -1.0,
                                        1.0,  0.0,
                                    );
                                    let _t_full = i as f32 + t;
                                    let tangent = v.try_normalize(f32::EPSILON)?;
                                    let outer = ROTATE_90DEG * tangent;
                                    let inner = -outer;
                                    Some((p + inner, p + outer))
                                });

                            for (inner, outer) in iter {
                                d.draw_line_v(Vector2::from(inner), Vector2::from(outer), Color::RED);
                            }
                        }

                        // draw group
                        LayerContent::Group(_group) => {
                            todo!("group rendering not yet implemented")
                        }
                    }
                }
            }

            // draw tool visuals
            match editor.current_tool {
                Tool::PointSelect => {

                }

                Tool::VectorBrush => {

                }

                Tool::VectorPen => {

                }

                Tool::RasterBrush => {

                }
            }

            // draw artboard name
            for artboard in &editor.document.artboards {
                let corner = d.get_world_to_screen2D(Vector2::new(artboard.rect.x, artboard.rect.y), editor.camera);
                d.draw_text(&artboard.name, corner.x as i32, corner.y as i32 - engine.theme.font_size, engine.theme.font_size, engine.theme.color_foreground);
            }
        }

        // draw editor tabs
        d.draw_rectangle_rec(engine.tab_well(d.get_render_width() as f32), engine.theme.color_panel_edge);
        for tab in engine.tab_iter() {
            let is_hovered = tab.rect.check_collision_point_rec(d.get_mouse_position());
            match tab.data {
                EngineTabData::Editor { index, editor, close_button_rect } => {
                    let is_close_button_hovered = is_hovered && close_button_rect.check_collision_point_rec(d.get_mouse_position());
                    let is_focused = engine.focused_editor_index_eq(index);

                    let tab_color = if is_focused {
                        engine.theme.color_accent
                    } else if is_hovered {
                        engine.theme.color_panel_edge
                    } else {
                        engine.theme.color_panel
                    };

                    let close_color = if is_close_button_hovered {
                        engine.theme.color_destructive
                    } else if is_focused {
                        engine.theme.color_foreground
                    } else if is_hovered {
                        engine.theme.color_panel
                    } else {
                        engine.theme.color_panel_edge
                    };

                    d.draw_rectangle_rec(tab.rect, tab_color);
                    d.draw_rectangle_rec(close_button_rect, close_color);
                    d.draw_text(
                        &editor.document.title,
                        (tab.rect.x + Engine::TAB_PADDING_H) as i32,
                        (tab.rect.y + Engine::TAB_PADDING_V) as i32,
                        engine.theme.font_size,
                        engine.theme.color_foreground,
                    );
                }

                EngineTabData::New | EngineTabData::Open => {
                    let tab_color = if is_hovered {
                        engine.theme.color_accent
                    } else {
                        engine.theme.color_panel
                    };

                    d.draw_rectangle_rec(tab.rect, tab_color);
                    d.draw_text(
                        match tab.data {
                            EngineTabData::Editor { .. } => unreachable!(),
                            EngineTabData::New => "+",
                            EngineTabData::Open => "o",
                        },
                        (tab.rect.x + Engine::TAB_PADDING_H) as i32,
                        (tab.rect.y + Engine::TAB_PADDING_V) as i32,
                        engine.theme.font_size,
                        engine.theme.color_foreground,
                    );
                }
            }
        }
    }
}
