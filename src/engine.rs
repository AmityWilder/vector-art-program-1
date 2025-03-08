use std::{ffi::CString, str::FromStr};
use raylib::{ffi::MeasureText, prelude::*};

use crate::Editor;

/// Application-wide visual customization options
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineTheme {
    /// The background color of the viewport
    pub color_background: Color,
    /// The color of text
    pub color_foreground: Color,
    /// The background color of panels
    pub color_panel: Color,
    /// The background color of panel outlines
    pub color_panel_edge: Color,
    /// The color of selected elements
    pub color_accent: Color,
    /// The color of buttons that perform destructive (not necessarily irreversible) actions
    pub color_destructive: Color,
    /// The color of buttons that perform irreversible actions
    pub color_irreversible: Color,
    /// The vertical size of standard UI text
    pub font_size: i32,
}

impl EngineTheme {
    /// The theme used by the application when the user hasn't customized it
    pub const fn default_theme() -> Self {
        Self {
            color_background: Color::new(24, 24, 24, 255),
            color_foreground: Color::new(200, 200, 200, 255),
            color_panel: Color::new(48, 48, 48, 255),
            color_panel_edge: Color::new(32, 32, 32, 255),
            color_accent: Color::BLUEVIOLET,
            color_destructive: Color::CORAL,
            color_irreversible: Color::RED,
            font_size: 10,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    /// The visual theme of the application
    pub theme: EngineTheme,

    editors: Vec<Editor>,

    /// The index of the editor currently receiving mouse/keyboard events
    ///
    /// [`None`] if no editor is focused (for example: if there are no editors)
    focused_editor: Option<u32>,
}

impl Engine {
    /// Horizontal padding from the edge to the content of an editor tab
    pub const TAB_PADDING_H: f32 = 5.0;

    /// Vertical padding from the edge to the content of an editor tab
    pub const TAB_PADDING_V: f32 = 3.0;

    /// Maximum width (including close button and padding) of an editor tab
    ///
    /// Tab names exceeding this should be clipped
    pub const TAB_MAX_WIDTH: f32 = 100.0;

    /// Construct an engine without allocations
    pub const fn new(theme: EngineTheme) -> Self {
        Self {
            editors: Vec::new(),
            theme,
            focused_editor: None,
        }
    }

    /// Push an editor and focuses it
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

    /// Set the focus index
    ///
    /// # Panics
    /// Panics if index is out of bounds
    pub fn focus_editor(&mut self, idx: u32) {
        self.focused_editor = Some(idx);
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

    /// Remove the editor at the index and returns it
    ///
    /// Key:
    /// - `^` - Focused
    /// - `x` - Removed
    ///
    /// If the focused editor is ahead of the editor being removed,
    /// the focues editor will be unaffected
    /// ```no_run
    ///           x
    /// [1][2][3][4][5]
    ///     ^
    ///            <-
    /// [1][2][3][5]
    ///     ^
    /// ```
    ///
    /// If the focused editor follows the editor being removed,
    /// the focus index will be changed so it refers to the same element
    /// ```no_run
    ///     x
    /// [1][2][3][4][5]
    ///           ^
    ///      <- <- <-
    /// [1][3][4][5]
    ///        ^<-
    /// ```
    ///
    /// If the editor being removed is currently focused,
    /// the focus index will be unchanged
    /// ```no_run
    ///     x
    /// [1][2][3][4][5]
    ///     ^
    ///      <- <- <-
    /// [1][3][4][5]
    ///     ^<-
    /// ```
    ///
    /// If the editor being removed is currently focused and at the end of the array,
    /// the focus index will move back an element
    /// ```no_run
    ///              x
    /// [1][2][3][4][5]
    ///              ^
    ///
    /// [1][2][3][4]
    ///           ^<-
    /// ```
    ///
    /// If there is only one editor,
    /// the focus index will be cleared
    ///
    /// # Panics
    /// Panics if index is out of bounds
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

    /// Iterate over tabs
    ///
    /// Order of tabs:
    /// - All [`Editor`] tabs
    /// - New document `+`
    /// - Open document `o`
    pub fn tab_iter(&self) -> EngineTabIter<'_> {
        EngineTabIter::new(self.editors.iter(), self.theme.font_size)
    }

    /// Get (calculate) tab bar rectangle
    pub fn tab_well(&self, window_width: f32) -> Rectangle {
        Rectangle::new(0.0, 0.0, window_width, self.theme.font_size as f32 + Engine::TAB_PADDING_V * 2.0)
    }
}

pub enum EngineTabData<'a> {
    /// An [`Editor`] tab
    Editor {
        /// The index of `editors`'s in the [`Engine`]
        index: u32,

        /// A reference to the editor the tab represents
        editor: &'a Editor,

        /// The bounding rectangle for the close button
        close_button_rect: Rectangle,
    },
    /// The "new document" tab
    New,
    /// The "open document" tab
    Open,
}

pub struct EngineTab<'a> {
    /// The bounding rectangle for the tab
    pub rect: Rectangle,

    /// Tab-type-specific information
    pub data: EngineTabData<'a>,
}

enum EngineTabIterData {
    Editor {
        index: u32,
        close_button_rect: Rectangle,
    },
    New,
    Open,
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
                EngineTabIterData::Editor { .. } | EngineTabIterData::New => {
                    let data;
                    (self.data, data) = match self.data {
                        EngineTabIterData::Editor { .. } => (EngineTabIterData::New, EngineTabData::New),
                        EngineTabIterData::New => (EngineTabIterData::Open, EngineTabData::Open),
                        EngineTabIterData::Open => unreachable!(),
                    };
                    self.rect.width = self.rect.height;
                    let rect = self.rect;
                    self.rect.x += self.rect.width + 1.0;
                    Some(EngineTab {
                        rect,
                        data,
                    })
                }

                EngineTabIterData::Open => {
                    None
                }
            }
        }
    }
}
