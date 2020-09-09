use baseview::Event as BaseEvent;
use iced_native::keyboard::Event as IcedKeyEvent;
use iced_native::keyboard::ModifiersState as IcedModifiersState;
use iced_native::mouse::Button as IcedMouseButton;
use iced_native::mouse::Event as IcedMouseEvent;
use iced_native::window::Event as IcedWindowEvent;
use iced_native::Event as IcedEvent;

pub fn baseview_to_iced_event(event: BaseEvent) -> Option<IcedEvent> {
    match event {
        BaseEvent::CursorMotion(x, y) => {
            //println!("Cursor moved, x: {}, y: {}", x, y);

            Some(IcedEvent::Mouse(IcedMouseEvent::CursorMoved {
                x: x as f32,
                y: y as f32,
            }))
        }
        BaseEvent::MouseDown(button_id) => {
            //println!("Mouse down, button id: {:?}", button_id);

            Some(IcedEvent::Mouse(IcedMouseEvent::ButtonPressed(
                baseview_mouse_button_to_iced(button_id),
            )))
        }
        BaseEvent::MouseUp(button_id) => {
            //println!("Mouse up, button id: {:?}", button_id);

            Some(IcedEvent::Mouse(IcedMouseEvent::ButtonPressed(
                baseview_mouse_button_to_iced(button_id),
            )))
        }
        BaseEvent::MouseScroll(mouse_scroll) => {
            //println!("Mouse scroll, {:?}", mouse_scroll);

            Some(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                delta: iced_native::mouse::ScrollDelta::Lines {
                    x: mouse_scroll.x_delta as f32,
                    y: mouse_scroll.y_delta as f32,
                },
            }))
        }
        BaseEvent::MouseClick(mouse_click) => {
            //println!("Mouse click, {:?}", mouse_click);

            None
        }
        BaseEvent::KeyDown(keycode) => {
            //println!("Key down, keycode: {}", keycode);

            // We need to map keycodes in baseview

            None
        }
        BaseEvent::KeyUp(keycode) => {
            //println!("Key up, keycode: {}", keycode);

            // We need to map keycodes in baseview

            None
        }
        BaseEvent::CharacterInput(char_code) => {
            //println!("Character input, char_code: {}", char_code);

            let char_code = (char_code as u8) as char;

            if is_private_use_character(char_code) {
                None
            } else {
                Some(IcedEvent::Keyboard(IcedKeyEvent::CharacterReceived(
                    char_code,
                )))
            }
        }
        BaseEvent::WindowResized(window_info) => {
            //println!("Window resized, {:?}", window_info);

            Some(IcedEvent::Window(IcedWindowEvent::Resized {
                width: window_info.width,
                height: window_info.height,
            }))
        }
        BaseEvent::WindowFocus => {
            //println!("Window focused");

            None
        }
        BaseEvent::WindowUnfocus => {
            //println!("Window unfocused");

            None
        }
        BaseEvent::WillClose => {
            //println!("Window will close");

            None
        }
    }
}

fn baseview_mouse_button_to_iced(
    id: baseview::MouseButtonID,
) -> IcedMouseButton {
    use baseview::MouseButtonID;

    match id {
        MouseButtonID::Left => IcedMouseButton::Left,
        MouseButtonID::Middle => IcedMouseButton::Middle,
        MouseButtonID::Right => IcedMouseButton::Right,
        MouseButtonID::Back => IcedMouseButton::Other(6),
        MouseButtonID::Forward => IcedMouseButton::Other(7),
        MouseButtonID::Other(other_id) => IcedMouseButton::Other(other_id),
    }
}

// As defined in: http://www.unicode.org/faq/private_use.html
fn is_private_use_character(c: char) -> bool {
    match c {
        '\u{E000}'..='\u{F8FF}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}' => true,
        _ => false,
    }
}
