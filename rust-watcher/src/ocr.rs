#![allow(dead_code)]
//! Screen capture and OCR for aw-watcher-enhanced.
//!
//! macOS: ScreenCaptureKit (capture) + Vision framework (OCR).
//! Captures the current screen and extracts text using Apple's native
//! VNRecognizeTextRequest. Zero external dependencies for OCR.

use log::{info, warn};
use std::collections::HashSet;
use std::time::Instant;

/// OCR result with extracted text and keywords.
#[derive(Debug, Clone, Default)]
pub struct OcrResult {
    pub full_text: String,
    pub keywords: Vec<String>,
}

/// Screen capture + OCR engine.
pub struct OcrEngine {
    last_capture_time: Instant,
    min_interval: f64,
    max_keywords: usize,
    available: bool,
    debug: bool,
}

impl OcrEngine {
    pub fn new(min_interval: f64, max_keywords: usize) -> Self {
        Self::new_with_debug(min_interval, max_keywords, false)
    }

    pub fn new_with_debug(min_interval: f64, max_keywords: usize, debug: bool) -> Self {
        let available = cfg!(any(target_os = "macos", target_os = "windows"));
        #[cfg(target_os = "macos")]
        info!("OCR engine: Apple Vision (ScreenCaptureKit + VNRecognizeTextRequest)");
        #[cfg(target_os = "windows")]
        info!("OCR engine: Windows.Media.Ocr (Win32 GDI capture + WinRT)");
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        info!("OCR engine: not available on this platform");

        if debug {
            info!(
                "[debug] OCR engine ready (min_interval={min_interval}s, max_keywords={max_keywords})"
            );
        }

        Self {
            last_capture_time: Instant::now() - std::time::Duration::from_secs(999),
            min_interval,
            max_keywords,
            available,
            debug,
        }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Capture the screen and run OCR. Returns None if too soon or unavailable.
    /// If `window_id` is provided, captures only that window; otherwise full screen.
    pub fn capture_and_ocr(&mut self) -> Option<OcrResult> {
        self.capture_and_ocr_window(None)
    }

    /// Capture a specific window (macOS: CGWindowID, Windows: HWND raw value).
    /// Pass `None` to capture the whole foreground screen.
    pub fn capture_and_ocr_window(&mut self, window_id: Option<u64>) -> Option<OcrResult> {
        if !self.available {
            if self.debug {
                info!("[debug] OCR skipped: engine unavailable on this platform");
            }
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_capture_time).as_secs_f64();
        if elapsed < self.min_interval {
            if self.debug {
                info!(
                    "[debug] OCR throttled: {:.2}s since last capture (min_interval={}s)",
                    elapsed, self.min_interval
                );
            }
            return None;
        }
        self.last_capture_time = now;

        if self.debug {
            info!("[debug] OCR capture starting (window_id={:?})", window_id);
        }
        let started = Instant::now();

        let outcome: Option<OcrResult>;

        #[cfg(target_os = "macos")]
        {
            let wid_u32 = window_id.map(|w| w as u32);
            outcome = match macos::capture_and_ocr(self.max_keywords, wid_u32) {
                Ok(result) => Some(result),
                Err(e) => {
                    warn!("OCR capture failed: {e}");
                    None
                }
            };
        }

        #[cfg(target_os = "windows")]
        {
            outcome = match windows_ocr::capture_and_ocr(self.max_keywords, window_id, self.debug) {
                Ok(result) => Some(result),
                Err(e) => {
                    warn!("OCR capture failed: {e}");
                    None
                }
            };
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = window_id;
            outcome = None;
        }

        if self.debug {
            let elapsed_ms = started.elapsed().as_millis();
            match &outcome {
                Some(r) => {
                    let text_preview: String = r
                        .full_text
                        .chars()
                        .take(120)
                        .collect::<String>()
                        .replace('\n', " / ");
                    info!(
                        "[debug] OCR done in {elapsed_ms}ms: {} chars, {} keywords | preview: {}",
                        r.full_text.len(),
                        r.keywords.len(),
                        text_preview
                    );
                }
                None => info!("[debug] OCR done in {elapsed_ms}ms: no result"),
            }
        }

        outcome
    }
}

/// Extract significant keywords from OCR text.
fn extract_keywords(text: &str, max: usize) -> Vec<String> {
    // Common stop words to skip
    static STOP_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
        "one", "our", "out", "has", "have", "from", "been", "some", "them", "than", "this",
        "that", "with", "will", "each", "make", "like", "just", "over", "such", "take",
        "file", "edit", "view", "help", "window", "menu", "new", "open", "save", "close",
        "copy", "paste", "undo", "redo", "find", "replace", "select", "delete",
    ];

