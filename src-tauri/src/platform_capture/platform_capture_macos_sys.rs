use std::ffi::c_void;

pub(super) type CGFloat = f64;
pub(super) type CGImageRef = *const c_void;
pub(super) type CGDataProviderRef = *const c_void;
pub(super) type CFDataRef = *const c_void;
pub(super) type CFTypeRef = *const c_void;

pub(super) const K_CG_NULL_WINDOW_ID: u32 = 0;
pub(super) const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;
pub(super) const K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING: u32 = 1 << 0;
pub(super) const K_CG_BITMAP_ALPHA_INFO_MASK: u32 = 0x1f;
pub(super) const K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST: u32 = 2;
pub(super) const K_CG_IMAGE_ALPHA_FIRST: u32 = 4;
pub(super) const K_CG_IMAGE_ALPHA_NONE_SKIP_FIRST: u32 = 6;
pub(super) const K_CG_BITMAP_BYTE_ORDER_MASK: u32 = 0x7000;
pub(super) const K_CG_BITMAP_BYTE_ORDER_32_LITTLE: u32 = 2 << 12;

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGPoint {
    pub(super) x: CGFloat,
    pub(super) y: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGSize {
    pub(super) width: CGFloat,
    pub(super) height: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGRect {
    pub(super) origin: CGPoint,
    pub(super) size: CGSize,
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub(super) fn CGPreflightScreenCaptureAccess() -> bool;
    pub(super) fn CGWindowListCreateImage(
        screen_bounds: CGRect,
        list_option: u32,
        window_id: u32,
        image_option: u32,
    ) -> CGImageRef;
    pub(super) fn CGImageGetWidth(image: CGImageRef) -> usize;
    pub(super) fn CGImageGetHeight(image: CGImageRef) -> usize;
    pub(super) fn CGImageGetBitsPerPixel(image: CGImageRef) -> usize;
    pub(super) fn CGImageGetBitmapInfo(image: CGImageRef) -> u32;
    pub(super) fn CGImageGetBytesPerRow(image: CGImageRef) -> usize;
    pub(super) fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
    pub(super) fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub(super) fn CFDataGetBytePtr(the_data: CFDataRef) -> *const u8;
    pub(super) fn CFDataGetLength(the_data: CFDataRef) -> isize;
    pub(super) fn CFRelease(cf: CFTypeRef);
}
