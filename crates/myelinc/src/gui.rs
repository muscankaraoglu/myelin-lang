//! Native lightweight desktop GUI subsystem for Myelin (`!gui`).
//! Uses direct native OS windowing (Win32) with zero external dependencies.

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone)]
pub struct ButtonControl {
    pub id: usize,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub callback: String,
}

#[derive(Clone)]
pub struct TextControl {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone)]
pub struct WindowState {
    pub id: usize,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub buttons: Vec<ButtonControl>,
    pub texts: Vec<TextControl>,
}



static WINDOW_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
static EVENT_QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn next_window_id() -> usize {
    WINDOW_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case, dead_code)]
mod win32 {
    use std::ffi::c_void;
    pub type HWND = *mut c_void;
    pub type HINSTANCE = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type LPARAM = isize;
    pub type WPARAM = usize;
    pub type LRESULT = isize;
    pub type WNDPROC = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

    pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
    pub const WS_VISIBLE: u32 = 0x10000000;
    pub const WS_CHILD: u32 = 0x40000000;
    pub const BS_PUSHBUTTON: u32 = 0x00000000;
    pub const SS_LEFT: u32 = 0x00000000;
    pub const SW_SHOW: i32 = 5;
    pub const WM_DESTROY: u32 = 0x0002;
    pub const WM_COMMAND: u32 = 0x0111;
    pub const MB_OK: u32 = 0x00000000;
    pub const MB_ICONINFORMATION: u32 = 0x00000040;
    pub const MB_YESNO: u32 = 0x00000004;
    pub const MB_ICONQUESTION: u32 = 0x00000020;
    pub const IDYES: i32 = 6;
    pub const COLOR_WINDOW: u32 = 5;

    #[repr(C)]
    pub struct WNDCLASSEXW {
        pub cbSize: u32,
        pub style: u32,
        pub lpfnWndProc: WNDPROC,
        pub cbClsExtra: i32,
        pub cbWndExtra: i32,
        pub hInstance: HINSTANCE,
        pub hIcon: *mut c_void,
        pub hCursor: *mut c_void,
        pub hbrBackground: *mut c_void,
        pub lpszMenuName: *const u16,
        pub lpszClassName: *const u16,
        pub hIconSm: *mut c_void,
    }

    #[repr(C)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct MSG {
        pub hwnd: HWND,
        pub message: u32,
        pub wParam: WPARAM,
        pub lParam: LPARAM,
        pub time: u32,
        pub pt: POINT,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn MessageBoxW(hwnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
        pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
        pub fn CreateWindowExW(
            dwExStyle: u32,
            lpClassName: *const u16,
            lpWindowName: *const u16,
            dwStyle: u32,
            X: i32,
            Y: i32,
            nWidth: i32,
            nHeight: i32,
            hWndParent: HWND,
            hMenu: HMENU,
            hInstance: HINSTANCE,
            lpParam: *mut c_void,
        ) -> HWND;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
        pub fn UpdateWindow(hWnd: HWND) -> i32;
        pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
        pub fn PeekMessageW(
            lpMsg: *mut MSG,
            hWnd: HWND,
            wMsgFilterMin: u32,
            wMsgFilterMax: u32,
            wRemoveMsg: u32,
        ) -> i32;
        pub fn TranslateMessage(lpMsg: *const MSG) -> i32;
        pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
        pub fn PostQuitMessage(nExitCode: i32);
        pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    }
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn alert(title: &str, message: &str) {
    #[cfg(target_os = "windows")]
    unsafe {
        let t = to_wide_null(title);
        let m = to_wide_null(message);
        win32::MessageBoxW(
            std::ptr::null_mut(),
            m.as_ptr(),
            t.as_ptr(),
            win32::MB_OK | win32::MB_ICONINFORMATION,
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("[GUI ALERT: {}] {}", title, message);
    }
}

pub fn confirm(title: &str, message: &str) -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        let t = to_wide_null(title);
        let m = to_wide_null(message);
        let res = win32::MessageBoxW(
            std::ptr::null_mut(),
            m.as_ptr(),
            t.as_ptr(),
            win32::MB_YESNO | win32::MB_ICONQUESTION,
        );
        res == win32::IDYES
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("[GUI CONFIRM: {}] {} (y/n default y)", title, message);
        true
    }
}

