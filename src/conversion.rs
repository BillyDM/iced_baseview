use baseview::Event as BaseEvent;
use iced_runtime::core::mouse::Button as IcedMouseButton;
use iced_runtime::core::mouse::Event as IcedMouseEvent;
use iced_runtime::core::window::Event as IcedWindowEvent;
use iced_runtime::core::Event as IcedEvent;
use iced_runtime::core::Point;
use iced_runtime::keyboard::Event as IcedKeyEvent;
use iced_runtime::keyboard::Modifiers as IcedModifiers;
use keyboard_types::Modifiers as BaseviewModifiers;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;

pub fn baseview_to_iced_events(
    event: BaseEvent,
    iced_events: &mut Vec<IcedEvent>,
    iced_modifiers: &mut IcedModifiers,
    ignore_non_modifier_keys: bool,
) {
    match event {
        BaseEvent::Mouse(mouse_event) => match mouse_event {
            baseview::MouseEvent::CursorMoved {
                position,
                modifiers,
            } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push(event);
                }
                iced_events.push(IcedEvent::Mouse(IcedMouseEvent::CursorMoved {
                    position: Point::new(position.x as f32, position.y as f32),
                }));
            }
            baseview::MouseEvent::ButtonPressed { button, modifiers } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push(event);
                }
                iced_events.push(IcedEvent::Mouse(IcedMouseEvent::ButtonPressed(
                    baseview_mouse_button_to_iced(button),
                )));
            }
            baseview::MouseEvent::ButtonReleased { button, modifiers } => {
                if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                    iced_events.push(event);
                }
                iced_events.push(IcedEvent::Mouse(IcedMouseEvent::ButtonReleased(
                    baseview_mouse_button_to_iced(button),
                )));
            }
            baseview::MouseEvent::WheelScrolled { delta, modifiers } => match delta {
                baseview::ScrollDelta::Lines { x, y } => {
                    if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                        iced_events.push(event);
                    }
                    iced_events.push(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                        delta: iced_runtime::core::mouse::ScrollDelta::Lines { x, y },
                    }));
                }
                baseview::ScrollDelta::Pixels { x, y } => {
                    if let Some(event) = update_modifiers(iced_modifiers, modifiers) {
                        iced_events.push(event);
                    }
                    iced_events.push(IcedEvent::Mouse(IcedMouseEvent::WheelScrolled {
                        delta: iced_runtime::core::mouse::ScrollDelta::Pixels { x, y },
                    }));
                }
            },
            _ => {}
        },

        BaseEvent::Keyboard(event) => {
            if let Some(event) = update_modifiers(iced_modifiers, event.modifiers) {
                iced_events.push(event);
            }

            if ignore_non_modifier_keys {
                return;
            }

            let is_down = match event.state {
                keyboard_types::KeyState::Down => true,
                keyboard_types::KeyState::Up => false,
            };

            let key = baseview_to_iced_key(event.key);
            let location = baseview_key_location_to_iced(event.location);

            let physical_key = if let Some(code) = baseview_to_iced_keycode(event.code) {
                iced_runtime::core::keyboard::key::Physical::Code(code)
            } else {
                iced_runtime::core::keyboard::key::Physical::Unidentified(
                    iced_runtime::core::keyboard::key::NativeCode::Unidentified,
                )
            };

            if is_down {
                let text = if let iced_runtime::core::keyboard::Key::Character(s) = &key {
                    Some(s.clone())
                } else {
                    None
                };
                iced_events.push(IcedEvent::Keyboard(IcedKeyEvent::KeyPressed {
                    key: key.clone(),
                    modified_key: key,
                    physical_key,
                    modifiers: *iced_modifiers,
                    location,
                    text,
                }));
            } else {
                iced_events.push(IcedEvent::Keyboard(IcedKeyEvent::KeyReleased {
                    key: key.clone(),
                    location,
                    modifiers: *iced_modifiers,
                    modified_key: key,
                    physical_key,
                }));
            }
        }

        BaseEvent::Window(window_event) => match window_event {
            baseview::WindowEvent::Resized(window_info) => {
                iced_events.push(IcedEvent::Window(IcedWindowEvent::Resized(
                    iced_runtime::core::Size {
                        width: window_info.logical_size().width as f32,
                        height: window_info.logical_size().height as f32,
                    },
                )));
            }
            baseview::WindowEvent::Unfocused => {
                *iced_modifiers = IcedModifiers::empty();
            }
            _ => {}
        },
    }
}

