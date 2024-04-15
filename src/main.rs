use std::env;

use windows::core::{GUID, w};
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::UpdateWindow,
    System::Power::*,
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::*,
    },
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{NIF_GUID, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_SETVERSION, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW};

// Bindings lookup - https://microsoft.github.io/windows-rs/features/#/master

fn main() {
    let hwnd = setup();
    message_loop(hwnd);
}

const NID_ACTION: usize = 1;

const INTERVAL: u32 = 30 * 1000;

fn setup() -> HWND {
    // Setup
    unsafe {
        let hinstance = HINSTANCE::from(GetModuleHandleW(None).unwrap());

        SetThreadExecutionState(ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED | ES_CONTINUOUS);

        let wnd_class_name = w!("StayAwake");

        // Create a window class
        let wndclass = WNDCLASSEXW {
            cbSize: core::mem::size_of::<WNDCLASSEXW>() as u32,
            hInstance: hinstance,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            lpszClassName: wnd_class_name,
            ..Default::default()
        };
        let _atom = RegisterClassExW(&wndclass);

        let hwnd = CreateWindowExW(WS_EX_TOOLWINDOW, wnd_class_name, w!(""), WS_VISIBLE, 0, 0, 1, 1, HWND::default(), HMENU::default(), hinstance, None);

        ShowWindow(hwnd, SW_HIDE);
        UpdateWindow(hwnd);
        SetTimer::<HWND>(hwnd, NID_ACTION, INTERVAL, None);

        setup_notification_icon(hwnd);

        hwnd
    }
}

unsafe fn get_x(param: usize) -> i32 {
    // GET_X_LPARAM
    core::mem::transmute((param & 0xffff) as u32)
}

unsafe fn get_y(param: usize) -> i32 {
    // GET_Y_LPARAM - note code assumes usize == u64
    core::mem::transmute(((param >> 16) & 0xffff) as u32)
}

const MENU_QUIT: usize = 1;

extern "system" fn wndproc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                // F15 key
                let mut ip: INPUT = INPUT::default();
                ip.r#type = INPUT_KEYBOARD;
                ip.Anonymous.ki.wVk = VK_F15;
                SendInput(&[ip], std::mem::size_of::<INPUT>() as i32);
                LRESULT(0)
            }
            WM_COMMAND => {
                if w_param.0 == MENU_QUIT {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            },
            WM_USER_TRAY_NOTIFICATION => {
                // Tray
                match l_param.0 as u32 {
                    WM_RBUTTONUP => {
                        // Show context menu
                        let x = get_x(w_param.0);
                        let y = get_y(w_param.0);

                        let _ = CreatePopupMenu().map(|hmenu| {
                            let _ = AppendMenuW(hmenu, MF_STRING| MF_ENABLED, MENU_QUIT, w!("Quit"));
                            let _ = TrackPopupMenu(
                                hmenu, TPM_LEFTALIGN | TPM_BOTTOMALIGN, x as i32, y as i32, 0, hwnd, None);
                            DestroyMenu(hmenu);
                        });
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param)
        }
    }
}

const WM_USER_TRAY_NOTIFICATION: u32 = WM_USER;

fn setup_notification_icon(hwnd: HWND) {
    unsafe {
        let hicon = if let Ok(handle) = LoadImageW(None, w!(".\\stayawake.ico"), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE | LR_LOADFROMFILE) {
            HICON(handle.0)
        } else {
            // TODO Get icon associated with executable
            HICON::default()
        };

        let guid = GUID::new().unwrap();
        let mut nid = NOTIFYICONDATAW {
            cbSize: core::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uFlags: NIF_ICON | NIF_GUID | NIF_MESSAGE,
            guidItem: guid,
            hIcon: hicon,
            uCallbackMessage: WM_USER_TRAY_NOTIFICATION,
            ..Default::default()
        };
        nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;

        Shell_NotifyIconW(NIM_ADD, &nid);
        Shell_NotifyIconW(NIM_SETVERSION, &nid);
    }
}

fn message_loop(hwnd: HWND) {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            DispatchMessageW(&msg);
        }
    }
}