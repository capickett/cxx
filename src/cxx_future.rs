use crate::private::UniquePtrTarget;
use crate::Exception;
use crate::{CxxString, UniquePtr};
use core::ffi::c_void;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Binding to C++ `cxx::Future<T>`.
#[repr(C, packed)]
pub struct CxxFuture<T>
where
    T: FutureResult,
{
    repr: *mut c_void,
    ty: PhantomData<T>,
}

impl<T> CxxFuture<T> where T: FutureResult {
}

unsafe impl<T> Send for CxxFuture<T> where T: Send + FutureResult {}
unsafe impl<T> Sync for CxxFuture<T> where T: Sync + FutureResult {}

impl<T> Drop for CxxFuture<T>
where
    T: FutureResult,
{
    fn drop(&mut self) {
        let this = self as *mut Self as *mut c_void;
        unsafe { <T as FutureResult>::__drop(this) }
    }
}

impl<T> Future for CxxFuture<T>
where
    T: FutureResult,
{
    type Output = Result<UniquePtr<T>, Exception>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = self.get_unchecked_mut() as *mut Self as *mut c_void;

            if !T::__future_ready(this) {
                // Provide context to awaken later
                return Poll::Pending;
            }

            let ptr = T::__move_result_unchecked(this) as *mut T;
            Poll::Ready(Ok(UniquePtr::from_raw(ptr)))
        }
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
#[doc(hidden)]
pub unsafe trait FutureResult: Sized + UniquePtrTarget {
    const __NAME: &'static dyn Display;

    fn __future_ready(this: *const c_void) -> bool;

    unsafe fn __move_result_unchecked(this: *mut c_void) -> *mut c_void;

    unsafe fn __drop(this: *mut c_void);
}

macro_rules! impl_future_result {
    ($segment:expr, $name:expr, $ty:ty) => {
        const_assert_eq!(1, mem::align_of::<CxxFuture<$ty>>());

        unsafe impl FutureResult for $ty {
            const __NAME: &'static dyn Display = &$name;

            fn __future_ready(this: *const c_void) -> bool {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$ready")]
                        fn __future_ready(_: *const c_void) -> bool;
                    }
                }
                unsafe { __future_ready(this) }
            }
            unsafe fn __move_result_unchecked(this: *mut c_void) -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$move_result")]
                        fn __move_result_unchecked(_: *mut c_void) -> *mut c_void;
                    }
                }
                __move_result_unchecked(this)
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$drop")]
                        fn __drop(_: *mut c_void);
                    }
                }
                __drop(this)
            }
        }
    };
}

// macro_rules! impl_future_result_for_primitive {
//     ($ty:ident) => {
//         impl_future_result!(stringify!($ty), stringify!($ty), $ty);
//     };
// }

// impl_future_result_for_primitive!(u8);
// impl_future_result_for_primitive!(u16);
// impl_future_result_for_primitive!(u32);
// impl_future_result_for_primitive!(u64);
// impl_future_result_for_primitive!(usize);
// impl_future_result_for_primitive!(i8);
// impl_future_result_for_primitive!(i16);
// impl_future_result_for_primitive!(i32);
// impl_future_result_for_primitive!(i64);
// impl_future_result_for_primitive!(isize);
// impl_future_result_for_primitive!(f32);
// impl_future_result_for_primitive!(f64);

impl_future_result!("string", "CxxString", CxxString);
