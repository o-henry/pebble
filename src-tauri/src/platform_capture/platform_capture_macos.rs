use crate::{
    capture_backend::{
        byte_len, capture_error, cropped_frame, CaptureError, CaptureErrorCode, CaptureResult,
        RGBA_BYTES_PER_PIXEL,
    },
    region_selection_types::{PhysicalRegion, WindowCaptureTarget},
};
use platform_capture_macos_sys::*;
use screencapturekit::{
    cg::CGRect as ScreenCaptureRect,
    screenshot_manager::{CGImageExt, SCScreenshotManager},
    shareable_content::SCShareableContent,
    stream::{configuration::SCStreamConfiguration, content_filter::SCContentFilter},
};

use super::BackdropColor;

const BACKDROP_SAMPLE_POINT_SIZE: f64 = 128.0;
const MAX_BACKDROP_PIXELS: usize = 1_024;

#[path = "platform_capture_macos_sys.rs"]
mod platform_capture_macos_sys;

pub(super) fn capture_region(region: &PhysicalRegion, scale_factor: f64) -> CaptureResult {
    if !preflight_screen_capture_access() {
        return Err(permission_denied(region));
    }

    if let Some(target) = region.source_window.as_ref() {
        return capture_source_window_region(region, target);
    }

    let image = unsafe {
        CGWindowListCreateImage(
            capture_rect(region, scale_factor),
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
            K_CG_NULL_WINDOW_ID,
            K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING,
        )
    };

    let image = ScopedCfRef::new(image.cast())
        .ok_or_else(|| capture_unavailable(region, "macOS returned no capture image."))?;

    let bytes = unsafe { rgba_bytes_from_image(region, image.as_cg_image()) }?;
    Ok(cropped_frame(region, bytes))
}

pub(super) fn source_window_for_region(
    region: &PhysicalRegion,
    scale_factor: f64,
) -> Option<WindowCaptureTarget> {
    if !preflight_screen_capture_access() {
        return None;
    }

    let selection = capture_rect(region, scale_factor);
    let center = CGPoint {
        x: selection.origin.x + selection.size.width / 2.0,
        y: selection.origin.y + selection.size.height / 2.0,
    };
    let source = topmost_source_window(center)?;
    let relative_x = selection.origin.x - source.bounds.origin.x;
    let relative_y = selection.origin.y - source.bounds.origin.y;

    if relative_x < 0.0
        || relative_y < 0.0
        || relative_x + selection.size.width > source.bounds.size.width
        || relative_y + selection.size.height > source.bounds.size.height
    {
        return None;
    }

    Some(WindowCaptureTarget {
        window_id: source.window_id,
        relative_x_millipoints: to_millipoints(relative_x)?,
        relative_y_millipoints: to_millipoints(relative_y)?,
        width_millipoints: to_unsigned_millipoints(selection.size.width)?,
        height_millipoints: to_unsigned_millipoints(selection.size.height)?,
    })
}

