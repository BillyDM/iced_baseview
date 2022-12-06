// Copied from https://github.com/iced-rs/iced/blob/master/src/widget.rs
// (commit 84c28a7) with minor adjustments.
//
// Copyright 2019 Héctor Ramón, Iced contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Display information and interactive controls in your crate::Application.
//!

/// A container that distributes its contents vertically.
pub type Column<'a, Message, Theme> =
    iced_native::widget::Column<'a, Message, crate::Renderer<Theme>>;

/// A container that distributes its contents horizontally.
pub type Row<'a, Message, Theme> = iced_native::widget::Row<'a, Message, crate::Renderer<Theme>>;

pub mod application {
    pub use iced_native::application::{Appearance, StyleSheet};
}

pub mod button {
    //! Allow your users to perform actions by pressing a button.
    //!
    //! A [`Button`] has some local [`State`].
    pub use iced_native::widget::button::{Appearance, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Button<'a, Message, Theme> =
        iced_native::widget::Button<'a, Message, crate::Renderer<Theme>>;
}

pub mod checkbox {
    //! Show toggle controls using checkboxes.
    pub use iced_native::widget::checkbox::{Appearance, StyleSheet};

    /// A box that can be checked.
    pub type Checkbox<'a, Message, Theme> =
        iced_native::widget::Checkbox<'a, Message, crate::Renderer<Theme>>;
}

pub mod container {
    //! Decorate content and apply alignment.
    pub use iced_native::widget::container::{Appearance, StyleSheet};

    /// An element decorating some content.
    pub type Container<'a, Message, Theme> =
        iced_native::widget::Container<'a, Message, crate::Renderer<Theme>>;
}

pub mod pane_grid {
    //! Let your users split regions of your crate::Application and organize layout dynamically.
    //!
    //! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    //!
    //! # Example
    //! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
    //! drag and drop, and hotkey support.
    //!
    //! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.4/examples/pane_grid
    pub use iced_native::widget::pane_grid::{
        Axis, Configuration, Direction, DragEvent, Line, Node, Pane, ResizeEvent, Split, State,
        StyleSheet,
    };

    /// A collection of panes distributed using either vertical or horizontal splits
    /// to completely fill the space available.
    ///
    /// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    pub type PaneGrid<'a, Message, Theme> =
        iced_native::widget::PaneGrid<'a, Message, crate::Renderer<Theme>>;

    /// The content of a [`Pane`].
    pub type Content<'a, Message, Theme> =
        iced_native::widget::pane_grid::Content<'a, Message, crate::Renderer<Theme>>;

    /// The title bar of a [`Pane`].
    pub type TitleBar<'a, Message, Theme> =
        iced_native::widget::pane_grid::TitleBar<'a, Message, crate::Renderer<Theme>>;
}

pub mod pick_list {
    //! Display a dropdown list of selectable values.
    pub use iced_native::widget::pick_list::{Appearance, StyleSheet};

    /// A widget allowing the selection of a single value from a list of options.
    pub type PickList<'a, T, Message, Theme> =
        iced_native::widget::PickList<'a, T, Message, crate::Renderer<Theme>>;
}

pub mod radio {
    //! Create choices using radio buttons.
    pub use iced_native::widget::radio::{Appearance, StyleSheet};

    /// A circular button representing a choice.
    pub type Radio<Message, Theme> = iced_native::widget::Radio<Message, crate::Renderer<Theme>>;
}

pub mod scrollable {
    //! Navigate an endless amount of content with a scrollbar.
    pub use iced_native::widget::scrollable::{style::Scrollbar, style::Scroller, StyleSheet};

    /// A widget that can vertically display an infinite amount of content
    /// with a scrollbar.
    pub type Scrollable<'a, Message, Theme> =
        iced_native::widget::Scrollable<'a, Message, crate::Renderer<Theme>>;
}

pub mod toggler {
    //! Show toggle controls using togglers.
    pub use iced_native::widget::toggler::{Appearance, StyleSheet};

    /// A toggler widget.
    pub type Toggler<'a, Message, Theme> =
        iced_native::widget::Toggler<'a, Message, crate::Renderer<Theme>>;
}

pub mod text {
    pub use iced_native::widget::text::{Appearance, StyleSheet};

    /// A paragraph of text.
    pub type Text<'a, Theme> = iced_native::widget::Text<'a, crate::Renderer<Theme>>;
}

pub mod text_input {
    //! Display fields that can be filled with text.
    //!
    //! A [`TextInput`] has some local [`State`].
    pub use iced_native::widget::text_input::{Appearance, StyleSheet};

    /// A field that can be filled with text.
    pub type TextInput<'a, Message, Theme> =
        iced_native::widget::TextInput<'a, Message, crate::Renderer<Theme>>;
}

pub mod tooltip {
    //! Display a widget over another.
    pub use iced_native::widget::tooltip::Position;

    /// A widget allowing the selection of a single value from a list of options.
    pub type Tooltip<'a, Message, Theme> =
        iced_native::widget::Tooltip<'a, Message, crate::Renderer<Theme>>;
}

pub use iced_native::widget::progress_bar;
pub use iced_native::widget::rule;
pub use iced_native::widget::slider;
pub use iced_native::widget::Space;

pub use button::Button;
pub use checkbox::Checkbox;
pub use container::Container;
pub use pane_grid::PaneGrid;
pub use pick_list::PickList;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use rule::Rule;
pub use scrollable::Scrollable;
pub use slider::Slider;
pub use text::Text;
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tooltip::Tooltip;

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub use iced_graphics::widget::canvas;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub mod image {
    //! Display images in your user interface.
    pub use iced_native::image::Handle;

    /// A frame that displays an image.
    pub type Image = iced_native::widget::Image<Handle>;

    pub use iced_native::widget::image::viewer;
    pub use viewer::Viewer;
}

#[cfg(feature = "qr_code")]
#[cfg_attr(docsrs, doc(cfg(feature = "qr_code")))]
pub use iced_graphics::widget::qr_code;

#[cfg(feature = "svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
pub mod svg {
    //! Display vector graphics in your crate::Application.
    pub use iced_native::svg::Handle;
    pub use iced_native::widget::Svg;
}

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub use canvas::Canvas;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub use image::Image;

#[cfg(feature = "qr_code")]
#[cfg_attr(docsrs, doc(cfg(feature = "qr_code")))]
pub use qr_code::QRCode;

#[cfg(feature = "svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
pub use svg::Svg;
