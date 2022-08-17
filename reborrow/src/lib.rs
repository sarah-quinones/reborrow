//! Emulate reborrowing for user types.
//!
//! Given a `&'a` \[mutable\] reference of a `&'b` view over some owned object,
//! reborrowing it means getting an active `&'a` view over the owned object,
//! which renders the original reference inactive until it's dropped, at which point
//! the original reference becomes active again.
//!
//! # Examples:
//! This fails to compile since we can't use a non-`Copy` value after it's moved.
//! ```compile_fail
//! fn takes_mut_option(o: Option<&mut i32>) {}
//!
//! let mut x = 0;
//! let o = Some(&mut x);
//! takes_mut_option(o); // `o` is moved here,
//! takes_mut_option(o); // so it can't be used here.
//! ```
//!
//! This can be worked around by unwrapping the option, reborrowing it, and then wrapping it again.
//! ```
//! fn takes_mut_option(o: Option<&mut i32>) {}
//!
//! let mut x = 0;
//! let mut o = Some(&mut x);
//! takes_mut_option(o.as_mut().map(|r| &mut **r)); // "Reborrowing" the `Option`
//! takes_mut_option(o.as_mut().map(|r| &mut **r)); // allows us to use it later on.
//! drop(o); // can still be used here
//! ```
//!
//! Using this crate, this can be shortened to
//! ```
//! use reborrow::ReborrowMut;
//!
//! fn takes_mut_option(o: Option<&mut i32>) {}
//!
//! let mut x = 0;
//! let mut o = Some(&mut x);
//! takes_mut_option(o.rb_mut()); // "Reborrowing" the `Option`
//! takes_mut_option(o.rb_mut()); // allows us to use it later on.
//! drop(o); // can still be used here
//! ```
//!
//! The derive macro can be used with structs or tuple structs, on the mutable variant and
//! generates the trait definitions for [`Reborrow`], [`ReborrowMut`], and [`IntoConst`].
//!
//! ```
//! use reborrow::Reborrow;
//!
//! mod shared {
//!     use reborrow::ReborrowCopy;
//!
//!     #[derive(ReborrowCopy)]
//!     pub struct I32Ref<'a, 'b> {
//!         pub i: i32,
//!         pub j: &'a i32,
//!         pub k: &'b i32,
//!     }
//!
//!     #[derive(ReborrowCopy)]
//!     pub struct I32TupleRef<'a, 'b>(pub i32, pub &'a i32, pub &'b i32);
//! }
//!
//! #[derive(Reborrow)]
//! #[Const(shared::I32Ref)]
//! struct I32RefMut<'a, 'b> {
//!     i: i32,
//!     #[reborrow]
//!     j: &'a mut i32,
//!     #[reborrow]
//!     k: &'b mut i32,
//! }
//!
//! #[derive(Reborrow)]
//! #[Const(shared::I32TupleRef)]
//! pub struct I32TupleRefMut<'a, 'b>(
//!     i32,
//!     #[reborrow] &'a mut i32,
//!     #[reborrow] &'b mut i32,
//! );
//! ```

// _Outlives: suggestion from /u/YatoRust
// https://www.reddit.com/r/rust/comments/tjzy97/reborrow_emulating_reborrowing_for_user_types/i1nco4i/

mod seal {
    pub trait Seal<T: ?Sized> {}
    impl<T: ?Sized> Seal<T> for T {}
}

use seal::Seal;

#[cfg(feature = "derive")]
pub use reborrow_derive::{Reborrow, ReborrowCopy};