    let stop_set: HashSet<&str> = STOP_WORDS.iter().copied().collect();

    let mut seen = HashSet::new();
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|w| {
            let lower = w.to_lowercase();
            w.len() >= 4
                && w.len() <= 40
                && !stop_set.contains(lower.as_str())
                && w.chars().any(|c| c.is_alphabetic())
                && seen.insert(lower)
        })
        .take(max)
        .map(|w| w.to_string())
        .collect()
}

// ─── macOS implementation ────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::{extract_keywords, OcrResult};
    use std::ffi::c_void;

    // ScreenCaptureKit capture via CGWindowList (simpler, no permission prompts for own display)
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGMainDisplayID() -> u32;
        fn CGDisplayBounds(display: u32) -> CGRect;
        fn CGWindowListCreateImage(
            screen_bounds: CGRect,
            list_option: u32,
            window_id: u32,
            image_option: u32,
        ) -> *const c_void; // CGImageRef
        fn CGImageGetWidth(image: *const c_void) -> usize;
        fn CGImageGetHeight(image: *const c_void) -> usize;
    }

    #[link(name = "Foundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    // Vision framework
    #[link(name = "Vision", kind = "framework")]
    extern "C" {}

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    // CGWindowListOption
    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;
    // CGWindowImageOption
    const K_CG_WINDOW_IMAGE_DEFAULT: u32 = 0;
    const K_CG_NULL_WINDOW_ID: u32 = 0;

    /// Capture the focused window (or full screen as fallback) and run Vision OCR.
    pub fn capture_and_ocr(max_keywords: usize, window_id: Option<u32>) -> Result<OcrResult, String> {
        unsafe {
            // Create an autorelease pool to drain all autoreleased ObjC objects
            // (NSArray, VNRecognizedText, etc.) created during OCR.
            // Without this, autoreleased objects accumulate forever in non-main threads.
            #[link(name = "objc", kind = "dylib")]
            extern "C" {
                fn objc_getClass(name: *const u8) -> *const c_void;
                fn sel_registerName(name: *const u8) -> *const c_void;
            }
            extern "C" { fn objc_msgSend(); }
            type MsgSend0 = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
            let send0: MsgSend0 = std::mem::transmute(objc_msgSend as *const c_void);
            let pool = send0(
                send0(objc_getClass(b"NSAutoreleasePool\0".as_ptr()), sel_registerName(b"alloc\0".as_ptr())),
                sel_registerName(b"init\0".as_ptr()),
            );

            // 1. Capture the focused window if we have its ID, else full screen.
            // CGWindowListCreateImage with a specific window_id and
            // kCGWindowListOptionIncludingWindow captures just that window.
            let image = if let Some(wid) = window_id {
                // kCGWindowListOptionIncludingWindow = (1 << 3) = 8
                // kCGWindowImageBoundsIgnoreFraming = (1 << 0) = 1
                CGWindowListCreateImage(
                    CGRect {
                        origin: CGPoint { x: 0.0, y: 0.0 },
                        size: CGSize { width: 0.0, height: 0.0 },
                    }, // CGRectNull — auto-size to the window bounds
                    8, // kCGWindowListOptionIncludingWindow
                    wid,
                    1, // kCGWindowImageBoundsIgnoreFraming
                )
            } else {
                let display_id = CGMainDisplayID();
                let bounds = CGDisplayBounds(display_id);
                CGWindowListCreateImage(
                    bounds,
                    K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
                    K_CG_NULL_WINDOW_ID,
                    K_CG_WINDOW_IMAGE_DEFAULT,
                )
            };

            if image.is_null() {
                send0(pool, sel_registerName(b"drain\0".as_ptr()));
                return Err("Failed to capture screen".into());
            }

            let _width = CGImageGetWidth(image);
            let _height = CGImageGetHeight(image);

            // 2. Run Vision OCR using objc_msgSend
            let text = run_vision_ocr(image);
            CFRelease(image);

            // 3. Drain the autorelease pool before returning
            send0(pool, sel_registerName(b"drain\0".as_ptr()));

            let text = text?;

            // 4. Extract keywords
            let keywords = extract_keywords(&text, max_keywords);

            Ok(OcrResult {
                full_text: text,
                keywords,
            })
        }
    }

    /// Run VNRecognizeTextRequest on a CGImage using Objective-C runtime.
    ///
    /// On ARM64 macOS, `objc_msgSend` uses the standard C calling convention
    /// (register-based), NOT a variadic calling convention. Rust's `extern "C" fn(...)`
    /// variadic declaration causes the compiler to use the wrong ABI (stack-based for
    /// variadic args). We must cast `objc_msgSend` to typed function pointers for every
    /// call that passes arguments beyond (receiver, selector).
    unsafe fn run_vision_ocr(cg_image: *const c_void) -> Result<String, String> {
        use core_foundation::base::TCFType;
        use core_foundation::string::CFString;

        #[link(name = "objc", kind = "dylib")]
        extern "C" {
            fn objc_getClass(name: *const u8) -> *const c_void;
            fn sel_registerName(name: *const u8) -> *const c_void;
        }

        // The raw objc_msgSend symbol — we transmute it for each call site.
        extern "C" {
            fn objc_msgSend();
        }
        let msg_send = objc_msgSend as *const c_void;

        macro_rules! cls {
            ($name:expr) => {
                objc_getClass(concat!($name, "\0").as_ptr())
            };
        }

        macro_rules! sel {
            ($name:expr) => {
                sel_registerName(concat!($name, "\0").as_ptr())
            };
        }

        // Typed function pointer aliases for objc_msgSend
        type MsgSend0 = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
        type MsgSend1Ptr = unsafe extern "C" fn(*const c_void, *const c_void, *const c_void) -> *const c_void;
        type MsgSend1Int = unsafe extern "C" fn(*const c_void, *const c_void, i64) -> *const c_void;
        type MsgSend2Ptr = unsafe extern "C" fn(*const c_void, *const c_void, *const c_void, *const c_void) -> *const c_void;
        type MsgSend1Usize = unsafe extern "C" fn(*const c_void, *const c_void, usize) -> *const c_void;

        let send0: MsgSend0 = std::mem::transmute(msg_send);
        let send1p: MsgSend1Ptr = std::mem::transmute(msg_send);
        let send1i: MsgSend1Int = std::mem::transmute(msg_send);
        let send2p: MsgSend2Ptr = std::mem::transmute(msg_send);
        let send1u: MsgSend1Usize = std::mem::transmute(msg_send);

        // VNRecognizeTextRequest *request = [[VNRecognizeTextRequest alloc] init];
        let vn_class = cls!("VNRecognizeTextRequest");
        let request = send0(send0(vn_class, sel!("alloc")), sel!("init"));
        if request.is_null() {
            return Err("Failed to create VNRecognizeTextRequest".into());
        }

        // [request setRecognitionLevel:VNRequestTextRecognitionLevelAccurate] (1)
        send1i(request, sel!("setRecognitionLevel:"), 1i64);
        // [request setUsesLanguageCorrection:YES]
        send1i(request, sel!("setUsesLanguageCorrection:"), 1i64);

        // VNImageRequestHandler *handler = [[VNImageRequestHandler alloc] initWithCGImage:cg_image options:nil];
        let handler_class = cls!("VNImageRequestHandler");
        let handler = send2p(
            send0(handler_class, sel!("alloc")),
            sel!("initWithCGImage:options:"),
            cg_image,
            std::ptr::null(),
        );
        if handler.is_null() {
            CFRelease(request);
            return Err("Failed to create VNImageRequestHandler".into());
        }

        // NSArray *requests = @[request];
        let requests = send1p(cls!("NSArray"), sel!("arrayWithObject:"), request);

        // [handler performRequests:requests error:&error];
        let mut error: *const c_void = std::ptr::null();
        type MsgSendPerform = unsafe extern "C" fn(
            *const c_void, *const c_void, *const c_void, *mut *const c_void,
        ) -> *const c_void;
        let send_perform: MsgSendPerform = std::mem::transmute(msg_send);
        let result = send_perform(
            handler,
            sel!("performRequests:error:"),
            requests,
            &mut error,
        );
        let success = !result.is_null() && result as usize != 0;

        if !success || !error.is_null() {
            CFRelease(request);
            CFRelease(handler);
            return Err("Vision OCR request failed".into());
        }

        // NSArray<VNRecognizedTextObservation *> *observations = request.results;
        let observations = send0(request, sel!("results"));

        let mut full_text = String::new();

        if !observations.is_null() {
            let count = send0(observations, sel!("count")) as usize;

            for i in 0..count {
                let observation = send1u(observations, sel!("objectAtIndex:"), i);
                if observation.is_null() {
                    continue;
                }

                // NSArray<VNRecognizedText *> *candidates = [observation topCandidates:1];
                let candidates = send1u(observation, sel!("topCandidates:"), 1usize);
                if candidates.is_null() {
                    continue;
                }

                let candidate_count = send0(candidates, sel!("count")) as usize;
                if candidate_count == 0 {
                    continue;
                }

                let candidate = send1u(candidates, sel!("objectAtIndex:"), 0usize);
                if candidate.is_null() {
                    continue;
                }

                // NSString *text = candidate.string;
                let ns_string = send0(candidate, sel!("string"));
                if ns_string.is_null() {
                    continue;
                }

                // Convert NSString to Rust String via CFString
                let cf_str = CFString::wrap_under_get_rule(ns_string as _);
                let line = cf_str.to_string();
                if !line.is_empty() {
                    full_text.push_str(&line);
                    full_text.push('\n');
                }
            }
        }

        CFRelease(handler);
        CFRelease(request);

        Ok(full_text)
    }
}

