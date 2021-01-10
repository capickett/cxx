use crate::private::UniquePtrTarget;
use crate::Exception;
use crate::UniquePtr;
use alloc::boxed::Box;
use core::ffi::c_void;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

/// Binding to C++ `cxx::Future<T>`.
#[repr(C, packed)]
pub struct CxxFuture<T>
where
    T: FutureResult,
{
    repr: [*mut c_void; 2],
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

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = self.get_unchecked_mut() as *mut Self as *mut c_void;

            let result = UniquePtr::from_raw(T::__move_result_unchecked(this) as *mut T);
            if !result.is_null() {
                // Already done, return right away
                return Poll::Ready(Ok(result));
            }

            // Provide context to awaken later
            unsafe extern "C" fn wake(waker_ptr: *mut Waker) {
                let waker = Box::from_raw(waker_ptr);
                waker.wake();
            }

            let waker = Box::new(cx.waker().clone());
            T::__future_set_waker(this, Box::into_raw(waker), wake);
            Poll::Pending
        }
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
#[doc(hidden)]
pub unsafe trait FutureResult: Sized + UniquePtrTarget {
    const __NAME: &'static dyn Display;

    fn __future_ready(this: *const c_void) -> bool;

    unsafe fn __future_set_waker(this: *mut c_void, waker: *mut Waker, wake: unsafe extern fn(*mut Waker));

    unsafe fn __move_result_unchecked(this: *mut c_void) -> *mut c_void;

    unsafe fn __drop(this: *mut c_void);
}

// macro_rules! impl_future_result {
//     ($segment:expr, $name:expr, $ty:ty) => {
//         const_assert_eq!(1, mem::align_of::<CxxFuture<$ty>>());

//         unsafe impl FutureResult for $ty {
//             const __NAME: &'static dyn Display = &$name;

//             fn __future_ready(this: *const c_void) -> bool {
//                 extern "C" {
//                     attr! {
//                         #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$ready")]
//                         fn __future_ready(_: *const c_void) -> bool;
//                     }
//                 }
//                 unsafe { __future_ready(this) }
//             }
//             unsafe fn __future_set_waker(this: *mut c_void, waker: *mut Waker, wake: fn(*mut Waker)) {
//                 extern "C" {}
//             }
//             unsafe fn __move_result_unchecked(this: *mut c_void) -> *mut c_void {
//                 extern "C" {
//                     attr! {
//                         #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$move_result")]
//                         fn __move_result_unchecked(_: *mut c_void) -> *mut c_void;
//                     }
//                 }
//                 __move_result_unchecked(this)
//             }
//             unsafe fn __drop(this: *mut c_void) {
//                 extern "C" {
//                     attr! {
//                         #[link_name = concat!("cxxbridge1$cxx$Future$", $segment, "$drop")]
//                         fn __drop(_: *mut c_void);
//                     }
//                 }
//                 __drop(this)
//             }
//         }
//     };
// }

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

// impl_future_result!("string", "CxxString", CxxString);
