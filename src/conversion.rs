use baseview::Event as BaseEvent;
use iced_core::Point;
use iced_native::keyboard::Event as IcedKeyEvent;
use iced_native::keyboard::Modifiers as IcedModifiers;
use iced_native::mouse::Button as IcedMouseButton;
use iced_native::mouse::Event as IcedMouseEvent;
use iced_native::window::Event as IcedWindowEvent;
use iced_native::Event as IcedEvent;
use keyboard_types::Modifiers;

pub fn baseview_to_iced_events(
    event: BaseEvent,
    iced_events: &mut Vec<IcedEvent>,
    iced_modifiers: &mut IcedModifiers,
    ignore_non_modifier_keys: bool,
) {
    let mut maybe_update_modifiers = |modifiers: Modifiers| {
        let old_modifiers = *iced_modifiers;

        iced_modifiers
            .set(IcedModifiers::SHIFT, modifiers.contains(Modifiers::SHIFT));
        iced_modifiers
            .set(IcedModifiers::CTRL, modifiers.contains(Modifiers::CONTROL));
        iced_modifiers
            .set(IcedModifiers::ALT, modifiers.contains(Modifiers::ALT));
        iced_modifiers
            .set(IcedModifiers::LOGO, modifiers.contains(Modifiers::META));
        if *iced_modifiers != old_modifiers {
            iced_events.push(IcedEvent::Keyboard(
                iced_native::keyboard::Event::ModifiersChanged(*iced_modifiers),
            ));
        }
    };

    match event {
        BaseEvent::Mouse(mouse_event) => {
            match mouse_event {
                baseview::MouseEvent::CursorMoved {
                    position,
                    modifiers,
                } => {
                    maybe_update_modifiers(modifiers);
                    iced_events.push(IcedEvent::Mouse(
                        IcedMouseEvent::CursorMoved {
                            position: Point::new(
                                position.x as f32,
                                position.y as f32,
                            ),
                        },
                    ));
                }
                baseview::MouseEvent::ButtonPressed { button, modifiers } => {
                    maybe_update_modifiers(modifiers);
                    iced_events.push(IcedEvent::Mouse(
                        IcedMouseEvent::ButtonPressed(
                            baseview_mouse_button_to_iced(button),
                        ),
                    ));
                }
                baseview::MouseEvent::ButtonReleased { button, modifiers } => {
                    maybe_update_modifiers(modifiers);
                    iced_events.push(IcedEvent::Mouse(
                        IcedMouseEvent::ButtonReleased(
                            baseview_mouse_button_to_iced(button),
                        ),
                    ));
                }
                baseview::MouseEvent::WheelScrolled { delta, modifiers } => {
                    maybe_update_modifiers(modifiers);
                    match delta {
                        baseview::ScrollDelta::Lines { x, y } => {
                            iced_events.push(IcedEvent::Mouse(
                                IcedMouseEvent::WheelScrolled {
                                    delta:
                                        iced_native::mouse::ScrollDelta::Lines {
                                            x,
                                            y,
                                        },
                                },
                            ));
                        }
                        baseview::ScrollDelta::Pixels { x, y } => {
                            iced_events.push(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                            delta: iced_native::mouse::ScrollDelta::Pixels {
                                x,
                                y,
                            },
                        }));
                        }
                    }
                }
                _ => {}
            }
        }

        BaseEvent::Keyboard(event) => {
            use keyboard_types::Code;

            let is_down = match event.state {
                keyboard_types::KeyState::Down => true,
                keyboard_types::KeyState::Up => false,
            };

            // TODO: Remove manual setting of modifiers once the issue
            // is fixed in baseview.
            let is_modifier = match event.code {
                Code::AltLeft | Code::AltRight => {
                    iced_modifiers.set(IcedModifiers::ALT, is_down);
                    true
                }
                Code::ControlLeft | Code::ControlRight => {
                    iced_modifiers.set(IcedModifiers::COMMAND, is_down);
                    true
                }
                Code::ShiftLeft | Code::ShiftRight => {
                    iced_modifiers.set(IcedModifiers::SHIFT, is_down);
                    true
                }
                Code::MetaLeft | Code::MetaRight => {
                    iced_modifiers.set(IcedModifiers::LOGO, is_down);
                    true
                }
                _ => false,
            };
            if is_modifier {
                iced_events.push(IcedEvent::Keyboard(
                    iced_native::keyboard::Event::ModifiersChanged(
                        *iced_modifiers,
                    ),
                ));
            }

            if ignore_non_modifier_keys {
                return;
            }

            let opt_key_code = baseview_to_iced_keycode(event.code);

            if is_down {
                if let Some(key_code) = opt_key_code {
                    iced_events.push(IcedEvent::Keyboard(
                        IcedKeyEvent::KeyPressed {
                            key_code,
                            modifiers: *iced_modifiers,
                        },
                    ));
                }

                if let keyboard_types::Key::Character(written) = event.key {
                    for chr in written.chars() {
                        iced_events.push(IcedEvent::Keyboard(
                            IcedKeyEvent::CharacterReceived(chr),
                        ));
                    }
                }
            } else if let Some(key_code) = opt_key_code {
                iced_events.push(IcedEvent::Keyboard(
                    IcedKeyEvent::KeyReleased {
                        key_code,
                        modifiers: *iced_modifiers,
                    },
                ));
            }
        }

        BaseEvent::Window(window_event) => match window_event {
            baseview::WindowEvent::Resized(window_info) => {
                iced_events.push(IcedEvent::Window(IcedWindowEvent::Resized {
                    width: window_info.logical_size().width as u32,
                    height: window_info.logical_size().height as u32,
                }));
            }
            baseview::WindowEvent::Unfocused => {
                *iced_modifiers = IcedModifiers::empty();
            }
            _ => {}
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
/// Converts a physical cursor position to a logical `Point`.
pub fn cursor_position(position: PhyPoint, scale_factor: f64) -> Point {
    Point::new(
        (f64::from(position.x) * scale_factor) as f32,
        (f64::from(position.y) * scale_factor) as f32,
    )
}
*/

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

fn baseview_to_iced_keycode(
    code: keyboard_types::Code,
) -> Option<iced_core::keyboard::KeyCode> {
    use iced_core::keyboard::KeyCode as ICode;
    use keyboard_types::Code as KCode;

    match code {
        KCode::Digit1 => Some(ICode::Key1),
        KCode::Digit2 => Some(ICode::Key2),
        KCode::Digit3 => Some(ICode::Key3),
        KCode::Digit4 => Some(ICode::Key4),
        KCode::Digit5 => Some(ICode::Key5),
        KCode::Digit6 => Some(ICode::Key6),
        KCode::Digit7 => Some(ICode::Key7),
        KCode::Digit8 => Some(ICode::Key8),
        KCode::Digit9 => Some(ICode::Key9),
        KCode::Digit0 => Some(ICode::Key0),

        KCode::KeyA => Some(ICode::A),
        KCode::KeyB => Some(ICode::B),
        KCode::KeyC => Some(ICode::C),
        KCode::KeyD => Some(ICode::D),
        KCode::KeyE => Some(ICode::E),
        KCode::KeyF => Some(ICode::F),
        KCode::KeyG => Some(ICode::G),
        KCode::KeyH => Some(ICode::H),
        KCode::KeyI => Some(ICode::I),
        KCode::KeyJ => Some(ICode::J),
        KCode::KeyK => Some(ICode::K),
        KCode::KeyL => Some(ICode::L),
        KCode::KeyM => Some(ICode::M),
        KCode::KeyN => Some(ICode::N),
        KCode::KeyO => Some(ICode::O),
        KCode::KeyP => Some(ICode::P),
        KCode::KeyQ => Some(ICode::Q),
        KCode::KeyR => Some(ICode::R),
        KCode::KeyS => Some(ICode::S),
        KCode::KeyT => Some(ICode::T),
        KCode::KeyU => Some(ICode::U),
        KCode::KeyV => Some(ICode::V),
        KCode::KeyW => Some(ICode::W),
        KCode::KeyX => Some(ICode::X),
        KCode::KeyY => Some(ICode::Y),
        KCode::KeyZ => Some(ICode::Z),

        KCode::Escape => Some(ICode::Escape),

        KCode::F1 => Some(ICode::F1),
        KCode::F2 => Some(ICode::F2),
        KCode::F3 => Some(ICode::F3),
        KCode::F4 => Some(ICode::F4),
        KCode::F5 => Some(ICode::F5),
        KCode::F6 => Some(ICode::F6),
        KCode::F7 => Some(ICode::F7),
        KCode::F8 => Some(ICode::F8),
        KCode::F9 => Some(ICode::F9),
        KCode::F10 => Some(ICode::F10),
        KCode::F11 => Some(ICode::F11),
        KCode::F12 => Some(ICode::F12),

        KCode::PrintScreen => Some(ICode::Snapshot),
        KCode::ScrollLock => Some(ICode::Scroll),
        KCode::Pause => Some(ICode::Pause),

        KCode::Insert => Some(ICode::Insert),
        KCode::Home => Some(ICode::Home),
        KCode::Delete => Some(ICode::Delete),
        KCode::End => Some(ICode::End),
        KCode::PageDown => Some(ICode::PageDown),
        KCode::PageUp => Some(ICode::PageUp),

        KCode::ArrowLeft => Some(ICode::Left),
        KCode::ArrowUp => Some(ICode::Up),
        KCode::ArrowRight => Some(ICode::Right),
        KCode::ArrowDown => Some(ICode::Down),

        KCode::Backspace => Some(ICode::Backspace),
        KCode::Enter => Some(ICode::Enter),
        KCode::Space => Some(ICode::Space),

        KCode::NumLock => Some(ICode::Numlock),
        KCode::Numpad0 => Some(ICode::Numpad0),
        KCode::Numpad1 => Some(ICode::Numpad1),
        KCode::Numpad2 => Some(ICode::Numpad2),
        KCode::Numpad3 => Some(ICode::Numpad3),
        KCode::Numpad4 => Some(ICode::Numpad4),
        KCode::Numpad5 => Some(ICode::Numpad5),
        KCode::Numpad6 => Some(ICode::Numpad6),
        KCode::Numpad7 => Some(ICode::Numpad7),
        KCode::Numpad8 => Some(ICode::Numpad8),
        KCode::Numpad9 => Some(ICode::Numpad9),
        KCode::NumpadAdd => Some(ICode::NumpadAdd),
        KCode::NumpadDivide => Some(ICode::NumpadDivide),
        KCode::NumpadDecimal => Some(ICode::NumpadDecimal),
        KCode::NumpadComma => Some(ICode::NumpadComma),
        KCode::NumpadEnter => Some(ICode::NumpadEnter),
        KCode::NumpadEqual => Some(ICode::NumpadEquals),
        KCode::NumpadMultiply => Some(ICode::NumpadMultiply),
        KCode::NumpadSubtract => Some(ICode::NumpadSubtract),

        //KCode::AbntC1 => Some(ICode::AbntC1),    // TODO ?
        //KCode::AbntC1 => Some(ICode::AbntC1),    // TODO ?
        KCode::Convert => Some(ICode::Convert),
        KCode::KanaMode => Some(ICode::Kana),
        //KCode::Kanji => ICode::Kanji),    // TODO ?
        KCode::NonConvert => Some(ICode::NoConvert),
        KCode::IntlYen => Some(ICode::Yen),

        KCode::AltLeft => Some(ICode::LAlt),
        KCode::AltRight => Some(ICode::RAlt),
        KCode::BracketLeft => Some(ICode::LBracket),
        KCode::BracketRight => Some(ICode::RBracket),
        KCode::ControlLeft => Some(ICode::LControl),
        KCode::ControlRight => Some(ICode::RControl),
        KCode::ShiftLeft => Some(ICode::LShift),
        KCode::ShiftRight => Some(ICode::RShift),
        KCode::MetaLeft => Some(ICode::LWin),
        KCode::MetaRight => Some(ICode::RWin),

        KCode::Minus => Some(ICode::Minus),
        KCode::Period => Some(ICode::Period),
        //KCode::Plus => Some(ICode::Plus),    // TODO ?
        KCode::Equal => Some(ICode::Equals),
        KCode::Quote => Some(ICode::Apostrophe),
        KCode::Comma => Some(ICode::Comma),
        //KCode::Grave => Some(ICode::Grave),    // TODO ?
        //KCode::Colon => Some(ICode::Colon),    // TODO ?
        KCode::Semicolon => Some(ICode::Semicolon),
        KCode::Backslash => Some(ICode::Backslash),
        KCode::Slash => Some(ICode::Slash),
        KCode::Tab => Some(ICode::Tab),
        //KCode::Underline => Some(ICode::Underline),    // TODO ?
        KCode::Copy => Some(ICode::Copy),
        KCode::Paste => Some(ICode::Paste),
        KCode::Cut => Some(ICode::Cut),

        KCode::MediaSelect => Some(ICode::MediaSelect),
        KCode::MediaStop => Some(ICode::MediaStop),
        KCode::MediaPlayPause => Some(ICode::PlayPause),
        KCode::AudioVolumeMute => Some(ICode::Mute),
        KCode::AudioVolumeDown => Some(ICode::VolumeDown),
        KCode::AudioVolumeUp => Some(ICode::VolumeUp),
        KCode::MediaTrackNext => Some(ICode::NextTrack),
        KCode::MediaTrackPrevious => Some(ICode::PrevTrack),

        _ => None,
    }
}
