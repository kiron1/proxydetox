use crate::Stack;
use duktape_sys::{duk_context, duk_create_heap, duk_destroy_heap};
use std::ffi::{c_void, CStr};
use std::ptr::null_mut;
use std::result::Result;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("Creation of duktape context failed")]
    CreationFailed,
}

unsafe extern "C" fn fatal_handler(_udata: *mut c_void, msg: *const i8) {
    // Note that 'msg' may be NULL.
    let msg = if !msg.is_null() {
        CStr::from_ptr(msg as *mut i8)
            .to_owned()
            .into_string()
            .unwrap()
    } else {
        String::from("unknown")
    };
    panic!("duk error: {}", msg);
}

pub struct Context {
    ptr: *mut duk_context,
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let ptr = unsafe { duk_create_heap(None, None, None, null_mut(), Some(fatal_handler)) };
        if ptr.is_null() {
            Err(Error::CreationFailed)
        } else {
            Ok(Context { ptr })
        }
    }
}

impl Stack for Context {
    unsafe fn ptr(&mut self) -> *mut duk_context {
        self.ptr
    }
}

unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            duk_destroy_heap(self.ptr);
        }
    }
}

pub struct ContextRef {
    ptr: *mut duk_context,
}

impl Stack for ContextRef {
    unsafe fn ptr(&mut self) -> *mut duk_context {
        self.ptr
    }
}

unsafe impl Send for ContextRef {}

impl From<*mut duk_context> for ContextRef {
    fn from(ptr: *mut duk_context) -> ContextRef {
        ContextRef { ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ctx = Context::new().unwrap();
        match ctx.eval("5+5") {
            Ok(crate::Value::Number(k)) => assert!((k - 10.0f64).abs() < 0.001f64),
            _ => panic!("eval failed"),
        }
    }
}