fn capture_source_window_region(
    region: &PhysicalRegion,
    target: &WindowCaptureTarget,
) -> CaptureResult {
    let content = SCShareableContent::create()
        .with_exclude_desktop_windows(true)
        .with_on_screen_windows_only(false)
        .get()
        .map_err(|_| {
            capture_unavailable(region, "macOS could not list the selected source window.")
        })?;
    let windows = content.windows();
    let source = windows
        .iter()
        .find(|window| window.window_id() == target.window_id)
        .ok_or_else(|| {
            capture_unavailable(region, "The selected source window is no longer available.")
        })?;
    let filter = SCContentFilter::create().with_window(source).build();
    let width = u32::try_from(region.width)
        .map_err(|_| capture_unavailable(region, "The selected region width is invalid."))?;
    let height = u32::try_from(region.height)
        .map_err(|_| capture_unavailable(region, "The selected region height is invalid."))?;
    let source_rect = ScreenCaptureRect::new(
        target.relative_x_millipoints as f64 / 1_000.0,
        target.relative_y_millipoints as f64 / 1_000.0,
        target.width_millipoints as f64 / 1_000.0,
        target.height_millipoints as f64 / 1_000.0,
    );
    let configuration = SCStreamConfiguration::new()
        .with_width(width)
        .with_height(height)
        .with_source_rect(source_rect)
        .with_scales_to_fit(false)
        .with_shows_cursor(false);
    let image = SCScreenshotManager::capture_image(&filter, &configuration).map_err(|_| {
        capture_unavailable(
            region,
            "macOS could not capture the selected source window.",
        )
    })?;
    if image.width() != region.width as usize || image.height() != region.height as usize {
        return Err(capture_unavailable(
            region,
            "The selected source window changed display scale or size.",
        ));
    }
    let bytes = image.rgba_data().map_err(|_| {
        capture_unavailable(region, "macOS source window pixel data was not usable.")
    })?;
    if bytes.len() != byte_len(region)? {
        return Err(capture_unavailable(
            region,
            "macOS source window returned an unexpected frame size.",
        ));
    }
    Ok(cropped_frame(region, bytes))
}

#[derive(Clone, Copy)]
struct WindowInfo {
    window_id: u32,
    bounds: CGRect,
}

fn topmost_source_window(point: CGPoint) -> Option<WindowInfo> {
    let list = unsafe {
        CGWindowListCopyWindowInfo(
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
            K_CG_NULL_WINDOW_ID,
        )
    };
    let list = ScopedCfRef::new(list.cast())?;
    let count = unsafe { CFArrayGetCount(list.as_cf_array()) };

    for index in 0..count {
        let dictionary = unsafe { CFArrayGetValueAtIndex(list.as_cf_array(), index) };
        let Some(info) = (unsafe { window_info_from_dictionary(dictionary.cast()) }) else {
            continue;
        };
        if info.owner_pid != std::process::id() as i32
            && info.layer == 0
            && rect_contains_point(info.bounds, point)
        {
            return Some(WindowInfo {
                window_id: info.window_id,
                bounds: info.bounds,
            });
        }
    }

    None
}

struct RawWindowInfo {
    window_id: u32,
    owner_pid: i32,
    layer: i32,
    bounds: CGRect,
}

unsafe fn window_info_from_dictionary(dictionary: CFDictionaryRef) -> Option<RawWindowInfo> {
    if dictionary.is_null() {
        return None;
    }
    let mut bounds = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 0.0,
            height: 0.0,
        },
    };
    let bounds_dictionary = CFDictionaryGetValue(dictionary, K_CG_WINDOW_BOUNDS.cast());
    if bounds_dictionary.is_null()
        || !CGRectMakeWithDictionaryRepresentation(bounds_dictionary.cast(), &mut bounds)
    {
        return None;
    }
    Some(RawWindowInfo {
        window_id: u32::try_from(dictionary_i32(dictionary, K_CG_WINDOW_NUMBER)?).ok()?,
        owner_pid: dictionary_i32(dictionary, K_CG_WINDOW_OWNER_PID)?,
        layer: dictionary_i32(dictionary, K_CG_WINDOW_LAYER)?,
        bounds,
    })
}

unsafe fn dictionary_i32(dictionary: CFDictionaryRef, key: CFStringRef) -> Option<i32> {
    let number = CFDictionaryGetValue(dictionary, key.cast());
    let mut value = 0_i32;
    (!number.is_null()
        && CFNumberGetValue(
            number,
            K_CF_NUMBER_SINT32_TYPE,
            (&mut value as *mut i32).cast(),
        ))
    .then_some(value)
}

fn to_millipoints(value: f64) -> Option<i64> {
    let scaled = (value * 1_000.0).round();
    (scaled.is_finite() && scaled >= 0.0 && scaled <= i64::MAX as f64).then_some(scaled as i64)
}

fn to_unsigned_millipoints(value: f64) -> Option<u64> {
    let scaled = (value * 1_000.0).round();
    (scaled.is_finite() && scaled > 0.0 && scaled <= u64::MAX as f64).then_some(scaled as u64)
}

