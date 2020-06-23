use duktape_sys::{
    duk_call, duk_context, duk_destroy_heap, duk_eval_string, duk_get_boolean,
    duk_get_global_string, duk_get_lstring, duk_get_number, duk_get_type, duk_pop, duk_push_string,
    duke_create_heap_default, DUK_TYPE_BOOLEAN, DUK_TYPE_NONE, DUK_TYPE_NULL, DUK_TYPE_NUMBER,
    DUK_TYPE_STRING, DUK_TYPE_UNDEFINED,
};
use std::ffi::{CStr, CString};
use std::fmt::{Error, Formatter};
use std::ptr::null_mut;
use std::result::Result;

pub struct Context {
    ptr: *mut duk_context,
}

impl Context {
    pub fn new() -> Result<Self, CreateContextError> {
        let ptr = unsafe { duke_create_heap_default() };
        if ptr == null_mut() {
            Err(CreateContextError)
        } else {
            Ok(Context { ptr })
        }
    }

    fn pop_string(&mut self, idx: i32) -> Result<crate::Value, TypeError> {
        let mut len: libc::size_t = 0;
        let cstr = unsafe { duk_get_lstring(self.ptr, idx, &mut len) };
        let cstr: &CStr = unsafe { CStr::from_ptr(cstr) };
        let cstr: CString = cstr.to_owned();
        unsafe {
            duk_pop(self.ptr);
        }
        Ok(crate::Value::String(
            cstr.into_string().map_err(|_| TypeError::BadString)?,
        ))
    }

    pub fn pop(&mut self) -> Result<crate::Value, TypeError> {
        let typeid: u32 = unsafe { duk_get_type(self.ptr, -1) as u32 };
        match typeid {
            DUK_TYPE_NONE => Err(TypeError::NoneType),
            DUK_TYPE_UNDEFINED => {
                let r = Ok(crate::Value::Undefined);
                unsafe {
                    duk_pop(self.ptr);
                }
                r
            }
            DUK_TYPE_NULL => {
                let r = Ok(crate::Value::Null);
                unsafe {
                    duk_pop(self.ptr);
                }
                r
            }
            DUK_TYPE_BOOLEAN => {
                let r = Ok(crate::Value::from(
                    unsafe { duk_get_boolean(self.ptr, -1) } != 0,
                ));
                unsafe {
                    duk_pop(self.ptr);
                }
                r
            }
            DUK_TYPE_NUMBER => {
                let r = Ok(crate::Value::from(unsafe { duk_get_number(self.ptr, -1) }));
                unsafe {
                    duk_pop(self.ptr);
                }
                r
            }
            DUK_TYPE_STRING => self.pop_string(-1),
            _ => Err(TypeError::NoneType),
        }
    }

    pub fn eval(&mut self, src: &str) -> Result<crate::Value, TypeError> {
        unsafe {
            duk_eval_string(
                self.ptr,
                &CString::new(src).map_err(|_| TypeError::BadString)?,
            );
        }
        self.pop()
    }

    pub fn get_global_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        let exists = unsafe { duk_get_global_string(self.ptr, cstr.as_ptr()) } != 0;
        if exists {
            Ok(())
        } else {
            Err(TypeError::NoneType)
        }
    }

    pub fn push_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_string(self.ptr, cstr.as_ptr());
        }
        Ok(())
    }

    pub fn call(&mut self, nargs: i32) {
        unsafe {
            duk_call(self.ptr, nargs);
        }
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CreateContextError;

impl std::error::Error for CreateContextError {}

impl std::fmt::Display for CreateContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "create contex error")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeError {
    NoneType,
    UnknownType,
    BadString,
}

impl std::error::Error for TypeError {}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            TypeError::NoneType => write!(f, "none type error"),
            TypeError::UnknownType => write!(f, "unknown type error"),
            TypeError::BadString => write!(f, "bad string error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ctx = Context::new().unwrap();
        match ctx.eval("5+5") {
            Ok(crate::Value::Number(k)) => assert_eq!(k, 10.0f64),
            _ => assert!(false),
        }
    }
}
