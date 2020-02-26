//! A module defining TextField. TextField is a basegl component displaying editable block of text.

pub mod content;
pub mod cursor;
pub mod frp;
pub mod location;
pub mod render;

use crate::prelude::*;

use crate::control::callback::CallbackMut1;
use crate::display::object::DisplayObjectData;
use crate::display::shape::text::text_field::content::TextFieldContent;
use crate::display::shape::text::text_field::content::TextChange;
use crate::display::shape::text::text_field::cursor::Cursors;
use crate::display::shape::text::text_field::cursor::Cursor;
use crate::display::shape::text::text_field::cursor::Step;
use crate::display::shape::text::text_field::cursor::CursorNavigation;
use crate::display::shape::text::text_field::location::TextLocation;
use crate::display::shape::text::text_field::location::TextLocationChange;
use crate::display::shape::text::text_field::frp::TextFieldFrp;
use crate::display::shape::text::glyph::font::FontHandle;
use crate::display::shape::text::glyph::font::FontRegistry;
use crate::display::shape::text::text_field::render::TextFieldSprites;
use crate::display::shape::text::text_field::render::assignment::GlyphLinesAssignmentUpdate;
use crate::display::world::World;

use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use std::ops::Range;



// ===============================
// === TextChangedNotification ===
// ===============================

#[derive(Clone,Debug)]
pub struct TextChangedNotification {
    pub replaced_range      : Range<TextLocation>,
    pub replaced_range_char : Range<usize>,
    pub inserted_string     : String,
}



// =====================
// === TextComponent ===
// =====================

/// A display properties of TextField.
#[derive(Debug)]
pub struct TextFieldProperties {
    /// FontHandle used for rendering text.
    pub font: FontHandle,
    /// Text size being a line height in pixels.
    pub text_size: f32,
    /// Base color of displayed text.
    //TODO: base_color should use definitions in core/data/color
    pub base_color: Vector4<f32>,
    /// Size of this component.
    pub size: Vector2<f32>,
}

impl TextFieldProperties {
    const DEFAULT_FONT_FACE:&'static str = "DejaVuSansMono";

    /// A default set of properties.
    pub fn default(fonts:&mut FontRegistry) -> Self {
        TextFieldProperties {
            font      : fonts.get_or_load_embedded_font(Self::DEFAULT_FONT_FACE).unwrap(),
            text_size : 16.0,
            base_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            size      : Vector2::new(100.0,100.0),
        }
    }
}