fn rect_contains_point(rect: CGRect, point: CGPoint) -> bool {
    point.x >= rect.origin.x
        && point.y >= rect.origin.y
        && point.x < rect.origin.x + rect.size.width
        && point.y < rect.origin.y + rect.size.height
}

pub(super) fn capture_window_backdrop_color(
    window: &tauri::WebviewWindow,
) -> Option<BackdropColor> {
    if !preflight_screen_capture_access() {
        return None;
    }

    let native_window = window.ns_window().ok()?;
    let window_id = unsafe { native_window_number(native_window) }?;
    let bounds = window_bounds(window_id)?;
    let rect = backdrop_rect(bounds)?;
    let image = unsafe {
        CGWindowListCreateImage(
            rect,
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_BELOW_WINDOW,
            window_id,
            K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING,
        )
    };
    let image = ScopedCfRef::new(image.cast())?;
    unsafe { representative_color_from_image(image.as_cg_image()) }
}

fn window_bounds(window_id: u32) -> Option<CGRect> {
    let list =
        unsafe { CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW, window_id) };
    let list = ScopedCfRef::new(list.cast())?;
    let count = unsafe { CFArrayGetCount(list.as_cf_array()) };
    for index in 0..count {
        let dictionary = unsafe { CFArrayGetValueAtIndex(list.as_cf_array(), index) };
        let Some(info) = (unsafe { window_info_from_dictionary(dictionary.cast()) }) else {
            continue;
        };
        if info.window_id == window_id {
            return Some(info.bounds);
        }
    }
    None
}

unsafe fn native_window_number(native_window: *mut std::ffi::c_void) -> Option<u32> {
    if native_window.is_null() {
        return None;
    }
    let selector = sel_registerName(c"windowNumber".as_ptr());
    if selector.is_null() {
        return None;
    }
    let number = objc_msg_send_window_number(native_window, selector);
    u32::try_from(number).ok().filter(|number| *number != 0)
}

fn backdrop_rect(window_bounds: CGRect) -> Option<CGRect> {
    let width = window_bounds.size.width;
    let height = window_bounds.size.height;
    if !window_bounds.origin.x.is_finite()
        || !window_bounds.origin.y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return None;
    }
    let sample_size = BACKDROP_SAMPLE_POINT_SIZE.min(width).min(height);
    Some(CGRect {
        origin: CGPoint {
            x: window_bounds.origin.x + (width - sample_size) / 2.0,
            y: window_bounds.origin.y + (height - sample_size) / 2.0,
        },
        size: CGSize {
            width: sample_size,
            height: sample_size,
        },
    })
}

unsafe fn representative_color_from_image(image: CGImageRef) -> Option<BackdropColor> {
    let width = CGImageGetWidth(image);
    let height = CGImageGetHeight(image);
    if width == 0
        || height == 0
        || CGImageGetBitsPerPixel(image) != 32
        || !is_supported_bgra_bitmap_info(CGImageGetBitmapInfo(image))
    {
        return None;
    }
    let provider = CGImageGetDataProvider(image);
    if provider.is_null() {
        return None;
    }
    let data = ScopedCfRef::new(CGDataProviderCopyData(provider).cast())?;
    let source_len = usize::try_from(CFDataGetLength(data.as_cf_data())).ok()?;
    let bytes_per_row = CGImageGetBytesPerRow(image);
    let required_len = bytes_per_row.checked_mul(height)?;
    if source_len < required_len || bytes_per_row < width.checked_mul(RGBA_BYTES_PER_PIXEL)? {
        return None;
    }
    let source_ptr = CFDataGetBytePtr(data.as_cf_data());
    if source_ptr.is_null() {
        return None;
    }
    let source = std::slice::from_raw_parts(source_ptr, source_len);
    let pixel_count = width.checked_mul(height)?;
    let stride = (pixel_count as f64 / MAX_BACKDROP_PIXELS as f64)
        .sqrt()
        .ceil()
        .max(1.0) as usize;
    let mut red = Vec::new();
    let mut green = Vec::new();
    let mut blue = Vec::new();

    for row in (0..height).step_by(stride) {
        for column in (0..width).step_by(stride) {
            let offset = row * bytes_per_row + column * RGBA_BYTES_PER_PIXEL;
            let pixel = &source[offset..offset + RGBA_BYTES_PER_PIXEL];
            if pixel[3] < 128 {
                continue;
            }
            red.push(unpremultiply(pixel[2], pixel[3]));
            green.push(unpremultiply(pixel[1], pixel[3]));
            blue.push(unpremultiply(pixel[0], pixel[3]));
        }
    }

    Some(BackdropColor {
        red: median(&mut red)?,
        green: median(&mut green)?,
        blue: median(&mut blue)?,
    })
}

