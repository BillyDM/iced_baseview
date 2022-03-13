/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message> =
    iced_native::Element<'a, Message, crate::backend::Renderer>;