// ─── Windows implementation ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_ocr {
    use super::{extract_keywords, OcrResult};
    use log::info;
    use windows::Foundation::IAsyncOperation;
    use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine as WinOcrEngine;
    use windows::Storage::Streams::Buffer;
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HBITMAP, HDC, SRCCOPY,
    };
    use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClientRect, GetDesktopWindow, GetWindowRect, PW_RENDERFULLCONTENT,
    };

    /// Top-level Windows OCR entry point. Captures the target window (or the
    /// desktop if `window_id` is None) and runs `Windows.Media.Ocr` over it.
    pub fn capture_and_ocr(
        max_keywords: usize,
        window_id: Option<u64>,
        debug: bool,
    ) -> Result<OcrResult, String> {
        unsafe {
            let hwnd = match window_id {
                Some(raw) if raw != 0 => HWND(raw as *mut _),
                _ => GetDesktopWindow(),
            };

            let pixels = capture_window_pixels(hwnd, debug)?;
            if debug {
                let (nonzero, total) = pixel_sample(&pixels);
                info!(
                    "[debug] OCR capture buffer: {}x{}, {} bytes, {}/{} sampled pixels non-zero ({})",
                    pixels.width,
                    pixels.height,
                    pixels.bgra.len(),
                    nonzero,
                    total,
                    pixels.source
                );
            }
            let text = run_windows_ocr(&pixels)?;
            let keywords = extract_keywords(&text, max_keywords);
            Ok(OcrResult {
                full_text: text,
                keywords,
            })
        }
    }

    /// Sample ~256 evenly-spaced pixels and count how many are non-black.
    /// Used as a quick "did we get a real image vs. a black DWM scratch buffer"
    /// heuristic. Returns (non_zero, sampled).
    fn pixel_sample(pixels: &CapturedBitmap) -> (usize, usize) {
        let total = pixels.bgra.len() / 4;
        if total == 0 {
            return (0, 0);
        }
        let sample_count = total.min(256);
        let step = (total / sample_count).max(1);
        let mut nonzero = 0usize;
        let mut sampled = 0usize;
        let mut i = 0usize;
        while i < total && sampled < sample_count {
            let off = i * 4;
            // Look at B, G, R only (ignore alpha — DWM may leave alpha = 0).
            if pixels.bgra[off] != 0 || pixels.bgra[off + 1] != 0 || pixels.bgra[off + 2] != 0 {
                nonzero += 1;
            }
            sampled += 1;
            i += step;
        }
        (nonzero, sampled)
    }

    /// 32-bit BGRA bitmap captured from a window's client area.
    struct CapturedBitmap {
        width: i32,
        height: i32,
        /// Row-major, top-down, 4 bytes/pixel (B, G, R, A).
        bgra: Vec<u8>,
        /// Tag for debug logs: which capture path produced this buffer.
        source: &'static str,
    }

    /// Capture the foreground window into a top-down 32bpp BGRA buffer.
    ///
    /// Path A — PrintWindow with PW_RENDERFULLCONTENT (Win8.1+). DWM renders
    /// the actual composited content into our DC. Works for modern XAML/UWP
    /// apps, Windows Terminal, WPF, Edge, Office, etc., which a plain BitBlt
    /// from a window DC would only see as black.
    ///
    /// Path B — fallback BitBlt from the desktop DC at the window's screen
    /// coordinates. Always works for visible windows but also picks up any
    /// overlapping windows. Used when PrintWindow fails or returns an
    /// all-black frame.
    unsafe fn capture_window_pixels(hwnd: HWND, debug: bool) -> Result<CapturedBitmap, String> {
        // Use the client rect for size (excludes title bar / borders).
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_err() {
            return Err("GetClientRect failed".into());
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width <= 0 || height <= 0 {
            return Err(format!("invalid window size {width}x{height}"));
        }

        // First attempt: PrintWindow path.
        match capture_via_printwindow(hwnd, width, height, debug) {
            Ok(buf) if !is_all_black(&buf.bgra) => return Ok(buf),
            Ok(_) if debug => info!("[debug] PrintWindow returned all-black, falling back to screen-region BitBlt"),
            Err(e) if debug => info!("[debug] PrintWindow failed ({e}), falling back to screen-region BitBlt"),
            _ => {}
        }

        // Second attempt: BitBlt from the desktop DC at the window's screen rect.
        capture_via_screen_region(hwnd)
    }

    /// Heuristic: returns true if every byte in the buffer is zero. Cheap
    /// because we early-exit on the first non-zero byte.
    fn is_all_black(bgra: &[u8]) -> bool {
        !bgra.iter().any(|&b| b != 0)
    }

    /// Internal: PrintWindow path. Asks DWM to render the composited window
    /// contents into a memory DC, then GetDIBits into a top-down BGRA buffer.
    unsafe fn capture_via_printwindow(
        hwnd: HWND,
        width: i32,
        height: i32,
        debug: bool,
    ) -> Result<CapturedBitmap, String> {
        let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
        if hdc_screen.is_invalid() {
            return Err("GetDC(NULL) returned null".into());
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            return Err("CreateCompatibleDC failed".into());
        }
        let hbitmap: HBITMAP = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            return Err("CreateCompatibleBitmap failed".into());
        }
        let prev = SelectObject(hdc_mem, hbitmap);

        // PW_RENDERFULLCONTENT (0x2) forces full DWM rendering. Without this
        // flag PrintWindow can return black for DirectComposition-rendered
        // surfaces like Windows Terminal, modern Edge, and many UWP apps.
        let pw_ok = PrintWindow(hwnd, hdc_mem, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool();
        if debug {
            info!("[debug] PrintWindow(PW_RENDERFULLCONTENT) ok={pw_ok}");
        }

        let bgra = dib_extract(hdc_mem, hbitmap, width, height);

        SelectObject(hdc_mem, prev);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

        if !pw_ok {
            return Err("PrintWindow returned FALSE".into());
        }
        let bgra = bgra.ok_or_else(|| "GetDIBits returned 0 scanlines".to_string())?;
        Ok(CapturedBitmap {
            width,
            height,
            bgra,
            source: "PrintWindow",
        })
    }

    /// Internal: screen-region path. BitBlts from the desktop DC at the
    /// window's screen coordinates. Always works for visible windows.
    unsafe fn capture_via_screen_region(hwnd: HWND) -> Result<CapturedBitmap, String> {
        let mut win_rect = RECT::default();
        if GetWindowRect(hwnd, &mut win_rect).is_err() {
            return Err("GetWindowRect failed".into());
        }
        let width = win_rect.right - win_rect.left;
        let height = win_rect.bottom - win_rect.top;
        if width <= 0 || height <= 0 {
            return Err(format!("invalid screen rect {width}x{height}"));
        }

        let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
        if hdc_screen.is_invalid() {
            return Err("GetDC(NULL) returned null".into());
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            return Err("CreateCompatibleDC failed".into());
        }
        let hbitmap: HBITMAP = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
            return Err("CreateCompatibleBitmap failed".into());
        }
        let prev = SelectObject(hdc_mem, hbitmap);

        let blt = BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            hdc_screen,
            win_rect.left,
            win_rect.top,
            SRCCOPY,
        );

        let bgra = dib_extract(hdc_mem, hbitmap, width, height);

        SelectObject(hdc_mem, prev);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

        if blt.is_err() {
            return Err("BitBlt from screen DC failed".into());
        }
        let bgra = bgra.ok_or_else(|| "GetDIBits returned 0 scanlines".to_string())?;
        Ok(CapturedBitmap {
            width,
            height,
            bgra,
            source: "ScreenRegion",
        })
    }

    /// Read pixels out of an HBITMAP as a top-down 32bpp BGRA buffer.
    /// Negative biHeight asks GDI for a top-down DIB, matching what
    /// Windows.Graphics.Imaging expects for the Bgra8 pixel format.
    unsafe fn dib_extract(hdc_mem: HDC, hbitmap: HBITMAP, width: i32, height: i32) -> Option<Vec<u8>> {
        let stride = (width as usize) * 4;
        let mut bgra = vec![0u8; stride * height as usize];
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let scanlines = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr() as *mut _),
            &mut info,
            DIB_RGB_COLORS,
        );
        if scanlines == 0 {
            None
        } else {
            Some(bgra)
        }
    }

    /// Build a SoftwareBitmap from BGRA bytes and run the system OCR engine.
    /// Blocks on the WinRT IAsyncOperation via `.get()`.
    unsafe fn run_windows_ocr(pixels: &CapturedBitmap) -> Result<String, String> {
        // 1. Allocate an IBuffer-compatible Buffer and copy our BGRA bytes
        //    into it through IBufferByteAccess.
        let buffer = Buffer::Create(pixels.bgra.len() as u32)
            .map_err(|e| format!("Buffer::Create failed: {e}"))?;
        copy_into_buffer(&buffer, &pixels.bgra)?;

        // SoftwareBitmap's BGRA8 format requires alpha to be Premultiplied or
        // Ignore. Our BitBlt produced opaque pixels, so Ignore is correct and
        // doesn't trigger the premultiplied validation path.
        let software_bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
            &buffer,
            BitmapPixelFormat::Bgra8,
            pixels.width,
            pixels.height,
            BitmapAlphaMode::Ignore,
        )
        .map_err(|e| format!("SoftwareBitmap::CreateCopyWithAlphaFromBuffer failed: {e}"))?;

        // 2. Instantiate the system OCR engine using the user's languages.
        let engine = WinOcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|e| format!("TryCreateFromUserProfileLanguages failed: {e}"))?;

        // 3. Run recognition. RecognizeAsync returns an IAsyncOperation; we
        //    block on it because the enrichment thread is already a worker.
        let op: IAsyncOperation<_> = engine
            .RecognizeAsync(&software_bitmap)
            .map_err(|e| format!("RecognizeAsync failed: {e}"))?;
        let result = op
            .get()
            .map_err(|e| format!("OCR operation faulted: {e}"))?;

        let hstr = result
            .Text()
            .map_err(|e| format!("OcrResult.Text failed: {e}"))?;

        Ok(hstr.to_string_lossy())
    }

    /// Copy bytes into a WinRT Buffer via IBufferByteAccess + memcpy.
    /// SetLength must be set explicitly — newly-created Buffers report
    /// Length == 0 regardless of Capacity, and Imaging APIs read Length.
    unsafe fn copy_into_buffer(buffer: &Buffer, bytes: &[u8]) -> Result<(), String> {
        use windows::core::Interface;
        use windows::Storage::Streams::IBuffer;
        use windows::Win32::System::WinRT::IBufferByteAccess;

        let access: IBufferByteAccess = buffer
            .cast()
            .map_err(|e| format!("IBufferByteAccess cast failed: {e}"))?;
        let ptr = access
            .Buffer()
            .map_err(|e| format!("IBufferByteAccess.Buffer failed: {e}"))?;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());

        let ibuf: IBuffer = buffer
            .cast()
            .map_err(|e| format!("IBuffer cast failed: {e}"))?;
        ibuf.SetLength(bytes.len() as u32)
            .map_err(|e| format!("IBuffer.SetLength failed: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let text = "Visual Studio Code main.py dagtech-api src/routes/auth.py def login_user password_hash database connection pool";
        let kw = extract_keywords(text, 10);
        assert!(!kw.is_empty());
        assert!(kw.len() <= 10);
        // Should skip short words and stop words
        assert!(!kw.contains(&"def".to_string()));
        assert!(!kw.contains(&"the".to_string()));
    }

    #[test]
    fn test_extract_keywords_dedup() {
        let text = "Rust rust RUST programming Programming language";
        let kw = extract_keywords(text, 10);
        // "rust" should appear only once (case-insensitive dedup)
        let rust_count = kw.iter().filter(|w| w.to_lowercase() == "rust").count();
        assert_eq!(rust_count, 1);
    }

    #[test]
    fn test_extract_keywords_empty() {
        let kw = extract_keywords("", 10);
        assert!(kw.is_empty());
    }

    #[test]
    fn test_ocr_engine_unavailable_platform() {
        // On non-macOS this should be unavailable
        let engine = OcrEngine::new(5.0, 20);
        if !cfg!(target_os = "macos") {
            assert!(!engine.is_available());
        }
    }

    #[test]
    fn test_ocr_engine_creation() {
        // Just verify the engine can be created without panic
        let engine = OcrEngine::new(5.0, 20);
        // On macOS it should be available
        if cfg!(target_os = "macos") {
            assert!(engine.is_available());
        }
    }
}
