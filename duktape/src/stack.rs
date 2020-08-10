use duktape_sys::{
    duk_call, duk_context, duk_dup, duk_eval_string, duk_get_boolean, duk_get_buffer_data,
    duk_get_global_string, duk_get_number, duk_get_pointer, duk_get_prop, duk_get_string,
    duk_get_top, duk_get_type, duk_is_undefined, duk_pop, duk_push_boolean, duk_push_c_function,
    duk_push_global_stash, duk_push_null, duk_push_pointer, duk_push_string, duk_put_global_string,
    duk_put_prop, duk_require_stack, duk_ret_t, duk_swap_top, DUK_TYPE_BOOLEAN, DUK_TYPE_BUFFER,
    DUK_TYPE_NONE, DUK_TYPE_NULL, DUK_TYPE_NUMBER, DUK_TYPE_POINTER, DUK_TYPE_STRING,
    DUK_TYPE_UNDEFINED,
};
use std::ffi::{c_void, CStr, CString};

use crate::error::TypeError;
use std::result::Result;

pub trait Stack {
    unsafe fn ptr(&mut self) -> *mut duk_context;

    fn pop_string(&mut self) -> Result<crate::Value, TypeError> {
        let result = self.get_string(-1)?;

        unsafe {
            duk_pop(self.ptr());
        }
        Ok(crate::Value::String(result))
    }

    fn pop(&mut self) -> Result<crate::Value, TypeError> {
        let typeid: u32 = unsafe { duk_get_type(self.ptr(), -1) as u32 };
        match typeid {
            DUK_TYPE_NONE => Err(TypeError::NoneType),
            DUK_TYPE_UNDEFINED => {
                let r = Ok(crate::Value::Undefined);
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_NULL => {
                let r = Ok(crate::Value::Null);
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_BOOLEAN => {
                let r = Ok(crate::Value::from(
                    unsafe { duk_get_boolean(self.ptr(), -1) } != 0,
                ));
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_NUMBER => {
                let r = Ok(crate::Value::from(unsafe {
                    duk_get_number(self.ptr(), -1)
                }));
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_STRING => self.pop_string(),
            _ => Err(TypeError::NoneType),
        }
    }

    fn pop_ptr<T>(&mut self) -> Result<*mut T, TypeError> {
        let typeid: u32 = unsafe { duk_get_type(self.ptr(), -1) as u32 };
        match typeid {
            DUK_TYPE_BUFFER => unsafe {
                Ok(duk_get_buffer_data(self.ptr(), -1, std::ptr::null_mut()) as *mut T)
            },
            DUK_TYPE_POINTER => unsafe {
                let ptr = duk_get_pointer(self.ptr(), -1) as *mut T;
                Ok(ptr)
            },
            _ => Err(TypeError::NoneType),
        }
    }

    fn eval(&mut self, src: &str) -> Result<crate::Value, TypeError> {
        unsafe {
            duk_eval_string(
                self.ptr(),
                &CString::new(src).map_err(|_| TypeError::BadString)?,
            );
        }
        self.pop()
    }

    fn put_global_pointer<T: Sized>(&mut self, name: &str, ptr: *mut T) -> Result<(), TypeError> {
        let name = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_pointer(self.ptr(), ptr as *mut c_void);
            duk_put_global_string(self.ptr(), name.as_ptr());
        }
        Ok(())
    }

    fn push_global_stash(&mut self) {
        unsafe {
            duk_push_global_stash(self.ptr());
        }
    }

    fn dup(&mut self, from_idx: i32) {
        unsafe {
            duk_dup(self.ptr(), from_idx);
        }
    }

    fn drop(&mut self) {
        unsafe { duk_pop(self.ptr()) }
    }

    fn top(&mut self) -> i32 {
        unsafe { duk_get_top(self.ptr()) as i32 }
    }

    fn swap_top(&mut self, idx: i32) {
        unsafe {
            duk_swap_top(self.ptr(), idx);
        }
    }

    fn get_prop(&mut self, obj_idx: i32) {
        unsafe {
            duk_get_prop(self.ptr(), obj_idx);
        }
    }

    fn put_prop(&mut self, obj_idx: i32) {
        unsafe {
            duk_put_prop(self.ptr(), obj_idx);
        }
    }

    fn is_undefined(&mut self, idx: i32) -> bool {
        unsafe { duk_is_undefined(self.ptr(), idx) != 0u32 }
    }

    fn get_global_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        let exists = unsafe { duk_get_global_string(self.ptr(), cstr.as_ptr()) } != 0;
        if exists {
            Ok(())
        } else {
            Err(TypeError::NoneType)
        }
    }

    fn get_string(&mut self, idx: i32) -> Result<String, TypeError> {
        let cstr = unsafe { duk_get_string(self.ptr(), idx) };
        let cstr = unsafe { CStr::from_ptr(cstr) };
        let cstr = cstr.to_owned();
        cstr.into_string().map_err(|_| TypeError::BadString)
    }

    fn require_stack(&mut self, sz: i32) {
        unsafe {
            duk_require_stack(self.ptr(), sz);
        }
    }

    fn push_null(&mut self) {
        unsafe {
            duk_push_null(self.ptr());
        }
    }

    fn push_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_string(self.ptr(), cstr.as_ptr());
        }
        Ok(())
    }

    fn push_bool(&mut self, value: bool) {
        let value = if value { 1 } else { 0 };
        unsafe {
            duk_push_boolean(self.ptr(), value);
        }
    }
    fn push_c_function(
        &mut self,
        name: &str,
        func: unsafe extern "C" fn(ctx: *mut duk_context) -> duk_ret_t,
        nargs: i32,
    ) -> Result<(), TypeError> {
        let name = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_c_function(
                self.ptr(),
                /* func pointer */ Some(func),
                /* nargs: */ nargs,
            );
        }
        unsafe {
            duk_put_global_string(self.ptr(), name.as_ptr());
        }
        Ok(())
    }

    fn call(&mut self, nargs: i32) {
        unsafe {
            duk_call(self.ptr(), nargs);
        }
    }
}