/// Immutable reborrowing.
pub trait Reborrow<'short, _Outlives: Seal<&'short Self> = &'short Self>
where
    Self: 'short,
{
    type Target;
    #[must_use]
    fn rb(&'short self) -> Self::Target;
}

/// Mutable reborrowing.
pub trait ReborrowMut<'short, _Outlives: Seal<&'short Self> = &'short Self>
where
    Self: 'short,
{
    type Target;
    #[must_use]
    fn rb_mut(&'short mut self) -> Self::Target;
}

/// Consume a mutable reference to produce an immutable one.
pub trait IntoConst {
    type Target;
    #[must_use]
    fn into_const(self) -> Self::Target;
}

/// This trait is similar to [`std::convert::AsRef`], but works with arbitrary reference proxy
/// types, instead of being limited to Rust references.
pub trait AsPseudoRef<'short, Target, _Outlives: Seal<&'short Self> = &'short Self>
where
    Self: 'short,
{
    #[must_use]
    fn as_pseudo_ref(&'short self) -> Target;
}

/// This trait is similar to [`std::convert::AsMut`], but works with arbitrary reference proxy
/// types, instead of being limited to Rust references.
pub trait AsPseudoMut<'short, Target, _Outlives: Seal<&'short Self> = &'short Self>
where
    Self: 'short,
{
    #[must_use]
    fn as_pseudo_mut(&'short mut self) -> Target;
}

impl<'short, T: ?Sized + AsRef<Target>, Target: ?Sized> AsPseudoRef<'short, &'short Target> for T {
    #[inline]
    fn as_pseudo_ref(&'short self) -> &'short Target {
        self.as_ref()
    }
}

impl<'short, T: ?Sized + AsMut<Target>, Target: ?Sized> AsPseudoMut<'short, &'short mut Target>
    for T
{
    #[inline]
    fn as_pseudo_mut(&'short mut self) -> &'short mut Target {
        self.as_mut()
    }
}

impl<'short, 'a, T> Reborrow<'short> for &'a T
where
    T: ?Sized,
{
    type Target = &'short T;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        *self
    }
}

impl<'short, 'a, T> ReborrowMut<'short> for &'a T
where
    T: ?Sized,
{
    type Target = &'short T;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        *self
    }
}

impl<'a, T> IntoConst for &'a T
where
    T: ?Sized,
{
    type Target = &'a T;

    #[inline]
    fn into_const(self) -> Self::Target {
        self
    }
}

impl<'short, 'a, T> Reborrow<'short> for &'a mut T
where
    T: ?Sized,
{
    type Target = &'short T;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        *self
    }
}

impl<'short, 'a, T> ReborrowMut<'short> for &'a mut T
where
    T: ?Sized,
{
    type Target = &'short mut T;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        *self
    }
}

impl<'a, T> IntoConst for &'a mut T
where
    T: ?Sized,
{
    type Target = &'a T;

    #[inline]
    fn into_const(self) -> Self::Target {
        self
    }
}

impl<'short, T> Reborrow<'short> for Option<T>
where
    T: Reborrow<'short>,
{
    type Target = Option<T::Target>;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        match self {
            &None => None,
            &Some(ref x) => Some(x.rb()),
        }
    }
}

impl<'short, T> ReborrowMut<'short> for Option<T>
where
    T: ReborrowMut<'short>,
{
    type Target = Option<T::Target>;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        match self {
            &mut None => None,
            &mut Some(ref mut x) => Some(x.rb_mut()),
        }
    }
}

impl<T> IntoConst for Option<T>
where
    T: IntoConst,
{
    type Target = Option<T::Target>;

    #[inline]
    fn into_const(self) -> Self::Target {
        match self {
            None => None,
            Some(x) => Some(x.into_const()),
        }
    }
}

impl<'short, T, E> Reborrow<'short> for Result<T, E>
where
    T: Reborrow<'short>,
    E: Reborrow<'short>,
{
    type Target = Result<T::Target, E::Target>;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        match self {
            &Ok(ref v) => Ok(v.rb()),
            &Err(ref e) => Err(e.rb()),
        }
    }
}

impl<'short, T, E> ReborrowMut<'short> for Result<T, E>
where
    T: ReborrowMut<'short>,
    E: ReborrowMut<'short>,
{
    type Target = Result<T::Target, E::Target>;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        match self {
            &mut Ok(ref mut v) => Ok(v.rb_mut()),
            &mut Err(ref mut e) => Err(e.rb_mut()),
        }
    }
}

impl<T, E> IntoConst for Result<T, E>
where
    T: IntoConst,
    E: IntoConst,
{
    type Target = Result<T::Target, E::Target>;

    #[inline]
    fn into_const(self) -> Self::Target {
        match self {
            Ok(v) => Ok(v.into_const()),
            Err(e) => Err(e.into_const()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option() {
        let mut a = 0;
        let mut opt = Some(&mut a);
        let opt_mut = &mut opt;
        let _ = opt_mut.rb_mut();
    }

    #[test]
    fn result() {
        let mut a = 0;
        let mut opt = Ok::<&mut i32, &()>(&mut a);
        let opt_mut = &mut opt;
        let _ = opt_mut.rb_mut();
    }

    #[test]
    fn custom_view_type() {
        struct MyViewType<'a> {
            r: &'a mut i32,
        }

        impl<'short, 'a> ReborrowMut<'short> for MyViewType<'a>
        where
            'a: 'short,
        {
            type Target = MyViewType<'short>;

            fn rb_mut(&'short mut self) -> Self::Target {
                MyViewType { r: self.r }
            }
        }

        fn takes_mut_option(_o: Option<MyViewType>) {}

        let mut x = 0;
        let mut o = Some(MyViewType { r: &mut x });
        takes_mut_option(o.rb_mut());
        takes_mut_option(o.rb_mut());
        drop(o);
    }

    #[test]
    fn as_ref() {
        let v = vec![()];
        let _r: &[()] = v.as_pseudo_ref();
    }
}
