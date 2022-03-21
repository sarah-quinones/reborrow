#![feature(generic_associated_types)]
#![allow(deprecated_where_clause_location)]

//! Emulate reborrowing for user types.
//!
//! Given a `&'a` [mutable] reference of a `&'b` view over some owned object,
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

/// Immutable reborrowing.
pub trait Reborrow {
    type Target<'b>
    where
        Self: 'b;

    fn rb<'b>(&'b self) -> Self::Target<'b>;
}

/// Mutable reborrowing.
pub trait ReborrowMut {
    type Target<'b>
    where
        Self: 'b;

    fn rb_mut<'b>(&'b mut self) -> Self::Target<'b>;
}

impl<'a, T> Reborrow for &'a T
where
    T: ?Sized,
{
    type Target<'b>
    where
        'a: 'b,
    = &'b T;

    fn rb<'b>(&'b self) -> Self::Target<'b> {
        *self
    }
}

impl<'a, T> ReborrowMut for &'a T
where
    T: ?Sized,
{
    type Target<'b>
    where
        'a: 'b,
    = &'b T;

    fn rb_mut<'b>(&'b mut self) -> Self::Target<'b> {
        *self
    }
}

impl<'a, T> Reborrow for &'a mut T
where
    T: ?Sized,
{
    type Target<'b>
    where
        'a: 'b,
    = &'b T;

    fn rb<'b>(&'b self) -> Self::Target<'b> {
        *self
    }
}

impl<'a, T> ReborrowMut for &'a mut T
where
    T: ?Sized,
{
    type Target<'b>
    where
        'a: 'b,
    = &'b mut T;

    fn rb_mut<'b>(&'b mut self) -> Self::Target<'b> {
        *self
    }
}

impl<T> Reborrow for Option<T>
where
    T: Reborrow,
{
    type Target<'b>
    where
        Self: 'b,
    = Option<T::Target<'b>>;

    fn rb<'b>(&'b self) -> Self::Target<'b> {
        match self {
            &None => None,
            &Some(ref x) => Some(x.rb()),
        }
    }
}

impl<T> ReborrowMut for Option<T>
where
    T: ReborrowMut,
{
    type Target<'b>
    where
        Self: 'b,
    = Option<T::Target<'b>>;

    fn rb_mut<'b>(&'b mut self) -> Self::Target<'b> {
        match self {
            &mut None => None,
            &mut Some(ref mut x) => Some(x.rb_mut()),
        }
    }
}

impl<T, E> Reborrow for Result<T, E>
where
    T: Reborrow,
    E: Reborrow,
{
    type Target<'b>
    where
        Self: 'b,
    = Result<T::Target<'b>, E::Target<'b>>;

    fn rb<'b>(&'b self) -> Self::Target<'b> {
        match self {
            &Ok(ref v) => Ok(v.rb()),
            &Err(ref e) => Err(e.rb()),
        }
    }
}

impl<T, E> ReborrowMut for Result<T, E>
where
    T: ReborrowMut,
    E: ReborrowMut,
{
    type Target<'b>
    where
        Self: 'b,
    = Result<T::Target<'b>, E::Target<'b>>;

    fn rb_mut<'b>(&'b mut self) -> Self::Target<'b> {
        match self {
            &mut Ok(ref mut v) => Ok(v.rb_mut()),
            &mut Err(ref mut e) => Err(e.rb_mut()),
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
}
