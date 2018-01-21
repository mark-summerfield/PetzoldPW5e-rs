// Transliterated from Charles Petzold's Programming Windows 5e
// http://www.charlespetzold.com/pw5/index.html
//
// Chapter 4 - SysMets3
//
// The original source code copyright:
//
// SYSMETS3.C -- System Metrics Display Program No. 2
//               (c) Charles Petzold, 1998
//
#![windows_subsystem = "windows"]

#![cfg(windows)] extern crate winapi;
extern crate sys_mets;

use sys_mets::SYS_METRICS;
use std::mem;
use std::cmp;
use std::ptr::{null_mut, null};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::ctypes::c_int;
use winapi::um::winuser::{CreateWindowExW, DefWindowProcW, PostQuitMessage, RegisterClassExW,
                          ShowWindow, UpdateWindow, GetMessageW, TranslateMessage, DispatchMessageW,
                          BeginPaint, EndPaint, MessageBoxW, LoadIconW, LoadCursorW, GetDC,
                          ReleaseDC, GetSystemMetrics, SetScrollInfo, GetScrollInfo, ScrollWindow,
                          MSG, PAINTSTRUCT, WNDCLASSEXW, SCROLLINFO,
                          WM_CREATE, WM_DESTROY, WM_PAINT, WM_SIZE, WM_VSCROLL, WM_HSCROLL,
                          WS_OVERLAPPEDWINDOW, WS_VSCROLL, WS_HSCROLL, SW_SHOW, CS_HREDRAW,
                          CS_VREDRAW, IDC_ARROW, IDI_APPLICATION, MB_ICONERROR, CW_USEDEFAULT,
                          SB_TOP, SB_BOTTOM, SB_LINEUP, SB_LINEDOWN, SB_LINELEFT,
                          SB_LINERIGHT, SB_PAGEUP, SB_PAGEDOWN, SB_PAGELEFT, SB_PAGERIGHT,
                          SB_THUMBPOSITION, SIF_ALL, SIF_RANGE, SIF_PAGE, SIF_POS, };
use winapi::um::wingdi::{GetStockObject, GetTextMetricsW, TextOutW, SetTextAlign,
                         TEXTMETRICW,
                         TA_LEFT, TA_RIGHT, TA_TOP, };
use winapi::um::winbase::lstrlenW;
use winapi::shared::minwindef::{HIWORD, LOWORD, DWORD,
                                UINT, WPARAM, LPARAM, LRESULT, HINSTANCE, TRUE, };
use winapi::shared::windef::{HWND, HBRUSH};
use winapi::shared::ntdef::LPCWSTR;

// There are some mismatches in winapi types between constants and their usage...
const WHITE_BRUSH: c_int = winapi::um::wingdi::WHITE_BRUSH as c_int;
const SB_VERT: c_int = winapi::um::winuser::SB_VERT as c_int;
const SB_HORZ: c_int = winapi::um::winuser::SB_HORZ as c_int;


fn to_wstring(str: &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(once(0)).collect()
}


