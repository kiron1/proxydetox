#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn duk_create_heap_default() -> *mut duk_context {
    duk_create_heap(None, None, None, std::ptr::null_mut(), None)
}

pub unsafe fn duk_eval_string(ctx: *mut duk_context, src: &CStr) {
    duk_eval_raw(
        ctx,
        src.as_ptr(),
        0,
        0 /*args*/ | DUK_COMPILE_EVAL | DUK_COMPILE_NOSOURCE | DUK_COMPILE_STRLEN | DUK_COMPILE_NOFILENAME,
    );
}

pub unsafe fn duk_push_external_buffer(ctx: *mut duk_context) {
    duk_push_buffer_raw(ctx, 0, DUK_BUF_FLAG_DYNAMIC | DUK_BUF_FLAG_EXTERNAL);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    #[test]
    fn it_works() {
        unsafe {
            let ctx = duk_create_heap_default();
            let prog = CString::new("1+2").expect("CString");
            duk_eval_string(ctx, &prog);
            assert_eq!(3, duk_get_int(ctx, -1));
            duk_destroy_heap(ctx);
        }
    }
}
