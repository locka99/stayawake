#![windows_subsystem = "windows"]

use windows::{
    core::{w, GUID, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::UpdateWindow,
        System::{LibraryLoader::GetModuleHandleW, Power::*},
        UI::{
            Input::KeyboardAndMouse::*,
            Shell::{
                Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE,
                NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICONDATAW_0, NOTIFYICON_VERSION_4,
            },
            WindowsAndMessaging::*,
        },
    },
};

const ACTIVITY_TIMER_ID: usize = 1;
const ACTIVITY_INTERVAL: u32 = 30 * 1000;
const CMD_MENU_QUIT: usize = 1;
const WM_USER_TRAY_NOTIFICATION: u32 = WM_USER;
const ID_TRAY_ICON: u32 = 101;

// Bindings lookup - https://microsoft.github.io/windows-rs/features/#/master

fn main() {
    unsafe {
        // Create the GUI and run in a loop processing timer and menu events
        let (hwnd, nid) = setup();
        message_loop();
        cleanup(hwnd, nid);
    }
}

unsafe fn setup() -> (HWND, NOTIFYICONDATAW) {
    // Setup
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

    // Create a window to process messages
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        wnd_class_name,
        w!(""),
        WS_VISIBLE,
        0,
        0,
        1,
        1,
        HWND::default(),
        HMENU::default(),
        hinstance,
        None,
    );
    ShowWindow(hwnd, SW_HIDE);
    UpdateWindow(hwnd);
    SetTimer::<HWND>(hwnd, ACTIVITY_TIMER_ID, ACTIVITY_INTERVAL, None);

    // Create a tray icon
    let nid = setup_notification_icon(hinstance, hwnd);

    (hwnd, nid)
}

unsafe fn cleanup(_hwnd: HWND, nid: NOTIFYICONDATAW) {
    // Tray icon will be crud that lingers in tray after exit unless it is explicitly removed
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn get_mouse_pos(param: usize) -> (i32, i32) {
    // GET_X_LPARAM / GET_Y_LPARAM macros are not available in bindings, so do it by hand. Returns
    // signed int to support multi monitor displays.
    let x = core::mem::transmute((param & 0xffff) as u32);
    let y = core::mem::transmute(((param >> 16) & 0xffff) as u32);
    (x, y)
}

unsafe fn make_int_resource(id: u32) -> PCWSTR {
    PCWSTR(id as *mut u16)
}

unsafe fn setup_notification_icon(hinstance: HINSTANCE, hwnd: HWND) -> NOTIFYICONDATAW {
    // Show an icon in the tray
    let hicon = LoadIconW(hinstance, make_int_resource(ID_TRAY_ICON)).unwrap();
    let nid = NOTIFYICONDATAW {
        cbSize: core::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uFlags: NIF_ICON | NIF_GUID | NIF_MESSAGE,
        guidItem: GUID::new().unwrap(),
        hIcon: hicon,
        uCallbackMessage: WM_USER_TRAY_NOTIFICATION,
        Anonymous: NOTIFYICONDATAW_0 {
            uVersion: NOTIFYICON_VERSION_4,
        },
        ..Default::default()
    };

    Shell_NotifyIconW(NIM_ADD, &nid);
    Shell_NotifyIconW(NIM_SETVERSION, &nid);

    nid
}

unsafe fn message_loop() {
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).into() {
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER => {
            // F15 key
            let ip: INPUT = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_F15,
                        ..Default::default()
                    },
                },
            };
            SendInput(&[ip], std::mem::size_of::<INPUT>() as i32);
        }
        WM_COMMAND => {
            // Menu commands
            if w_param.0 == CMD_MENU_QUIT {
                PostQuitMessage(0);
            }
        }
        WM_USER_TRAY_NOTIFICATION => {
            // Tray events
            if l_param.0 as u32 == WM_CONTEXTMENU {
                // Show context menu
                let (x, y) = get_mouse_pos(w_param.0);
                let _ = CreatePopupMenu().map(|hmenu| {
                    // Tray needs hwnd to be foreground and a postmsg hack after for the context menu to behave properly
                    SetForegroundWindow(hwnd);
                    let _ = AppendMenuW(hmenu, MF_STRING | MF_ENABLED, CMD_MENU_QUIT, w!("Quit"));
                    let _ =
                        TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_BOTTOMALIGN, x, y, 0, hwnd, None);
                    let _ = DestroyMenu(hmenu);
                    let _ = PostMessageW(hwnd, WM_NULL, WPARAM::default(), LPARAM::default());
                });
            }
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, w_param, l_param)
}