fn main() {
    let app_name = to_wstring("sys_mets3");
    let hinstance = 0 as HINSTANCE;

    unsafe {
        let wndclassex = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: GetStockObject(WHITE_BRUSH) as HBRUSH,
            lpszClassName: app_name.as_ptr(),
            hIconSm: null_mut(),
            lpszMenuName: null(),
        };
        let atom = RegisterClassExW(&wndclassex);

        if atom == 0 {
            MessageBoxW(null_mut(),
                        to_wstring("This program requires Windows NT!").as_ptr(),
                        app_name.as_ptr(),
                        MB_ICONERROR);
            return; //   premature exit
        }

        let caption = to_wstring("Get System Metrics No. 3");
        let hwnd = CreateWindowExW(
            0,                 // dwExStyle:
            atom as LPCWSTR,   // lpClassName: class name or atom
            caption.as_ptr(),  // lpWindowName: window caption
            WS_OVERLAPPEDWINDOW | WS_VSCROLL | WS_HSCROLL,  // dwStyle: window style
            CW_USEDEFAULT,     // x: initial x position
            CW_USEDEFAULT,     // y: initial y position
            CW_USEDEFAULT,     // nWidth: initial x size
            CW_USEDEFAULT,     // nHeight: initial y size
            null_mut(),        // hWndParent: parent window handle
            null_mut(),        // hMenu: window menu handle
            hinstance,         // hInstance: program instance handle
            null_mut());       // lpParam: creation parameters

        if hwnd.is_null() {
            return;  // premature exit
        }

        ShowWindow(hwnd, SW_SHOW);
        if UpdateWindow(hwnd) == 0 {
            return;  // premature exit
        }

        let mut msg: MSG = mem::uninitialized();

        loop {
            // three states: -1, 0 or non-zero
            let ret = GetMessageW(&mut msg, null_mut(), 0, 0);

            if ret == -1 {
                // handle the error and/or exit
                // for error call GetLastError();
                return;
            } else if ret == 0 {
                break;
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
// return msg.wParam;  // WM_QUIT
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND,
                                   message: UINT,
                                   wparam: WPARAM,
                                   lparam: LPARAM)
                                   -> LRESULT {
    static mut CX_CAPS: c_int = 0;
    static mut CX_CHAR: c_int = 0;
    static mut CY_CHAR: c_int = 0;
    static mut CX_CLIENT: c_int = 0;
    static mut CY_CLIENT: c_int = 0;
    static mut MAX_WIDTH: c_int = 0;

    match message {
        WM_CREATE => {
            let hdc = GetDC(hwnd);
            let mut tm: TEXTMETRICW = mem::uninitialized();

            GetTextMetricsW(hdc, &mut tm);
            CX_CHAR = tm.tmAveCharWidth;
            CX_CAPS = (if tm.tmPitchAndFamily & 1 == 1 { 3 } else { 2 }) * CX_CHAR / 2;
            CY_CHAR = tm.tmHeight + tm.tmExternalLeading;

            ReleaseDC(hwnd, hdc);

            // Save the width of the three columns

            MAX_WIDTH = 40 * CX_CHAR + 22 * CX_CAPS;

            0 as LRESULT  // message processed
        }

        WM_SIZE => {
            CX_CLIENT = LOWORD(lparam as DWORD) as c_int;
            CY_CLIENT = HIWORD(lparam as DWORD) as c_int;

            // Set vertical scroll bar range and page size

            let mut si = SCROLLINFO {
                cbSize: mem::size_of::<SCROLLINFO>() as UINT,
                fMask: SIF_RANGE | SIF_PAGE,
                nMin: 0,
                nMax: SYS_METRICS.len() as c_int - 1,
                nPage: (CY_CLIENT / CY_CHAR) as UINT,
                nPos: 0,
                nTrackPos: 0,
            };

            SetScrollInfo(hwnd, SB_VERT, &si, TRUE);

            // Set horizontal scroll bar range and page size

            si = SCROLLINFO {
                nMax: 2 + MAX_WIDTH / CX_CHAR,
                nPage: (CX_CLIENT / CX_CHAR) as UINT,
                ..si
            };
            SetScrollInfo(hwnd, SB_HORZ, &si, TRUE);

            0 as LRESULT  // message processed
        }

        WM_VSCROLL => {

            // Get all the vertical scroll bar information

            let mut si: SCROLLINFO = SCROLLINFO {
                cbSize: mem::size_of::<SCROLLINFO>() as UINT,
                fMask: SIF_ALL,
                ..mem::uninitialized()
            };

            GetScrollInfo(hwnd, SB_VERT, &mut si);

            // Save the position for comparison later on

            let vert_pos = si.nPos;

            match LOWORD(wparam as DWORD) as LPARAM {
                SB_TOP => { si.nPos = si.nMin; }
                SB_BOTTOM => { si.nPos = si.nMax; }
                SB_LINEUP => { si.nPos -= 1; }
                SB_LINEDOWN => { si.nPos += 1; }
                SB_PAGEUP => { si.nPos -= si.nPage as c_int; }
                SB_PAGEDOWN => { si.nPos += si.nPage as c_int; }
                SB_THUMBPOSITION => { si.nPos = si.nTrackPos; }
                _ => {}
            }

            // Set the position and then retrieve it.  Due to adjustments
            // by Windows it may not be the same as the value set.

            si.fMask = SIF_POS;
            SetScrollInfo(hwnd, SB_VERT, &si, TRUE);
            GetScrollInfo(hwnd, SB_VERT, &mut si);

            // If the position has changed, scroll the window.

            if si.nPos != vert_pos {
                ScrollWindow(hwnd, 0, CY_CHAR * (vert_pos - si.nPos), null(), null());
                UpdateWindow(hwnd);
            }

            0 as LRESULT
        }

        WM_HSCROLL => {
            // Get all the horizontal scroll bar information

            let mut si: SCROLLINFO = SCROLLINFO {
                cbSize: mem::size_of::<SCROLLINFO>() as UINT,
                fMask: SIF_ALL,
                ..mem::uninitialized()
            };

            GetScrollInfo(hwnd, SB_HORZ, &mut si);

            // Save the position for comparison later on

            let horz_pos = si.nPos;

            match LOWORD(wparam as DWORD) as LPARAM {
                SB_LINELEFT => { si.nPos -= 1; }
                SB_LINERIGHT => { si.nPos += 1; }
                SB_PAGELEFT => { si.nPos -= si.nPage as c_int; }
                SB_PAGERIGHT => { si.nPos += si.nPage as c_int; }
                SB_THUMBPOSITION => { si.nPos = si.nTrackPos; }
                _ => {}
            }

            // Set the position and then retrieve it.  Due to adjustments
            // by Windows it may not be the same as the value set.

            si.fMask = SIF_POS;
            SetScrollInfo(hwnd, SB_HORZ, &si, TRUE);
            GetScrollInfo(hwnd, SB_HORZ, &mut si);

            // If the position has changed, scroll the window.

            if si.nPos != horz_pos {
                ScrollWindow(hwnd, CX_CHAR * (horz_pos - si.nPos), 0, null(), null());
                UpdateWindow(hwnd);
            }

            0 as LRESULT
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::uninitialized();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut si: SCROLLINFO = mem::uninitialized();
            si = SCROLLINFO {
                cbSize: mem::size_of::<SCROLLINFO>() as UINT,
                fMask: SIF_POS,
                ..si
            };

            // Get vertical scroll bar position

            GetScrollInfo(hwnd, SB_VERT, &mut si);
            let vert_pos = si.nPos;

            // Get horizontal scroll bar position

            GetScrollInfo(hwnd, SB_HORZ, &mut si);
            let horz_pos = si.nPos;

            // Find painting limits

            let paint_beg = cmp::max(0, vert_pos + ps.rcPaint.top / CY_CHAR);
            let paint_end = cmp::min(SYS_METRICS.len() as c_int - 1, vert_pos + ps.rcPaint.bottom / CY_CHAR);

            for i in paint_beg..paint_end + 1 {
                let sys_metric = &SYS_METRICS[i as usize];
                let x = CX_CHAR * (1 - horz_pos);
                let y = CY_CHAR * (i - vert_pos);

                SetTextAlign(hdc, TA_LEFT | TA_TOP);

                let label = to_wstring(sys_metric.label);
                TextOutW(hdc,
                         x,
                         y,
                         label.as_ptr(),
                         lstrlenW(label.as_ptr()));

                let desc = to_wstring(sys_metric.desc);
                TextOutW(hdc,
                         x + 22 * CX_CAPS,
                         y,
                         desc.as_ptr(),
                         lstrlenW(desc.as_ptr()));

                SetTextAlign(hdc, TA_RIGHT | TA_TOP);

                let metric = to_wstring(&format!("{:5}", GetSystemMetrics(sys_metric.index)));
                TextOutW(hdc,
                         x + 22 * CX_CAPS + 40 * CX_CHAR,
                         y,
                         metric.as_ptr(),
                         lstrlenW(metric.as_ptr()));
            }

            EndPaint(hwnd, &mut ps);
            0 as LRESULT  // message processed
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0 as LRESULT  // message processed
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}