shared! { TextField

    /// Component rendering text
    ///
    /// This component is under heavy construction, so the api may easily changed in few future
    /// commits.
    #[derive(Derivative)]
    #[derivative(Debug)]
    pub struct TextFieldData {
        properties           : TextFieldProperties,
        content              : TextFieldContent,
        cursors              : Cursors,
        rendered             : TextFieldSprites,
        display_object       : DisplayObjectData,
        frp                  : Option<TextFieldFrp>,
        #[derivative(Debug="ignore")]
        text_change_callback : Option<CallbackMut1<TextChangedNotification>>
    }

    impl {
        /// Set position of this TextField.
        pub fn set_position(&mut self, position:Vector3<f32>) {
            self.display_object.set_position(position);
        }

        /// Get position of this TextField.
        pub fn position(&self) -> Vector3<f32> {
            self.display_object.position()
        }

        /// Get size.
        pub fn size(&self) -> Vector2<f32> {
            self.properties.size
        }

        /// Scroll text by given offset in pixels.
        pub fn scroll(&mut self, offset:Vector2<f32>) {
            let position_change = -Vector3::new(offset.x,offset.y,0.0);
            self.rendered.display_object.mod_position(|pos| *pos += position_change );
            let mut update = self.assignment_update();
            if offset.x != 0.0 {
                update.update_after_x_scroll(offset.x);
            }
            if offset.y != 0.0 {
                update.update_line_assignment();
            }
            self.rendered.update_glyphs(&mut self.content);
        }

        /// Get current scroll position.
        pub fn scroll_position(&self) -> Vector2<f32> {
            self.rendered.display_object.position().xy()
        }

        /// Removes all cursors except one which is set and given point.
        pub fn set_cursor(&mut self, point:Vector2<f32>) {
            self.cursors.remove_additional_cursors();
            self.jump_cursor(point,false);
        }

        /// Add cursor at point on the screen.
        pub fn add_cursor(&mut self, point:Vector2<f32>) {
            self.cursors.add_cursor(TextLocation::at_document_begin());
            self.jump_cursor(point,false);
        }

        /// Jump active cursor to point on the screen.
        pub fn jump_cursor(&mut self, point:Vector2<f32>, selecting:bool) {
            let content        = &mut self.content;
            let text_position  = self.rendered.display_object.global_position();
            let point_on_text  = point - text_position.xy();
            let mut navigation = CursorNavigation {content,selecting};
            self.cursors.jump_cursor(&mut navigation,point_on_text);
            self.rendered.update_cursor_sprites(&self.cursors, &mut self.content);
        }

        /// Move all cursors by given step.
        pub fn navigate_cursors(&mut self, step:Step, selecting:bool) {
            let content        = &mut self.content;
            let mut navigation = CursorNavigation {content,selecting};
            self.cursors.navigate_all_cursors(&mut navigation,step);
            self.rendered.update_cursor_sprites(&self.cursors, &mut self.content);
        }

        /// Make change in text content.
        ///
        /// As an opposite to `edit` function, here we don't care about cursors, nor call any
        /// "text changed" callback, just do the change described in `TextChange` structure.
        pub fn apply_change(&mut self, change:TextChange) {
            self.content.apply_change(change);
            self.assignment_update().update_after_text_edit();
            self.rendered.update_glyphs(&mut self.content);
        }

        /// Obtains the whole text content as a single String.
        pub fn get_content(&self) -> String {
            let mut line_strings = self.content.lines.iter().map(|l| l.to_string());
            line_strings.join("\n")
        }

        /// Get the selected text.
        pub fn get_selected_text(&self) -> String {
            let cursor_select  = |c:&Cursor| self.content.copy_fragment(c.selection_range());
            let mut selections = self.cursors.cursors.iter().map(cursor_select);
            selections.join("\n")
        }

        /// Update underlying Display Object.
        pub fn update(&self) {
            self.display_object.update()
        }

        /// Check if given point on screen is inside this TextField.
        pub fn is_inside(&self, point:Vector2<f32>) -> bool {
            let position = self.display_object.global_position();
            let size     = self.properties.size;
            let x_range  = position.x ..= (position.x + size.x);
            let y_range  = (position.y - size.y) ..= position.y;
            x_range.contains(&point.x) && y_range.contains(&point.y)
        }

        /// Set text edit callback.
        ///
        /// This callback will one each `write` function call and all functions using it. That's
        /// include all edits being an effect of keyboard or mouse event.
        pub fn set_text_edit_callback(&mut self, callback:CallbackMut1<TextChangedNotification>) {
            self.text_change_callback = Some(callback)
        }
    }
}


// === Constructor ===

impl TextField {
    /// Create new empty TextField
    pub fn new(world:&World, properties:TextFieldProperties) -> Self {
        Self::new_with_content(world,"",properties)
    }

    /// Create new TextField with predefined content.
    pub fn new_with_content(world:&World, initial_content:&str, properties:TextFieldProperties)
    -> Self {
        let data = TextFieldData::new(world,initial_content,properties);
        let rc   = Rc::new(RefCell::new(data));
        let this = Self {rc};
        let frp  = TextFieldFrp::new(world,this.downgrade());
        this.with_borrowed(move |mut data| { data.frp = Some(frp); });
        this
    }
}


// === Editing text ===

impl TextField {
    /// Edit text.
    ///
    /// All the currently selected text will be removed, and the given string will be inserted
    /// by each cursor.
    pub fn write(&self, text:&str) {
        let trimmed                 = text.trim_end_matches('\n');
        let cursor_ids              = self.with_borrowed(|this| this.cursors.sorted_cursor_indices());
        let is_line_per_cursor_edit = trimmed.contains('\n') && cursor_ids.len() > 1;

        if is_line_per_cursor_edit {
            let cursor_with_line = cursor_ids.iter().cloned().zip(trimmed.split('\n'));
            self.write_per_cursor(cursor_with_line);
        } else {
            let cursor_with_line = cursor_ids.iter().map(|cursor_id| (*cursor_id,text));
            self.write_per_cursor(cursor_with_line);
        };
        self.with_borrowed(|this| {
            this.assignment_update().update_after_text_edit();
            this.rendered.update_glyphs(&mut this.content);
            this.rendered.update_cursor_sprites(&this.cursors, &mut this.content);
        });
    }

