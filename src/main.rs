#![feature(let_chains, if_let_guard, arbitrary_self_types)]
#![warn(arithmetic_overflow, clippy::arithmetic_side_effects)]

use document::{Artboard, Document};
use editor::{Editor, MaybeNewStyle, Tool};
use engine::{Engine, EngineTab, EngineTabData};
use raylib::prelude::{KeyboardKey::*, MouseButton::*, *};

pub mod curve;
pub mod document;
pub mod editor;
pub mod engine;
pub mod layer;
pub mod style;

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
    let mut engine = Engine::new();
    #[cfg(debug_assertions)]
    {
        engine.create_editor({
            Editor::new({
                let mut document = Document::new("untitled".to_owned());
                let artboard = Artboard::new("artboard 1".to_owned(), Rectangle::new(0.0, 0.0, 512.0, 512.0));
                document.artboards.push(artboard);
                document
            }, MaybeNewStyle::new_default())
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
                                _ = engine.remove_editor(index);
                            } else {
                                engine.focus_editor(index).expect("tab_iter should only iterate over valid indices");
                            }
                        }

                        EngineTabData::New => {
                            engine.create_editor({
                                Editor::new({
                                    let document = Document::new("untitled".to_owned());
                                    document
                                }, MaybeNewStyle::new_default())
                            });
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
            // draw artboard background
            {
                let mut d = d.begin_mode2D(editor.camera);
                for artboard in &editor.document.artboards {
                    d.draw_rectangle_rec(artboard.rect, editor.document.paper_color);
                }
            }

            // draw artwork
            {
                // TODO
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
                        engine.theme.color_danger
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

                EngineTabData::New => {
                    let tab_color = if is_hovered {
                        engine.theme.color_accent
                    } else {
                        engine.theme.color_panel
                    };

                    d.draw_rectangle_rec(tab.rect, tab_color);
                    d.draw_text(
                        "+",
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