fn update_modifiers(
    iced_modifiers: &mut IcedModifiers,
    baseview_modifiers: BaseviewModifiers,
) -> Option<IcedEvent> {
    let mut new = IcedModifiers::default();

    new.set(
        IcedModifiers::ALT,
        baseview_modifiers.contains(BaseviewModifiers::ALT),
    );
    new.set(
        IcedModifiers::CTRL,
        baseview_modifiers.contains(BaseviewModifiers::CONTROL),
    );
    new.set(
        IcedModifiers::SHIFT,
        baseview_modifiers.contains(BaseviewModifiers::SHIFT),
    );
    new.set(
        IcedModifiers::LOGO,
        baseview_modifiers.contains(BaseviewModifiers::META),
    );

    if *iced_modifiers != new {
        *iced_modifiers = new;

        Some(IcedEvent::Keyboard(
            iced_runtime::core::keyboard::Event::ModifiersChanged(*iced_modifiers),
        ))
    } else {
        None
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
        MouseButton::Other(other_id) => IcedMouseButton::Other(other_id as u16),
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

fn baseview_key_location_to_iced(
    location: keyboard_types::Location,
) -> iced_runtime::core::keyboard::Location {
    use iced_runtime::core::keyboard::Location as ILocation;
    use keyboard_types::Location as KLocation;

    match location {
        KLocation::Standard => ILocation::Standard,
        KLocation::Left => ILocation::Left,
        KLocation::Right => ILocation::Right,
        KLocation::Numpad => ILocation::Numpad,
    }
}

fn baseview_to_iced_key(key: keyboard_types::Key) -> iced_runtime::core::keyboard::Key {
    use iced_runtime::core::keyboard::key::Named as IN;
    use iced_runtime::core::keyboard::Key as IKey;
    use keyboard_types::Key as KKey;

    match key {
        KKey::Character(s) => IKey::Character(s.into()),

        KKey::Alt => IKey::Named(IN::Alt),
        KKey::AltGraph => IKey::Named(IN::AltGraph),
        KKey::CapsLock => IKey::Named(IN::CapsLock),
        KKey::Control => IKey::Named(IN::Control),
        KKey::Fn => IKey::Named(IN::Fn),
        KKey::FnLock => IKey::Named(IN::FnLock),
        KKey::Meta => IKey::Named(IN::Meta),
        KKey::NumLock => IKey::Named(IN::NumLock),
        KKey::ScrollLock => IKey::Named(IN::ScrollLock),
        KKey::Shift => IKey::Named(IN::Shift),
        KKey::Symbol => IKey::Named(IN::Symbol),
        KKey::SymbolLock => IKey::Named(IN::SymbolLock),
        KKey::Hyper => IKey::Named(IN::Hyper),
        KKey::Super => IKey::Named(IN::Super),
        KKey::Enter => IKey::Named(IN::Enter),
        KKey::Tab => IKey::Named(IN::Tab),
        KKey::ArrowDown => IKey::Named(IN::ArrowDown),
        KKey::ArrowLeft => IKey::Named(IN::ArrowLeft),
        KKey::ArrowRight => IKey::Named(IN::ArrowRight),
        KKey::ArrowUp => IKey::Named(IN::ArrowUp),
        KKey::End => IKey::Named(IN::End),
        KKey::Home => IKey::Named(IN::Home),
        KKey::PageDown => IKey::Named(IN::PageDown),
        KKey::PageUp => IKey::Named(IN::PageUp),
        KKey::Backspace => IKey::Named(IN::Backspace),
        KKey::Clear => IKey::Named(IN::Clear),
        KKey::Copy => IKey::Named(IN::Copy),
        KKey::CrSel => IKey::Named(IN::CrSel),
        KKey::Cut => IKey::Named(IN::Cut),
        KKey::Delete => IKey::Named(IN::Delete),
        KKey::EraseEof => IKey::Named(IN::EraseEof),
        KKey::ExSel => IKey::Named(IN::ExSel),
        KKey::Insert => IKey::Named(IN::Insert),
        KKey::Paste => IKey::Named(IN::Paste),
        KKey::Redo => IKey::Named(IN::Redo),
        KKey::Undo => IKey::Named(IN::Undo),
        KKey::Accept => IKey::Named(IN::Accept),
        KKey::Again => IKey::Named(IN::Again),
        KKey::Attn => IKey::Named(IN::Attn),
        KKey::Cancel => IKey::Named(IN::Cancel),
        KKey::ContextMenu => IKey::Named(IN::ContextMenu),
        KKey::Escape => IKey::Named(IN::Escape),
        KKey::Execute => IKey::Named(IN::Execute),
        KKey::Find => IKey::Named(IN::Find),
        KKey::Help => IKey::Named(IN::Help),
        KKey::Pause => IKey::Named(IN::Pause),
        KKey::Play => IKey::Named(IN::Play),
        KKey::Props => IKey::Named(IN::Props),
        KKey::Select => IKey::Named(IN::Select),
        KKey::ZoomIn => IKey::Named(IN::ZoomIn),
        KKey::ZoomOut => IKey::Named(IN::ZoomOut),
        KKey::BrightnessDown => IKey::Named(IN::BrightnessDown),
        KKey::BrightnessUp => IKey::Named(IN::BrightnessUp),
        KKey::Eject => IKey::Named(IN::Eject),
        KKey::LogOff => IKey::Named(IN::LogOff),
        KKey::Power => IKey::Named(IN::Power),
        KKey::PowerOff => IKey::Named(IN::PowerOff),
        KKey::PrintScreen => IKey::Named(IN::PrintScreen),
        KKey::Hibernate => IKey::Named(IN::Hibernate),
        KKey::Standby => IKey::Named(IN::Standby),
        KKey::WakeUp => IKey::Named(IN::WakeUp),
        KKey::AllCandidates => IKey::Named(IN::AllCandidates),
        KKey::Alphanumeric => IKey::Named(IN::Alphanumeric),
        KKey::CodeInput => IKey::Named(IN::CodeInput),
        KKey::Compose => IKey::Named(IN::Compose),
        KKey::Convert => IKey::Named(IN::Convert),
        KKey::FinalMode => IKey::Named(IN::FinalMode),
        KKey::GroupFirst => IKey::Named(IN::GroupFirst),
        KKey::GroupLast => IKey::Named(IN::GroupLast),
        KKey::GroupNext => IKey::Named(IN::GroupNext),
        KKey::GroupPrevious => IKey::Named(IN::GroupPrevious),
        KKey::ModeChange => IKey::Named(IN::ModeChange),
        KKey::NextCandidate => IKey::Named(IN::NextCandidate),
        KKey::NonConvert => IKey::Named(IN::NonConvert),
        KKey::PreviousCandidate => IKey::Named(IN::PreviousCandidate),
        KKey::Process => IKey::Named(IN::Process),
        KKey::SingleCandidate => IKey::Named(IN::SingleCandidate),
        KKey::HangulMode => IKey::Named(IN::HangulMode),
        KKey::HanjaMode => IKey::Named(IN::HanjaMode),
        KKey::JunjaMode => IKey::Named(IN::JunjaMode),
        KKey::Eisu => IKey::Named(IN::Eisu),
        KKey::Hankaku => IKey::Named(IN::Hankaku),
        KKey::Hiragana => IKey::Named(IN::Hiragana),
        KKey::HiraganaKatakana => IKey::Named(IN::HiraganaKatakana),
        KKey::KanaMode => IKey::Named(IN::KanaMode),
        KKey::KanjiMode => IKey::Named(IN::KanjiMode),
        KKey::Katakana => IKey::Named(IN::Katakana),
        KKey::Romaji => IKey::Named(IN::Romaji),
        KKey::Zenkaku => IKey::Named(IN::Zenkaku),
        KKey::ZenkakuHankaku => IKey::Named(IN::ZenkakuHankaku),
        KKey::F1 => IKey::Named(IN::F1),
        KKey::F2 => IKey::Named(IN::F2),
        KKey::F3 => IKey::Named(IN::F3),
        KKey::F4 => IKey::Named(IN::F4),
        KKey::F5 => IKey::Named(IN::F5),
        KKey::F6 => IKey::Named(IN::F6),
        KKey::F7 => IKey::Named(IN::F7),
        KKey::F8 => IKey::Named(IN::F8),
        KKey::F9 => IKey::Named(IN::F9),
        KKey::F10 => IKey::Named(IN::F10),
        KKey::F11 => IKey::Named(IN::F11),
        KKey::F12 => IKey::Named(IN::F12),
        KKey::Soft1 => IKey::Named(IN::Soft1),
        KKey::Soft2 => IKey::Named(IN::Soft2),
        KKey::Soft3 => IKey::Named(IN::Soft3),
        KKey::Soft4 => IKey::Named(IN::Soft4),
        KKey::ChannelDown => IKey::Named(IN::ChannelDown),
        KKey::ChannelUp => IKey::Named(IN::ChannelUp),
        KKey::Close => IKey::Named(IN::Close),
        KKey::MailForward => IKey::Named(IN::MailForward),
        KKey::MailReply => IKey::Named(IN::MailReply),
        KKey::MailSend => IKey::Named(IN::MailSend),
        KKey::MediaClose => IKey::Named(IN::MediaClose),
        KKey::MediaFastForward => IKey::Named(IN::MediaFastForward),
        KKey::MediaPause => IKey::Named(IN::MediaPause),
        KKey::MediaPlay => IKey::Named(IN::MediaPlay),
        KKey::MediaPlayPause => IKey::Named(IN::MediaPlayPause),
        KKey::MediaRecord => IKey::Named(IN::MediaRecord),
        KKey::MediaRewind => IKey::Named(IN::MediaRewind),
        KKey::MediaStop => IKey::Named(IN::MediaStop),
        KKey::MediaTrackNext => IKey::Named(IN::MediaTrackNext),
        KKey::MediaTrackPrevious => IKey::Named(IN::MediaTrackPrevious),
        KKey::New => IKey::Named(IN::New),
        KKey::Open => IKey::Named(IN::Open),
        KKey::Print => IKey::Named(IN::Print),
        KKey::Save => IKey::Named(IN::Save),
        KKey::SpellCheck => IKey::Named(IN::SpellCheck),
        KKey::Key11 => IKey::Named(IN::Key11),
        KKey::Key12 => IKey::Named(IN::Key12),
        KKey::AudioBalanceLeft => IKey::Named(IN::AudioBalanceLeft),
        KKey::AudioBalanceRight => IKey::Named(IN::AudioBalanceRight),
        KKey::AudioBassBoostDown => IKey::Named(IN::AudioBassBoostDown),
        KKey::AudioBassBoostToggle => IKey::Named(IN::AudioBassBoostToggle),
        KKey::AudioBassBoostUp => IKey::Named(IN::AudioBassBoostUp),
        KKey::AudioFaderFront => IKey::Named(IN::AudioFaderFront),
        KKey::AudioFaderRear => IKey::Named(IN::AudioFaderRear),
        KKey::AudioSurroundModeNext => IKey::Named(IN::AudioSurroundModeNext),
        KKey::AudioTrebleDown => IKey::Named(IN::AudioTrebleDown),
        KKey::AudioTrebleUp => IKey::Named(IN::AudioTrebleUp),
        KKey::AudioVolumeDown => IKey::Named(IN::AudioVolumeDown),
        KKey::AudioVolumeUp => IKey::Named(IN::AudioVolumeUp),
        KKey::AudioVolumeMute => IKey::Named(IN::AudioVolumeMute),
        KKey::MicrophoneToggle => IKey::Named(IN::MicrophoneToggle),
        KKey::MicrophoneVolumeDown => IKey::Named(IN::MicrophoneVolumeDown),
        KKey::MicrophoneVolumeUp => IKey::Named(IN::MicrophoneVolumeUp),
        KKey::MicrophoneVolumeMute => IKey::Named(IN::MicrophoneVolumeMute),
        KKey::SpeechCorrectionList => IKey::Named(IN::SpeechCorrectionList),
        KKey::SpeechInputToggle => IKey::Named(IN::SpeechInputToggle),
        KKey::LaunchApplication1 => IKey::Named(IN::LaunchApplication1),
        KKey::LaunchApplication2 => IKey::Named(IN::LaunchApplication2),
        KKey::LaunchCalendar => IKey::Named(IN::LaunchCalendar),
        KKey::LaunchContacts => IKey::Named(IN::LaunchContacts),
        KKey::LaunchMail => IKey::Named(IN::LaunchMail),
        KKey::LaunchMediaPlayer => IKey::Named(IN::LaunchMediaPlayer),
        KKey::LaunchMusicPlayer => IKey::Named(IN::LaunchMusicPlayer),
        KKey::LaunchPhone => IKey::Named(IN::LaunchPhone),
        KKey::LaunchScreenSaver => IKey::Named(IN::LaunchScreenSaver),
        KKey::LaunchSpreadsheet => IKey::Named(IN::LaunchSpreadsheet),
        KKey::LaunchWebBrowser => IKey::Named(IN::LaunchWebBrowser),
        KKey::LaunchWebCam => IKey::Named(IN::LaunchWebCam),
        KKey::LaunchWordProcessor => IKey::Named(IN::LaunchWordProcessor),
        KKey::BrowserBack => IKey::Named(IN::BrowserBack),
        KKey::BrowserFavorites => IKey::Named(IN::BrowserFavorites),
        KKey::BrowserForward => IKey::Named(IN::BrowserForward),
        KKey::BrowserHome => IKey::Named(IN::BrowserHome),
        KKey::BrowserRefresh => IKey::Named(IN::BrowserRefresh),
        KKey::BrowserSearch => IKey::Named(IN::BrowserSearch),
        KKey::BrowserStop => IKey::Named(IN::BrowserStop),
        KKey::AppSwitch => IKey::Named(IN::AppSwitch),
        KKey::Call => IKey::Named(IN::Call),
        KKey::Camera => IKey::Named(IN::Camera),
        KKey::CameraFocus => IKey::Named(IN::CameraFocus),
        KKey::EndCall => IKey::Named(IN::EndCall),
        KKey::GoBack => IKey::Named(IN::GoBack),
        KKey::GoHome => IKey::Named(IN::GoHome),
        KKey::HeadsetHook => IKey::Named(IN::HeadsetHook),
        KKey::LastNumberRedial => IKey::Named(IN::LastNumberRedial),
        KKey::Notification => IKey::Named(IN::Notification),
        KKey::MannerMode => IKey::Named(IN::MannerMode),
        KKey::VoiceDial => IKey::Named(IN::VoiceDial),
        KKey::TV => IKey::Named(IN::TV),
        KKey::TV3DMode => IKey::Named(IN::TV3DMode),
        KKey::TVAntennaCable => IKey::Named(IN::TVAntennaCable),
        KKey::TVAudioDescription => IKey::Named(IN::TVAudioDescription),
        KKey::TVAudioDescriptionMixDown => IKey::Named(IN::TVAudioDescriptionMixDown),
        KKey::TVAudioDescriptionMixUp => IKey::Named(IN::TVAudioDescriptionMixUp),
        KKey::TVContentsMenu => IKey::Named(IN::TVContentsMenu),
        KKey::TVDataService => IKey::Named(IN::TVDataService),
        KKey::TVInput => IKey::Named(IN::TVInput),
        KKey::TVInputComponent1 => IKey::Named(IN::TVInputComponent1),
        KKey::TVInputComponent2 => IKey::Named(IN::TVInputComponent2),
        KKey::TVInputComposite1 => IKey::Named(IN::TVInputComposite1),
        KKey::TVInputComposite2 => IKey::Named(IN::TVInputComposite2),
        KKey::TVInputHDMI1 => IKey::Named(IN::TVInputHDMI1),
        KKey::TVInputHDMI2 => IKey::Named(IN::TVInputHDMI2),
        KKey::TVInputHDMI3 => IKey::Named(IN::TVInputHDMI3),
        KKey::TVInputHDMI4 => IKey::Named(IN::TVInputHDMI4),
        KKey::TVInputVGA1 => IKey::Named(IN::TVInputVGA1),
        KKey::TVMediaContext => IKey::Named(IN::TVMediaContext),
        KKey::TVNetwork => IKey::Named(IN::TVNetwork),
        KKey::TVNumberEntry => IKey::Named(IN::TVNumberEntry),
        KKey::TVPower => IKey::Named(IN::TVPower),
        KKey::TVRadioService => IKey::Named(IN::TVRadioService),
        KKey::TVSatellite => IKey::Named(IN::TVSatellite),
        KKey::TVSatelliteBS => IKey::Named(IN::TVSatelliteBS),
        KKey::TVSatelliteCS => IKey::Named(IN::TVSatelliteCS),
        KKey::TVSatelliteToggle => IKey::Named(IN::TVSatelliteToggle),
        KKey::TVTerrestrialAnalog => IKey::Named(IN::TVTerrestrialAnalog),
        KKey::TVTerrestrialDigital => IKey::Named(IN::TVTerrestrialDigital),
        KKey::TVTimer => IKey::Named(IN::TVTimer),
        KKey::AVRInput => IKey::Named(IN::AVRInput),
        KKey::AVRPower => IKey::Named(IN::AVRPower),
        KKey::ColorF0Red => IKey::Named(IN::ColorF0Red),
        KKey::ColorF1Green => IKey::Named(IN::ColorF1Green),
        KKey::ColorF2Yellow => IKey::Named(IN::ColorF2Yellow),
        KKey::ColorF3Blue => IKey::Named(IN::ColorF3Blue),
        KKey::ColorF4Grey => IKey::Named(IN::ColorF4Grey),
        KKey::ColorF5Brown => IKey::Named(IN::ColorF5Brown),
        KKey::ClosedCaptionToggle => IKey::Named(IN::ClosedCaptionToggle),
        KKey::Dimmer => IKey::Named(IN::Dimmer),
        KKey::DisplaySwap => IKey::Named(IN::DisplaySwap),
        KKey::DVR => IKey::Named(IN::DVR),
        KKey::Exit => IKey::Named(IN::Exit),
        KKey::FavoriteClear0 => IKey::Named(IN::FavoriteClear0),
        KKey::FavoriteClear1 => IKey::Named(IN::FavoriteClear1),
        KKey::FavoriteClear2 => IKey::Named(IN::FavoriteClear2),
        KKey::FavoriteClear3 => IKey::Named(IN::FavoriteClear3),
        KKey::FavoriteRecall0 => IKey::Named(IN::FavoriteRecall0),
        KKey::FavoriteRecall1 => IKey::Named(IN::FavoriteRecall1),
        KKey::FavoriteRecall2 => IKey::Named(IN::FavoriteRecall2),
        KKey::FavoriteRecall3 => IKey::Named(IN::FavoriteRecall3),
        KKey::FavoriteStore0 => IKey::Named(IN::FavoriteStore0),
        KKey::FavoriteStore1 => IKey::Named(IN::FavoriteStore1),
        KKey::FavoriteStore2 => IKey::Named(IN::FavoriteStore2),
        KKey::FavoriteStore3 => IKey::Named(IN::FavoriteStore3),
        KKey::Guide => IKey::Named(IN::Guide),
        KKey::GuideNextDay => IKey::Named(IN::GuideNextDay),
        KKey::GuidePreviousDay => IKey::Named(IN::GuidePreviousDay),
        KKey::Info => IKey::Named(IN::Info),
        KKey::InstantReplay => IKey::Named(IN::InstantReplay),
        KKey::Link => IKey::Named(IN::Link),
        KKey::ListProgram => IKey::Named(IN::ListProgram),
        KKey::LiveContent => IKey::Named(IN::LiveContent),
        KKey::Lock => IKey::Named(IN::Lock),
        KKey::MediaApps => IKey::Named(IN::MediaApps),
        KKey::MediaAudioTrack => IKey::Named(IN::MediaAudioTrack),
        KKey::MediaLast => IKey::Named(IN::MediaLast),
        KKey::MediaSkipBackward => IKey::Named(IN::MediaSkipBackward),
        KKey::MediaSkipForward => IKey::Named(IN::MediaSkipForward),
        KKey::MediaStepBackward => IKey::Named(IN::MediaStepBackward),
        KKey::MediaStepForward => IKey::Named(IN::MediaStepForward),
        KKey::MediaTopMenu => IKey::Named(IN::MediaTopMenu),
        KKey::NavigateIn => IKey::Named(IN::NavigateIn),
        KKey::NavigateNext => IKey::Named(IN::NavigateNext),
        KKey::NavigateOut => IKey::Named(IN::NavigateOut),
        KKey::NavigatePrevious => IKey::Named(IN::NavigatePrevious),
        KKey::NextFavoriteChannel => IKey::Named(IN::NextFavoriteChannel),
        KKey::NextUserProfile => IKey::Named(IN::NextUserProfile),
        KKey::OnDemand => IKey::Named(IN::OnDemand),
        KKey::Pairing => IKey::Named(IN::Pairing),
        KKey::PinPMove => IKey::Named(IN::PinPMove),
        KKey::PinPToggle => IKey::Named(IN::PinPToggle),
        KKey::PinPUp => IKey::Named(IN::PinPUp),
        KKey::PlaySpeedDown => IKey::Named(IN::PlaySpeedDown),
        KKey::PlaySpeedReset => IKey::Named(IN::PlaySpeedReset),
        KKey::PlaySpeedUp => IKey::Named(IN::PlaySpeedUp),
        KKey::RandomToggle => IKey::Named(IN::RandomToggle),
        KKey::RcLowBattery => IKey::Named(IN::RcLowBattery),
        KKey::RecordSpeedNext => IKey::Named(IN::RecordSpeedNext),
        KKey::RfBypass => IKey::Named(IN::RfBypass),
        KKey::ScanChannelsToggle => IKey::Named(IN::ScanChannelsToggle),
        KKey::ScreenModeNext => IKey::Named(IN::ScreenModeNext),
        KKey::Settings => IKey::Named(IN::Settings),
        KKey::SplitScreenToggle => IKey::Named(IN::SplitScreenToggle),
        KKey::STBInput => IKey::Named(IN::STBInput),
        KKey::STBPower => IKey::Named(IN::STBPower),
        KKey::Subtitle => IKey::Named(IN::Subtitle),
        KKey::Teletext => IKey::Named(IN::Teletext),
        KKey::VideoModeNext => IKey::Named(IN::VideoModeNext),
        KKey::Wink => IKey::Named(IN::Wink),
        KKey::ZoomToggle => IKey::Named(IN::ZoomToggle),
        KKey::F13 => IKey::Named(IN::F13),
        KKey::F14 => IKey::Named(IN::F14),
        KKey::F15 => IKey::Named(IN::F15),
        KKey::F16 => IKey::Named(IN::F16),
        KKey::F17 => IKey::Named(IN::F17),
        KKey::F18 => IKey::Named(IN::F18),
        KKey::F19 => IKey::Named(IN::F19),
        KKey::F20 => IKey::Named(IN::F20),
        KKey::F21 => IKey::Named(IN::F21),
        KKey::F22 => IKey::Named(IN::F22),
        KKey::F23 => IKey::Named(IN::F23),
        KKey::F24 => IKey::Named(IN::F24),
        _ => IKey::Unidentified,
    }
}

fn baseview_to_iced_keycode(
    code: keyboard_types::Code,
) -> Option<iced_runtime::core::keyboard::key::Code> {
    use iced_runtime::core::keyboard::key::Code as ICode;
    use keyboard_types::Code as KCode;

    match code {
        KCode::Digit1 => Some(ICode::Numpad1),
        KCode::Digit2 => Some(ICode::Numpad2),
        KCode::Digit3 => Some(ICode::Numpad3),
        KCode::Digit4 => Some(ICode::Numpad4),
        KCode::Digit5 => Some(ICode::Numpad5),
        KCode::Digit6 => Some(ICode::Numpad6),
        KCode::Digit7 => Some(ICode::Numpad7),
        KCode::Digit8 => Some(ICode::Numpad8),
        KCode::Digit9 => Some(ICode::Numpad9),
        KCode::Digit0 => Some(ICode::Numpad0),

        KCode::KeyA => Some(ICode::KeyA),
        KCode::KeyB => Some(ICode::KeyB),
        KCode::KeyC => Some(ICode::KeyC),
        KCode::KeyD => Some(ICode::KeyD),
        KCode::KeyE => Some(ICode::KeyE),
        KCode::KeyF => Some(ICode::KeyF),
        KCode::KeyG => Some(ICode::KeyG),
        KCode::KeyH => Some(ICode::KeyH),
        KCode::KeyI => Some(ICode::KeyI),
        KCode::KeyJ => Some(ICode::KeyJ),
        KCode::KeyK => Some(ICode::KeyK),
        KCode::KeyL => Some(ICode::KeyL),
        KCode::KeyM => Some(ICode::KeyM),
        KCode::KeyN => Some(ICode::KeyN),
        KCode::KeyO => Some(ICode::KeyO),
        KCode::KeyP => Some(ICode::KeyP),
        KCode::KeyQ => Some(ICode::KeyQ),
        KCode::KeyR => Some(ICode::KeyR),
        KCode::KeyS => Some(ICode::KeyS),
        KCode::KeyT => Some(ICode::KeyT),
        KCode::KeyU => Some(ICode::KeyU),
        KCode::KeyV => Some(ICode::KeyV),
        KCode::KeyW => Some(ICode::KeyW),
        KCode::KeyX => Some(ICode::KeyX),
        KCode::KeyY => Some(ICode::KeyY),
        KCode::KeyZ => Some(ICode::KeyZ),

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

        KCode::PrintScreen => Some(ICode::PrintScreen),
        KCode::ScrollLock => Some(ICode::ScrollLock),
        KCode::Pause => Some(ICode::Pause),

        KCode::Insert => Some(ICode::Insert),
        KCode::Home => Some(ICode::Home),
        KCode::Delete => Some(ICode::Delete),
        KCode::End => Some(ICode::End),
        KCode::PageDown => Some(ICode::PageDown),
        KCode::PageUp => Some(ICode::PageUp),

        KCode::ArrowLeft => Some(ICode::ArrowLeft),
        KCode::ArrowUp => Some(ICode::ArrowUp),
        KCode::ArrowRight => Some(ICode::ArrowRight),
        KCode::ArrowDown => Some(ICode::ArrowDown),

        KCode::Backspace => Some(ICode::Backspace),
        KCode::Enter => Some(ICode::Enter),
        KCode::Space => Some(ICode::Space),

        KCode::NumLock => Some(ICode::NumLock),
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
        KCode::NumpadEqual => Some(ICode::NumpadEqual),
        KCode::NumpadMultiply => Some(ICode::NumpadMultiply),
        KCode::NumpadSubtract => Some(ICode::NumpadSubtract),

        //KCode::AbntC1 => Some(ICode::AbntC1),    // TODO ?
        //KCode::AbntC1 => Some(ICode::AbntC1),    // TODO ?
        KCode::Convert => Some(ICode::Convert),
        KCode::KanaMode => Some(ICode::KanaMode),
        //KCode::Kanji => ICode::Kanji),    // TODO ?
        KCode::NonConvert => Some(ICode::NonConvert),
        KCode::IntlYen => Some(ICode::IntlYen),

        KCode::AltLeft => Some(ICode::AltLeft),
        KCode::AltRight => Some(ICode::AltRight),
        KCode::BracketLeft => Some(ICode::BracketLeft),
        KCode::BracketRight => Some(ICode::BracketRight),
        KCode::ControlLeft => Some(ICode::ControlLeft),
        KCode::ControlRight => Some(ICode::ControlRight),
        KCode::ShiftLeft => Some(ICode::ShiftLeft),
        KCode::ShiftRight => Some(ICode::ShiftRight),
        KCode::MetaLeft => Some(ICode::Meta),
        KCode::MetaRight => Some(ICode::Meta),

        KCode::Minus => Some(ICode::Minus),
        KCode::Period => Some(ICode::Period),
        //KCode::Plus => Some(ICode::Plus),    // TODO ?
        KCode::Equal => Some(ICode::Equal),
        KCode::Quote => Some(ICode::Quote),
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
        KCode::MediaPlayPause => Some(ICode::MediaPlayPause),
        KCode::AudioVolumeMute => Some(ICode::AudioVolumeMute),
        KCode::AudioVolumeDown => Some(ICode::AudioVolumeDown),
        KCode::AudioVolumeUp => Some(ICode::AudioVolumeUp),
        KCode::MediaTrackNext => Some(ICode::MediaTrackNext),
        KCode::MediaTrackPrevious => Some(ICode::MediaTrackPrevious),

        _ => None,
    }
}

pub fn convert_mouse_interaction(
    interaction: crate::runtime::core::mouse::Interaction,
) -> baseview::MouseCursor {
    use crate::runtime::core::mouse::Interaction as ICursor;
    use baseview::MouseCursor as BCursor;

    match interaction {
        ICursor::None => BCursor::Default,
        ICursor::Idle => BCursor::Default,
        ICursor::Pointer => BCursor::Hand,
        ICursor::Grab => BCursor::HandGrabbing,
        ICursor::Text => BCursor::Text,
        ICursor::Crosshair => BCursor::Crosshair,
        ICursor::Working => BCursor::Working,
        ICursor::Grabbing => BCursor::HandGrabbing,
        ICursor::ResizingHorizontally => BCursor::ColResize,
        ICursor::ResizingVertically => BCursor::RowResize,
        ICursor::ResizingDiagonallyUp => BCursor::NeswResize,
        ICursor::ResizingDiagonallyDown => BCursor::NwseResize,
        ICursor::NotAllowed => BCursor::NotAllowed,
        ICursor::ZoomIn => BCursor::ZoomIn,
        ICursor::ZoomOut => BCursor::ZoomOut,
        ICursor::Cell => BCursor::Cell,
        ICursor::Move => BCursor::Move,
        ICursor::Copy => BCursor::Copy,
        ICursor::Help => BCursor::Help,
    }
}

pub fn convert_raw_display_handle(
    handle05: raw_window_handle::RawDisplayHandle,
) -> raw_window_handle_06::RawDisplayHandle {
    use std::ptr::NonNull;

    match handle05 {
        raw_window_handle::RawDisplayHandle::AppKit(_) => {
            raw_window_handle_06::RawDisplayHandle::AppKit(
                raw_window_handle_06::AppKitDisplayHandle::new(),
            )
        }
        raw_window_handle::RawDisplayHandle::Xlib(handle) => {
            raw_window_handle_06::RawDisplayHandle::Xlib(
                raw_window_handle_06::XlibDisplayHandle::new(
                    NonNull::new(handle.display),
                    handle.screen,
                ),
            )
        }
        raw_window_handle::RawDisplayHandle::Xcb(handle) => {
            raw_window_handle_06::RawDisplayHandle::Xcb(
                raw_window_handle_06::XcbDisplayHandle::new(
                    NonNull::new(handle.connection),
                    handle.screen,
                ),
            )
        }
        raw_window_handle::RawDisplayHandle::Windows(_) => {
            raw_window_handle_06::RawDisplayHandle::Windows(
                raw_window_handle_06::WindowsDisplayHandle::new(),
            )
        }
        _ => todo!(),
    }
}

pub fn convert_raw_window_handle(
    handle05: raw_window_handle::RawWindowHandle,
) -> raw_window_handle_06::RawWindowHandle {
    use std::num::{NonZeroIsize, NonZeroU32};
    use std::ptr::NonNull;

    match handle05 {
        raw_window_handle::RawWindowHandle::AppKit(handle) => {
            raw_window_handle_06::RawWindowHandle::AppKit(
                raw_window_handle_06::AppKitWindowHandle::new(
                    NonNull::new(handle.ns_view).unwrap(),
                ),
            )
        }
        raw_window_handle::RawWindowHandle::Xlib(handle) => {
            raw_window_handle_06::RawWindowHandle::Xlib(
                raw_window_handle_06::XlibWindowHandle::new(handle.window),
            )
        }
        raw_window_handle::RawWindowHandle::Xcb(handle) => {
            raw_window_handle_06::RawWindowHandle::Xcb(raw_window_handle_06::XcbWindowHandle::new(
                NonZeroU32::new(handle.window).unwrap(),
            ))
        }
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            // will this work? i have no idea!
            let mut raw_handle = raw_window_handle_06::Win32WindowHandle::new(
                NonZeroIsize::new(handle.hwnd as isize).unwrap(),
            );

            raw_handle.hinstance = handle
                .hinstance
                .is_null()
                .then(|| NonZeroIsize::new(handle.hinstance as isize).unwrap());

            raw_window_handle_06::RawWindowHandle::Win32(raw_handle)
        }
        _ => todo!(),
    }
}

#[derive(Clone)]
pub struct WindowWrapper {
    window: raw_window_handle_06::RawWindowHandle,
    display: raw_window_handle_06::RawDisplayHandle,
}

pub fn convert_window(window: &baseview::Window<'_>) -> WindowWrapper {
    WindowWrapper {
        window: convert_raw_window_handle(window.raw_window_handle()),
        display: convert_raw_display_handle(window.raw_display_handle()),
    }
}

impl raw_window_handle_06::HasWindowHandle for WindowWrapper {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle_06::WindowHandle<'static>, raw_window_handle_06::HandleError>
    {
        Ok(unsafe { raw_window_handle_06::WindowHandle::borrow_raw(self.window) })
    }
}

impl raw_window_handle_06::HasDisplayHandle for WindowWrapper {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle_06::DisplayHandle<'static>, raw_window_handle_06::HandleError>
    {
        Ok(unsafe { raw_window_handle_06::DisplayHandle::borrow_raw(self.display) })
    }
}

unsafe impl Send for WindowWrapper {}
unsafe impl Sync for WindowWrapper {}