// Global button callback map: button_id -> callback_fn_name
static BUTTON_CALLBACKS: Mutex<Option<HashMap<usize, String>>> = Mutex::new(None);

#[cfg(target_os = "windows")]
unsafe extern "system" fn window_proc(
    hwnd: win32::HWND,
    msg: u32,
    wparam: win32::WPARAM,
    lparam: win32::LPARAM,
) -> win32::LRESULT {
    match msg {
        win32::WM_COMMAND => {
            let control_id = (wparam & 0xFFFF) as usize;
            if let Ok(guard) = BUTTON_CALLBACKS.lock() {
                if let Some(map) = &*guard {
                    if let Some(cb) = map.get(&control_id) {
                        if let Ok(mut q) = EVENT_QUEUE.lock() {
                            q.push(cb.clone());
                        }
                    }
                }
            }
            0
        }
        win32::WM_DESTROY => {
            unsafe { win32::PostQuitMessage(0); }
            0
        }
        _ => unsafe { win32::DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn show_window_loop<F>(win: &WindowState, mut invoke_cb: F) -> Result<(), String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    #[cfg(target_os = "windows")]
    unsafe {
        // Populate callback map
        let mut cb_map = HashMap::new();
        for btn in &win.buttons {
            if !btn.callback.is_empty() {
                cb_map.insert(btn.id, btn.callback.clone());
            }
        }
        *BUTTON_CALLBACKS.lock().unwrap() = Some(cb_map);

        let h_instance = win32::GetModuleHandleW(std::ptr::null());
        let class_name = to_wide_null("MyelinWindowClass");

        let wc = win32::WNDCLASSEXW {
            cbSize: std::mem::size_of::<win32::WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: window_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: (win32::COLOR_WINDOW + 1) as *mut std::ffi::c_void,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        win32::RegisterClassExW(&wc);

        let title_wide = to_wide_null(&win.title);
        let hwnd = win32::CreateWindowExW(
            0,
            class_name.as_ptr(),
            title_wide.as_ptr(),
            win32::WS_OVERLAPPEDWINDOW | win32::WS_VISIBLE,
            100,
            100,
            win.width,
            win.height,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            h_instance,
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            return Err("failed to create native win32 window".into());
        }

        // Add static text labels
        let static_class = to_wide_null("STATIC");
        for txt in &win.texts {
            let t_wide = to_wide_null(&txt.text);
            win32::CreateWindowExW(
                0,
                static_class.as_ptr(),
                t_wide.as_ptr(),
                win32::WS_CHILD | win32::WS_VISIBLE | win32::SS_LEFT,
                txt.x,
                txt.y,
                txt.w,
                txt.h,
                hwnd,
                std::ptr::null_mut(),
                h_instance,
                std::ptr::null_mut(),
            );
        }

        // Add button controls
        let button_class = to_wide_null("BUTTON");
        for btn in &win.buttons {
            let b_wide = to_wide_null(&btn.text);
            win32::CreateWindowExW(
                0,
                button_class.as_ptr(),
                b_wide.as_ptr(),
                win32::WS_CHILD | win32::WS_VISIBLE | win32::BS_PUSHBUTTON,
                btn.x,
                btn.y,
                btn.w,
                btn.h,
                hwnd,
                btn.id as win32::HMENU,
                h_instance,
                std::ptr::null_mut(),
            );
        }

        win32::ShowWindow(hwnd, win32::SW_SHOW);
        win32::UpdateWindow(hwnd);

        let mut msg: win32::MSG = std::mem::zeroed();
        while win32::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            win32::TranslateMessage(&msg);
            win32::DispatchMessageW(&msg);

            // Drain queued events
            let pending: Vec<String> = {
                let mut q = EVENT_QUEUE.lock().unwrap();
                q.drain(..).collect()
            };

            for cb in pending {
                invoke_cb(&cb)?;
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("GUI Window '{}' ({}x{}) displayed.", win.title, win.width, win.height);
        Ok(())
    }
}
