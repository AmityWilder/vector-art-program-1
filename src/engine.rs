use std::{ffi::CString, num::NonZeroU32, str::FromStr};
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
    /// one-indexed instead of zero-indexed
    focused_editor: Option<NonZeroU32>,
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
        self.focused_editor = NonZeroU32::new(self.editors.len() as u32);
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
            self.focused_editor = NonZeroU32::new(idx.checked_add(1).expect("4,294,967,295 editors is too many"));
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
        self.focused_editor.is_some_and(|focused| (focused.get() - 1) == idx)
    }

    pub fn focused_editor(&self) -> Option<&Editor> {
        self.focused_editor.map(|idx| &self.editors[(idx.get() - 1) as usize])
    }

    pub fn focused_editor_mut(&mut self) -> Option<&mut Editor> {
        self.focused_editor.map(|idx| &mut self.editors[(idx.get() - 1) as usize])
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
            if focused_editor.get() > num_editors {
                self.focused_editor = NonZeroU32::new(num_editors)
            } else if focused_editor.get() > index + 1 {
                self.focused_editor = NonZeroU32::new(focused_editor.get() - 1);
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

pub struct EngineTabIter<'a> {
    iter: std::slice::Iter<'a, Editor>,
    font_size: i32,
    rect: Rectangle,
    close_button_rect: Rectangle,
}

impl<'a> EngineTabIter<'a> {
    fn new(iter: std::slice::Iter<'a, Editor>, font_size: i32) -> Self {
        Self {
            iter,
            font_size,
            rect: Rectangle::new(0.0, 0.0, 0.0, font_size as f32 + Engine::TAB_PADDING_V * 2.0),
            close_button_rect: Rectangle::new(-Engine::TAB_PADDING_H - font_size as f32, Engine::TAB_PADDING_V, font_size as f32, font_size as f32),
        }
    }
}

impl<'a> Iterator for EngineTabIter<'a> {
    type Item = (&'a str, Rectangle, Rectangle);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(editor) = self.iter.next() {
            let tab_name = editor.document.title.as_str();
            let name_width = unsafe { MeasureText(CString::from_str(tab_name).unwrap().as_ptr(), self.font_size) } as f32;
            let tab_width = name_width + Engine::TAB_PADDING_H * 4.0 + self.font_size as f32;
            self.rect.width = tab_width.min(Engine::TAB_MAX_WIDTH);
            self.close_button_rect.x += self.rect.width;
            let (rect, close_button_rect) = (self.rect, self.close_button_rect);
            self.rect.x += self.rect.width;
            Some((tab_name, rect, close_button_rect))
        } else {
            None
        }
    }
}
