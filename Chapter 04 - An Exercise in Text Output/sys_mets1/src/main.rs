// Transliterated from Charles Petzold's Programming Windows 5e
// http://www.charlespetzold.com/pw5/index.html
//
// Chapter 4 - SysMets1
//
// The original source code copyright:
//
// SYSMETS1.C -- System Metrics Display Program No. 1
//               (c) Charles Petzold, 1998
//
#![windows_subsystem = "windows"]

#![cfg(windows)]
extern crate winapi;
extern crate sys_mets_data;
extern crate extras;

use sys_mets_data::SYS_METRICS;
use std::mem;
use std::ptr::{null_mut, null};
use winapi::ctypes::c_int;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{CreateWindowExW, DefWindowProcW, PostQuitMessage, RegisterClassExW,
                          ShowWindow, UpdateWindow, GetMessageW, TranslateMessage, DispatchMessageW,
                          BeginPaint, EndPaint, MessageBoxW, LoadIconW, LoadCursorW, GetDC,
                          ReleaseDC, GetSystemMetrics,
                          MSG, PAINTSTRUCT, WNDCLASSEXW,
                          WM_CREATE, WM_DESTROY, WS_OVERLAPPEDWINDOW, WM_PAINT, SW_SHOW, CS_HREDRAW,
                          CS_VREDRAW, IDC_ARROW, IDI_APPLICATION, MB_ICONERROR, CW_USEDEFAULT, };
use winapi::um::wingdi::{GetTextMetricsW, TextOutW, SetTextAlign,
                         TEXTMETRICW,
                         TA_LEFT, TA_RIGHT, TA_TOP, };
use winapi::um::winbase::lstrlenW;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, };
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::LPCWSTR;

// There are some things missing from winapi,
// and some that have been given an interesting interpretation
use extras::{WHITE_BRUSH, to_wstr, GetStockBrush};


fn main() {
    let app_name = to_wstr("sys_mets1");

    unsafe {
        let hinstance = GetModuleHandleW(null());

        let wndclassex = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: GetStockBrush(WHITE_BRUSH),
            lpszClassName: app_name.as_ptr(),
            hIconSm: null_mut(),
            lpszMenuName: null(),
        };
        let atom = RegisterClassExW(&wndclassex);

        if atom == 0 {
            MessageBoxW(null_mut(),
                        to_wstr("This program requires Windows NT!").as_ptr(),
                        app_name.as_ptr(),
                        MB_ICONERROR);
            return; //   premature exit
        }

        let caption = to_wstr("Get System Metrics No. 1");
        let hwnd = CreateWindowExW(
            0,                    // dwExStyle:
            atom as LPCWSTR,      // lpClassName: class name or atom
            caption.as_ptr(),     // lpWindowName: window caption
            WS_OVERLAPPEDWINDOW,  // dwStyle: window style
            CW_USEDEFAULT,        // x: initial x position
            CW_USEDEFAULT,        // y: initial y position
            CW_USEDEFAULT,        // nWidth: initial x size
            CW_USEDEFAULT,        // nHeight: initial y size
            null_mut(),           // hWndParent: parent window handle
            null_mut(),           // hMenu: window menu handle
            hinstance,            // hInstance: program instance handle
            null_mut());          // lpParam: creation parameters

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
    static mut CAPS_WIDTH: c_int = 0;
    static mut CHAR_WIDTH: c_int = 0;
    static mut CHAR_HEIGHT: c_int = 0;

    match message {
        WM_CREATE => {
            let hdc = GetDC(hwnd);
            let mut tm: TEXTMETRICW = mem::uninitialized();

            GetTextMetricsW(hdc, &mut tm);
            CHAR_WIDTH = tm.tmAveCharWidth;
            CAPS_WIDTH = (if tm.tmPitchAndFamily & 1 == 1 { 3 } else { 2 }) * CHAR_WIDTH / 2;
            CHAR_HEIGHT = tm.tmHeight + tm.tmExternalLeading;

            ReleaseDC(hwnd, hdc);

            0 as LRESULT  // message processed
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::uninitialized();
            let hdc = BeginPaint(hwnd, &mut ps);

            for (u, sys_metric) in SYS_METRICS.iter().enumerate() {
                let i = u as c_int;

                SetTextAlign(hdc, TA_LEFT | TA_TOP);

                let label = to_wstr(sys_metric.label);
                TextOutW(hdc,
                         0,
                         CHAR_HEIGHT * i,
                         label.as_ptr(),
                         lstrlenW(label.as_ptr()));

                let desc = to_wstr(sys_metric.desc);
                TextOutW(hdc,
                         22 * CAPS_WIDTH,
                         CHAR_HEIGHT * i,
                         desc.as_ptr(),
                         lstrlenW(desc.as_ptr()));

                SetTextAlign(hdc, TA_RIGHT | TA_TOP);

                let metric = to_wstr(&format!("{:5}", GetSystemMetrics(sys_metric.index)));
                TextOutW(hdc,
                         22 * CAPS_WIDTH + 40 * CHAR_WIDTH,
                         CHAR_HEIGHT * i,
                         metric.as_ptr(),
                         lstrlenW(metric.as_ptr()));
            }

            EndPaint(hwnd, &ps);
            0 as LRESULT  // message processed
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0 as LRESULT  // message processed
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}
