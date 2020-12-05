use baseview::Event as BaseEvent;
use iced_native::keyboard::Event as IcedKeyEvent;
use iced_native::keyboard::Modifiers as IcedModifiers;
use iced_native::mouse::Button as IcedMouseButton;
use iced_native::mouse::Event as IcedMouseEvent;
use iced_native::window::Event as IcedWindowEvent;
use iced_native::Event as IcedEvent;

pub fn baseview_to_iced_event(event: BaseEvent) -> Option<IcedEvent> {
    match event {
        BaseEvent::Mouse(mouse_event) => match mouse_event {
            baseview::MouseEvent::CursorMoved { position } => {
                Some(IcedEvent::Mouse(IcedMouseEvent::CursorMoved {
                    x: position.x as f32,
                    y: position.y as f32,
                }))
            }
            baseview::MouseEvent::ButtonPressed(button) => {
                Some(IcedEvent::Mouse(IcedMouseEvent::ButtonPressed(
                    baseview_mouse_button_to_iced(button),
                )))
            }
            baseview::MouseEvent::ButtonReleased(button) => {
                Some(IcedEvent::Mouse(IcedMouseEvent::ButtonReleased(
                    baseview_mouse_button_to_iced(button),
                )))
            }
            baseview::MouseEvent::WheelScrolled(scroll_delta) => {
                match scroll_delta {
                    baseview::ScrollDelta::Lines { x, y } => {
                        Some(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                            delta: iced_native::mouse::ScrollDelta::Lines {
                                x,
                                y,
                            },
                        }))
                    }
                    baseview::ScrollDelta::Pixels { x, y } => {
                        Some(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                            delta: iced_native::mouse::ScrollDelta::Pixels {
                                x,
                                y,
                            },
                        }))
                    }
                }
            }
            _ => None,
        },

        BaseEvent::Keyboard(event) => {
            use keyboard_types::Modifiers;

            let modifiers = IcedModifiers {
                shift: event.modifiers.contains(Modifiers::SHIFT),
                control: event.modifiers.contains(Modifiers::CONTROL),
                alt: event.modifiers.contains(Modifiers::ALT),
                logo: event.modifiers.contains(Modifiers::META),
            };

            use keyboard_types::Code;
            use iced_core::keyboard::KeyCode;

            let opt_key_code = match event.code {
                Code::ShiftLeft => Some(KeyCode::LShift),
                Code::ControlLeft => Some(KeyCode::LControl),
                Code::AltLeft => Some(KeyCode::LAlt),
                Code::MetaLeft => Some(KeyCode::LWin),
                Code::ShiftRight => Some(KeyCode::RShift),
                Code::ControlRight => Some(KeyCode::RControl),
                Code::AltRight => Some(KeyCode::RAlt),
                Code::MetaRight => Some(KeyCode::RWin),
                _ => None,
            };

            opt_key_code.map(|key_code| {
                let key_event = match event.state {
                    keyboard_types::KeyState::Down => {
                        IcedKeyEvent::KeyPressed {
                            key_code,
                            modifiers,
                        }
                    }
                    keyboard_types::KeyState::Up => {
                        IcedKeyEvent::KeyReleased {
                            key_code,
                            modifiers,
                        }
                    }
                };

                IcedEvent::Keyboard(key_event)
            })
        },

        BaseEvent::Window(window_event) => match window_event {
            baseview::WindowEvent::Resized(window_info) => {
                Some(IcedEvent::Window(IcedWindowEvent::Resized {
                    width: window_info.logical_size().width as u32,
                    height: window_info.logical_size().height as u32,
                }))
            }
            _ => None,
        },
    }
}

fn baseview_mouse_button_to_iced(id: baseview::MouseButton) -> IcedMouseButton {
    use baseview::MouseButton;

    match id {
        MouseButton::Left => IcedMouseButton::Left,
        MouseButton::Middle => IcedMouseButton::Middle,
        MouseButton::Right => IcedMouseButton::Right,
        MouseButton::Back => IcedMouseButton::Other(6),
        MouseButton::Forward => IcedMouseButton::Other(7),
        MouseButton::Other(other_id) => IcedMouseButton::Other(other_id),
    }
}

/*
// As defined in: http://www.unicode.org/faq/private_use.html
fn is_private_use_character(c: char) -> bool {
    match c {
        '\u{E000}'..='\u{F8FF}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}' => true,
        _ => false,
    }
}
*/