fn unpremultiply(channel: u8, alpha: u8) -> u8 {
    if alpha == u8::MAX {
        return channel;
    }
    ((u32::from(channel) * u32::from(u8::MAX) + u32::from(alpha) / 2) / u32::from(alpha))
        .min(u32::from(u8::MAX)) as u8
}

fn median(values: &mut [u8]) -> Option<u8> {
    values.sort_unstable();
    values.get(values.len() / 2).copied()
}

pub(super) fn request_screen_capture_access() -> bool {
    preflight_screen_capture_access() || unsafe { CGRequestScreenCaptureAccess() }
}

fn preflight_screen_capture_access() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

fn capture_rect(region: &PhysicalRegion, scale_factor: f64) -> CGRect {
    CGRect {
        origin: CGPoint {
            x: f64::from(region.x) / scale_factor,
            y: f64::from(region.y) / scale_factor,
        },
        size: CGSize {
            width: f64::from(region.width) / scale_factor,
            height: f64::from(region.height) / scale_factor,
        },
    }
}

unsafe fn rgba_bytes_from_image(
    region: &PhysicalRegion,
    image: CGImageRef,
) -> Result<Vec<u8>, CaptureError> {
    let width = CGImageGetWidth(image);
    let height = CGImageGetHeight(image);
    let expected_len = byte_len(region)?;

    if width != region.width as usize || height != region.height as usize {
        return Err(capture_unavailable(
            region,
            "macOS returned a capture image with unexpected dimensions.",
        ));
    }

    if CGImageGetBitsPerPixel(image) != 32
        || !is_supported_bgra_bitmap_info(CGImageGetBitmapInfo(image))
    {
        return Err(unsupported_format(region));
    }

    let provider = CGImageGetDataProvider(image);
    if provider.is_null() {
        return Err(capture_unavailable(
            region,
            "macOS capture image did not include a data provider.",
        ));
    }

    let data = ScopedCfRef::new(CGDataProviderCopyData(provider).cast()).ok_or_else(|| {
        capture_unavailable(
            region,
            "macOS capture image did not include readable pixel data.",
        )
    })?;

    copy_bgra_rows_to_rgba(
        CFDataGetBytePtr(data.as_cf_data()),
        CFDataGetLength(data.as_cf_data()),
        CGImageGetBytesPerRow(image),
        width,
        height,
        expected_len,
    )
    .map_err(|code| {
        capture_error(
            code,
            &region.monitor_id,
            "macOS capture pixel data was not usable.",
        )
    })
}

fn is_supported_bgra_bitmap_info(bitmap_info: u32) -> bool {
    let alpha = bitmap_info & K_CG_BITMAP_ALPHA_INFO_MASK;
    let byte_order = bitmap_info & K_CG_BITMAP_BYTE_ORDER_MASK;

    byte_order == K_CG_BITMAP_BYTE_ORDER_32_LITTLE
        && matches!(
            alpha,
            K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST
                | K_CG_IMAGE_ALPHA_FIRST
                | K_CG_IMAGE_ALPHA_NONE_SKIP_FIRST
        )
}