    /// Discards all current content and replaces it with new one.
    /// Whenever possible, tries to maintain cursor positions.
    pub fn set_content(&mut self, text:&str) {
        // FIXME [mwu] This is a provisional stub to allow `TextEditor` use
        //       proper API. This implementation should correctly remove old
        //       contents and update the cursors.
        //       See: https://github.com/luna/ide/issues/187
        self.write(text)
    }

    /// Remove all text selected by all cursors.
    pub fn remove_selection(&self) {
        self.write("");
    }

    /// Do delete operation on text.
    ///
    /// For cursors with selection it will just remove the selected text. For the rest, it will
    /// remove all content covered by `step`.
    pub fn do_delete_operation(&self, step:Step) {
        self.with_borrowed(|this| {
            let content           = &mut this.content;
            let selecting         = true;
            let mut navigation    = CursorNavigation {content,selecting};
            let without_selection = |c:&Cursor| !c.has_selection();
            this.cursors.navigate_cursors(&mut navigation,step,without_selection);
        });
        self.remove_selection();
    }
}


// === Private ===

impl TextField {

    fn write_per_cursor<'a,It>(&self, cursor_id_with_text_to_insert:It)
        where It : Iterator<Item=(usize,&'a str)> {
        let mut location_change = TextLocationChange::default();
        let mut opt_callback    = self.with_borrowed(|this| std::mem::take(&mut this.text_change_callback));
        for (cursor_id,to_insert) in cursor_id_with_text_to_insert {
            let notification = self.with_borrowed(|this| {
                let cursor   = &mut this.cursors.cursors[cursor_id];
                let replaced = location_change.apply_to_range(cursor.selection_range());
                let change   = TextChange::replace(replaced.clone(),to_insert);
                location_change.add_change(&change);
                *cursor = Cursor::new(change.inserted_text_range().end);
                this.content.apply_change(change.clone());
                TextChangedNotification {
                    replaced_range: replaced.clone(),
                    replaced_range_char: this.content.convert_location_range_to_char_index(replaced),
                    inserted_string: to_insert.to_string(),
                }
            });
            if let Some(callback) = opt_callback.as_mut() {
                callback(&notification);
            }
        }
        self.with_borrowed(|this| {
            if this.text_change_callback.is_none() {
                this.text_change_callback = opt_callback
            }
        });
    }
}

impl TextFieldData {
    fn new(world:&World, initial_content:&str, properties:TextFieldProperties) -> Self {
        let logger               = Logger::new("TextField");
        let display_object       = DisplayObjectData::new(logger);
        let content              = TextFieldContent::new(initial_content,&properties);
        let cursors              = Cursors::default();
        let rendered             = TextFieldSprites::new(world,&properties);
        let frp                  = None;
        let text_change_callback = None;
        display_object.add_child(rendered.display_object.clone_ref());

        Self {properties,content,cursors,rendered,display_object,frp,text_change_callback}
            .initialize()
    }

    fn initialize(mut self) -> Self{
        self.assignment_update().update_line_assignment();
        self.rendered.update_glyphs(&mut self.content);
        self.rendered.update_cursor_sprites(&self.cursors, &mut self.content);
        self
    }

    fn assignment_update(&mut self) -> GlyphLinesAssignmentUpdate {
        GlyphLinesAssignmentUpdate {
            content       : &mut self.content,
            assignment    : &mut self.rendered.assignment,
            scroll_offset : -self.rendered.display_object.position().xy(),
            view_size     : self.properties.size,
        }
    }
}

// === Display Object ===

impl From<&TextField> for DisplayObjectData {
    fn from(text_fields: &TextField) -> Self {
        text_fields.rc.borrow().display_object.clone_ref()
    }
}
