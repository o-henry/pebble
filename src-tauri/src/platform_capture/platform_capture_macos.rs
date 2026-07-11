use crate::{
    capture_backend::{
        byte_len, capture_error, cropped_frame, CaptureError, CaptureErrorCode, CaptureResult,
        RGBA_BYTES_PER_PIXEL,
    },
    region_selection_types::PhysicalRegion,
};
use platform_capture_macos_sys::*;

#[path = "platform_capture_macos_sys.rs"]
mod platform_capture_macos_sys;

pub(super) fn capture_region(region: &PhysicalRegion, scale_factor: f64) -> CaptureResult {
    if !preflight_screen_capture_access() {
        return Err(permission_denied(region));
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
