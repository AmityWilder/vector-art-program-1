use std::{ffi::CString, str::FromStr};
use raylib::{ffi::MeasureText, prelude::*};

use crate::Editor;

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineTheme {
    pub color_background: Color,
    pub color_foreground: Color,
    pub color_panel: Color,
    pub color_panel_edge: Color,
    pub color_accent: Color,
    pub color_danger: Color,
    pub font_size: i32,
}

impl EngineTheme {
    pub const fn default_theme() -> Self {
        Self {
            color_background: Color::new(24, 24, 24, 255),
            color_foreground: Color::new(200, 200, 200, 255),
            color_panel: Color::new(48, 48, 48, 255),
            color_panel_edge: Color::new(32, 32, 32, 255),
            color_accent: Color::BLUEVIOLET,
            color_danger: Color::RED,
            font_size: 10,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub theme: EngineTheme,
    editors: Vec<Editor>,
    /// The editor currently receiving mouse/keyboard events.
    focused_editor: Option<u32>,
}

impl Engine {
    pub const TAB_PADDING_H: f32 = 5.0;
    pub const TAB_PADDING_V: f32 = 3.0;
    pub const TAB_MAX_WIDTH: f32 = 100.0;

    pub const fn new() -> Self {
        Self {
            editors: Vec::new(),
            theme: EngineTheme::default_theme(),
            focused_editor: None,
        }
    }

    /// Pushes the editor and focuses it
    pub fn create_editor(&mut self, editor: Editor) {
        self.editors.push(editor);
        self.focused_editor = (self.editors.len() as u32).checked_sub(1);
    }

    #[inline]
    pub fn editors(&self) -> &[Editor] {
        &self.editors
    }

    /// Mutable elements, immutable size
    #[inline]
    pub fn editors_mut(&mut self) -> &mut [Editor] {
        &mut self.editors
    }

    #[inline]
    pub fn editor(&self, idx: u32) -> Option<&Editor> {
        self.editors.get(idx as usize)
    }

    #[inline]
    pub fn editor_mut(&mut self, idx: u32) -> Option<&mut Editor> {
        self.editors.get_mut(idx as usize)
    }

    /// Returns [`Some`] on success, [`None`] if the index is out of bounds
    #[must_use]
    pub fn focus_editor(&mut self, idx: u32) -> Option<()> {
        if idx < self.editors.len() as u32 {
            self.focused_editor = Some(idx);
            Some(())
        } else {
            None
        }
    }

    #[inline]
    pub fn unfocus_editor(&mut self) {
        self.focused_editor = None;
    }

    pub fn focused_editor_index_eq(&self, idx: u32) -> bool {
        self.focused_editor.is_some_and(|focused| focused == idx)
    }

    pub fn focused_editor(&self) -> Option<&Editor> {
        self.focused_editor.map(|idx| &self.editors[idx as usize])
    }

    pub fn focused_editor_mut(&mut self) -> Option<&mut Editor> {
        self.focused_editor.map(|idx| &mut self.editors[idx as usize])
    }

    /// Removes the editor at the index and returns it.
    ///
    /// If the editor is currently focused, the focused editor will be whatever was previously next in the array.
    ///
    /// If the focused editor was after the removed editor, the focused editor will be changed so it refers to the same element.
    ///
    /// If the focused editor was at the end of the array, and it is the one getting removed, the editor that came before it in the array will be focused.
    pub fn remove_editor(&mut self, index: u32) -> Editor {
        let editor = self.editors.remove(index as usize);
        let num_editors = self.editors.len() as u32;
        if let Some(focused_editor) = &self.focused_editor {
            if *focused_editor >= num_editors {
                self.focused_editor = num_editors.checked_sub(1)
            } else if index < *focused_editor {
                self.focused_editor = Some(focused_editor - 1);
            }
        }
        editor
    }

    pub fn tab_iter(&self) -> EngineTabIter<'_> {
        EngineTabIter::new(self.editors.iter(), self.theme.font_size)
    }

    /// Get (calculate) rectangle that editor tabs reside in
    pub fn tab_well(&self, window_width: f32) -> Rectangle {
        Rectangle::new(0.0, 0.0, window_width, self.theme.font_size as f32 + Engine::TAB_PADDING_V * 2.0)
    }
}

pub enum EngineTabData<'a> {
    Editor {
        index: u32,
        editor: &'a Editor,
        close_button_rect: Rectangle,
    },
    New,
}

pub struct EngineTab<'a> {
    pub rect: Rectangle,
    pub data: EngineTabData<'a>,
}

enum EngineTabIterData {
    Editor {
        index: u32,
        close_button_rect: Rectangle,
    },
    New,
}

pub struct EngineTabIter<'a> {
    iter: std::slice::Iter<'a, Editor>,
    font_size: i32,
    rect: Rectangle,
    data: EngineTabIterData,
}

impl<'a> EngineTabIter<'a> {
    fn new(iter: std::slice::Iter<'a, Editor>, font_size: i32) -> Self {
        Self {
            iter,
            font_size,
            rect: Rectangle::new(
                0.0,
                0.0,
                0.0,
                font_size as f32 + Engine::TAB_PADDING_V * 2.0,
            ),
            data: EngineTabIterData::Editor {
                index: 0,
                close_button_rect: Rectangle::new(
                    -Engine::TAB_PADDING_H - font_size as f32,
                    Engine::TAB_PADDING_V,
                    font_size as f32,
                    font_size as f32,
                ),
            }
        }
    }
}

impl<'a> Iterator for EngineTabIter<'a> {
    type Item = EngineTab<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(editor) = self.iter.next() {
            let EngineTabIterData::Editor { index, close_button_rect } = &mut self.data else { panic!("every tabs at the start should be an editor tab") };
            let tab_name = editor.document.title.as_str();
            let name_width = unsafe { MeasureText(CString::from_str(tab_name).unwrap().as_ptr(), self.font_size) } as f32;
            let tab_width = name_width + Engine::TAB_PADDING_H * 4.0 + self.font_size as f32;
            self.rect.width = tab_width.min(Engine::TAB_MAX_WIDTH);
            close_button_rect.x += self.rect.width;
            let (idx, rect, close_rec) = (*index, self.rect, *close_button_rect);
            *index += 1;
            self.rect.x += self.rect.width + 1.0;
            close_button_rect.x += 1.0;
            Some(EngineTab {
                rect,
                data: EngineTabData::Editor {
                    index: idx,
                    editor,
                    close_button_rect: close_rec,
                },
            })
        } else {
            match self.data {
                EngineTabIterData::Editor { .. } => {
                    self.data = EngineTabIterData::New;
                    self.rect.width = self.rect.height;
                    let rect = self.rect;
                    self.rect.x += self.rect.width + 1.0;
                    Some(EngineTab {
                        rect,
                        data: EngineTabData::New,
                    })
                }

                EngineTabIterData::New => {
                    None
                }
            }
        }
    }
}
