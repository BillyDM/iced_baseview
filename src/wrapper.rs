use raw_window_handle as rwh04;

use cfg_if::cfg_if;

/// Implement raw-window-handle API v0.5 on things implementing v0.4
///
/// For use until baseview migrates to v0.5. Necessary for integrating with wgpu in iced v0.5.
pub struct WindowHandleWrapper<'a, T: rwh04::HasRawWindowHandle>(pub &'a T);

unsafe impl<'a, T: rwh04::HasRawWindowHandle> rwh05::HasRawWindowHandle
    for WindowHandleWrapper<'a, T>
{
    fn raw_window_handle(&self) -> rwh05::RawWindowHandle {
        let handle = rwh04::HasRawWindowHandle::raw_window_handle(&self.0);

        cfg_if! {
            if #[cfg(target_os = "macos")] {
                if let rwh04::RawWindowHandle::AppKit(rwh04::AppKitHandle { ns_view, ns_window, .. }) = handle {
                    let mut h = rwh05::AppKitWindowHandle::empty();

                    h.ns_view = ns_view;
                    h.ns_window = ns_window;

                    rwh05::RawWindowHandle::AppKit(h)
                } else {
                    panic!("Not a macOS handle");
                }
            } else if #[cfg(target_os = "windows")] {
                if let rwh04::RawWindowHandle::Win32(rwh04::Win32Handle { hwnd, hinstance, .. }) = handle {
                    let mut h = rwh05::Win32WindowHandle::empty();

                    h.hwnd = hwnd;
                    h.hinstance = hinstance;

                    rwh05::RawWindowHandle::Win32(h)
                } else {
                    panic!("Not a Windows handle");
                }
            } else {
                if let rwh04::RawWindowHandle::Xlib(rwh04::XlibHandle { window, visual_id, .. }) = handle {
                    let mut h = rwh05::XlibWindowHandle::empty();

                    h.window = window;
                    h.visual_id = visual_id;

                    rwh05::RawWindowHandle::Xlib(h)
                } else {
                    panic!("Not an Xlib handle");
                }
            }
        }
    }
}

unsafe impl<'a, T: rwh04::HasRawWindowHandle> rwh05::HasRawDisplayHandle
    for WindowHandleWrapper<'a, T>
{
    fn raw_display_handle(&self) -> rwh05::RawDisplayHandle {
        use rwh05::*;

        cfg_if! {
            if #[cfg(target_os = "macos")] {
                RawDisplayHandle::AppKit(AppKitDisplayHandle::empty())
            } else if #[cfg(target_os = "windows")] {
                RawDisplayHandle::Windows(WindowsDisplayHandle::empty())
            } else {
                let handle = rwh04::HasRawWindowHandle::raw_window_handle(&self.0);

                if let rwh04::RawWindowHandle::Xlib(rwh04::XlibHandle { display, .. }) = handle {
                    let mut h = rwh05::XlibDisplayHandle::empty();

                    h.display = display;

                    rwh05::RawDisplayHandle::Xlib(h)
                } else {
                    panic!("Not an Xlib handle");
                }
            }
        }
    }
}
