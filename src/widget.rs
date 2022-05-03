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

//! Display information and interactive controls in your application.

/// A container that distributes its contents vertically.
pub type Column<'a, Message> =
    iced_native::widget::Column<'a, Message, crate::Renderer>;

/// A container that distributes its contents horizontally.
pub type Row<'a, Message> =
    iced_native::widget::Row<'a, Message, crate::Renderer>;

/// A paragraph of text.
pub type Text = iced_native::widget::Text<crate::Renderer>;

pub mod button {
    //! Allow your users to perform actions by pressing a button.
    //!
    //! A [`Button`] has some local [`State`].
    pub use iced_native::widget::button::{State, Style, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Button<'a, Message> =
        iced_native::widget::Button<'a, Message, crate::Renderer>;
}

pub mod checkbox {
    //! Show toggle controls using checkboxes.
    pub use iced_native::widget::checkbox::{Style, StyleSheet};

    /// A box that can be checked.
    pub type Checkbox<'a, Message> =
        iced_native::widget::Checkbox<'a, Message, crate::Renderer>;
}

pub mod container {
    //! Decorate content and apply alignment.
    pub use iced_native::widget::container::{Style, StyleSheet};

    /// An element decorating some content.
    pub type Container<'a, Message> =
        iced_native::widget::Container<'a, Message, crate::Renderer>;
}

pub mod pane_grid {
    //! Let your users split regions of your application and organize layout dynamically.
    //!
    //! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    //!
    //! # Example
    //! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
    //! drag and drop, and hotkey support.
    //!
    //! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.4/examples/pane_grid
    pub use iced_native::widget::pane_grid::{
        Axis, Configuration, Direction, DragEvent, Line, Node, Pane,
        ResizeEvent, Split, State, StyleSheet,
    };

    /// A collection of panes distributed using either vertical or horizontal splits
    /// to completely fill the space available.
    ///
    /// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
    pub type PaneGrid<'a, Message> =
        iced_native::widget::PaneGrid<'a, Message, crate::Renderer>;

    /// The content of a [`Pane`].
    pub type Content<'a, Message> =
        iced_native::widget::pane_grid::Content<'a, Message, crate::Renderer>;

    /// The title bar of a [`Pane`].
    pub type TitleBar<'a, Message> =
        iced_native::widget::pane_grid::TitleBar<'a, Message, crate::Renderer>;
}

pub mod pick_list {
    //! Display a dropdown list of selectable values.
    pub use iced_native::overlay::menu::Style as Menu;
    pub use iced_native::widget::pick_list::{State, Style, StyleSheet};

    /// A widget allowing the selection of a single value from a list of options.
    pub type PickList<'a, T, Message> =
        iced_native::widget::PickList<'a, T, Message, crate::Renderer>;
}

pub mod radio {
    //! Create choices using radio buttons.
    pub use iced_native::widget::radio::{Style, StyleSheet};

    /// A circular button representing a choice.
    pub type Radio<'a, Message> =
        iced_native::widget::Radio<'a, Message, crate::Renderer>;
}

pub mod scrollable {
    //! Navigate an endless amount of content with a scrollbar.
    pub use iced_native::widget::scrollable::{
        style::Scrollbar, style::Scroller, State, StyleSheet,
    };

    /// A widget that can vertically display an infinite amount of content
    /// with a scrollbar.
    pub type Scrollable<'a, Message> =
        iced_native::widget::Scrollable<'a, Message, crate::Renderer>;
}

pub mod toggler {
    //! Show toggle controls using togglers.
    pub use iced_native::widget::toggler::{Style, StyleSheet};

    /// A toggler widget.
    pub type Toggler<'a, Message> =
        iced_native::widget::Toggler<'a, Message, crate::Renderer>;
}

pub mod text_input {
    //! Display fields that can be filled with text.
    //!
    //! A [`TextInput`] has some local [`State`].
    use crate::Renderer;

    pub use iced_native::widget::text_input::{State, Style, StyleSheet};

    /// A field that can be filled with text.
    pub type TextInput<'a, Message> =
        iced_native::widget::TextInput<'a, Message, Renderer>;
}

pub mod tooltip {
    //! Display a widget over another.
    pub use iced_native::widget::tooltip::Position;

    /// A widget allowing the selection of a single value from a list of options.
    pub type Tooltip<'a, Message> =
        iced_native::widget::Tooltip<'a, Message, crate::Renderer>;
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
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tooltip::Tooltip;

#[cfg(any(feature = "wgpu_canvas", feature = "glow_canvas"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "wgpu_canvas", feature = "glow_canvas"))))]
pub use iced_graphics::widget::canvas;

#[cfg(feature = "wgpu_image")]
#[cfg_attr(docsrs, doc(cfg(feature = "wgpu_image")))]
pub mod image {
    //! Display images in your user interface.
    pub use iced_native::image::Handle;

    /// A frame that displays an image.
    pub type Image = iced_native::widget::Image<Handle>;

    pub use iced_native::widget::image::viewer;
    pub use viewer::Viewer;
}

#[cfg(any(feature = "wgpu_qr_code", feature = "glow_qr_code"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "wgpu_qr_code", feature = "glow_qr_code"))))]
pub use iced_graphics::widget::qr_code;

#[cfg(feature = "wgpu_svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "wgpu_svg")))]
pub mod svg {
    //! Display vector graphics in your application.
    pub use iced_native::svg::Handle;
    pub use iced_native::widget::Svg;
}

#[cfg(any())]
#[cfg_attr(docsrs, doc(cfg(feature = "wgpu_canvas", feature = "glow_canvas")))]
pub use canvas::Canvas;

#[cfg(feature = "wgpu_image")]
#[cfg_attr(docsrs, doc(cfg(feature = "wgpu_image")))]
pub use image::Image;

#[cfg(any(feature = "wgpu_qr_code", feature = "glow_qr_code"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "wgpu_qr_code", feature = "glow_qr_code"))))]
pub use qr_code::QRCode;

#[cfg(feature = "wgpu_svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "wgpu_svg")))]
pub use svg::Svg;