fn copy_bgra_rows_to_rgba(
    source: *const u8,
    source_len: isize,
    bytes_per_row: usize,
    width: usize,
    height: usize,
    expected_len: usize,
) -> Result<Vec<u8>, CaptureErrorCode> {
    if source.is_null() || source_len < 0 {
        return Err(CaptureErrorCode::CaptureUnavailable);
    }

    let source_len =
        usize::try_from(source_len).map_err(|_| CaptureErrorCode::CaptureUnavailable)?;
    let required_len = bytes_per_row
        .checked_mul(height)
        .ok_or(CaptureErrorCode::CaptureUnavailable)?;
    let row_len = width
        .checked_mul(RGBA_BYTES_PER_PIXEL)
        .ok_or(CaptureErrorCode::CaptureUnavailable)?;

    if source_len < required_len || bytes_per_row < row_len {
        return Err(CaptureErrorCode::CaptureUnavailable);
    }

    let mut rgba = Vec::with_capacity(expected_len);
    let source = unsafe { std::slice::from_raw_parts(source, source_len) };

    for row in 0..height {
        let row_start = row * bytes_per_row;
        for pixel in source[row_start..row_start + row_len].chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }

    if rgba.len() == expected_len {
        Ok(rgba)
    } else {
        Err(CaptureErrorCode::CaptureUnavailable)
    }
}

fn permission_denied(region: &PhysicalRegion) -> CaptureError {
    capture_error(
        CaptureErrorCode::PermissionDenied,
        &region.monitor_id,
        "Screen recording permission is required before real capture can run.",
    )
}

fn capture_unavailable(region: &PhysicalRegion, message: &'static str) -> CaptureError {
    capture_error(
        CaptureErrorCode::CaptureUnavailable,
        &region.monitor_id,
        message,
    )
}

fn unsupported_format(region: &PhysicalRegion) -> CaptureError {
    capture_error(
        CaptureErrorCode::UnsupportedPixelFormat,
        &region.monitor_id,
        "macOS returned a capture pixel format Pebble cannot safely convert.",
    )
}

struct ScopedCfRef {
    ptr: CFTypeRef,
}

impl ScopedCfRef {
    fn new(ptr: CFTypeRef) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    fn as_cg_image(&self) -> CGImageRef {
        self.ptr.cast()
    }

    fn as_cf_data(&self) -> CFDataRef {
        self.ptr.cast()
    }

    fn as_cf_array(&self) -> CFArrayRef {
        self.ptr.cast()
    }
}

impl Drop for ScopedCfRef {
    fn drop(&mut self) {
        unsafe { CFRelease(self.ptr) };
    }
}

#[cfg(test)]
pub(super) fn test_copy_bgra_rows_to_rgba(
    source: &[u8],
    bytes_per_row: usize,
    width: usize,
    height: usize,
    expected_len: usize,
) -> Result<Vec<u8>, CaptureErrorCode> {
    copy_bgra_rows_to_rgba(
        source.as_ptr(),
        source.len() as isize,
        bytes_per_row,
        width,
        height,
        expected_len,
    )
}

#[cfg(test)]
pub(super) fn test_capture_rect(
    region: &PhysicalRegion,
    scale_factor: f64,
) -> (f64, f64, f64, f64) {
    let rect = capture_rect(region, scale_factor);

    (
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}

#[cfg(test)]
pub(super) fn test_is_supported_bgra_bitmap_info(bitmap_info: u32) -> bool {
    is_supported_bgra_bitmap_info(bitmap_info)
}

#[cfg(test)]
pub(super) fn test_window_list_options() -> (u32, u32) {
    (
        K_CG_WINDOW_LIST_OPTION_ON_SCREEN_BELOW_WINDOW,
        K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW,
    )
}

#[cfg(test)]
pub(super) fn test_backdrop_rect(x: f64, y: f64, width: f64, height: f64) -> (f64, f64, f64, f64) {
    let rect = backdrop_rect(CGRect {
        origin: CGPoint { x, y },
        size: CGSize { width, height },
    })
    .expect("backdrop rect");
    (
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}
