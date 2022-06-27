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

// Outlives: suggestion from /u/YatoRust
// https://www.reddit.com/r/rust/comments/tjzy97/reborrow_emulating_reborrowing_for_user_types/i1nco4i/

/// Immutable reborrowing.
pub trait Reborrow<'b, Outlives = &'b Self> {
    type Target;
    fn rb(&'b self) -> Self::Target;
}

/// Mutable reborrowing.
pub trait ReborrowMut<'b, Outlives = &'b Self>
where
    Self: 'b,
{
    type Target;

    fn rb_mut(&'b mut self) -> Self::Target;
}

impl<'b, 'a, T> Reborrow<'b> for &'a T
where
    T: ?Sized,
{
    type Target = &'b T;

    fn rb(&'b self) -> Self::Target {
        *self
    }
}

impl<'b, 'a, T> ReborrowMut<'b> for &'a T
where
    T: ?Sized,
{
    type Target = &'b T;

    fn rb_mut(&'b mut self) -> Self::Target {
        *self
    }
}

impl<'b, 'a, T> Reborrow<'b> for &'a mut T
where
    T: ?Sized,
{
    type Target = &'b T;

    fn rb(&'b self) -> Self::Target {
        *self
    }
}

impl<'b, 'a, T> ReborrowMut<'b> for &'a mut T
where
    T: ?Sized,
{
    type Target = &'b mut T;

    fn rb_mut(&'b mut self) -> Self::Target {
        *self
    }
}

impl<'b, T> Reborrow<'b> for Option<T>
where
    T: Reborrow<'b>,
{
    type Target = Option<T::Target>;

    fn rb(&'b self) -> Self::Target {
        match self {
            &None => None,
            &Some(ref x) => Some(x.rb()),
        }
    }
}

impl<'b, T> ReborrowMut<'b> for Option<T>
where
    T: ReborrowMut<'b>,
{
    type Target = Option<T::Target>;

    fn rb_mut(&'b mut self) -> Self::Target {
        match self {
            &mut None => None,
            &mut Some(ref mut x) => Some(x.rb_mut()),
        }
    }
}

impl<'b, T, E> Reborrow<'b> for Result<T, E>
where
    T: Reborrow<'b>,
    E: Reborrow<'b>,
{
    type Target = Result<T::Target, E::Target>;

    fn rb(&'b self) -> Self::Target {
        match self {
            &Ok(ref v) => Ok(v.rb()),
            &Err(ref e) => Err(e.rb()),
        }
    }
}

impl<'b, T, E> ReborrowMut<'b> for Result<T, E>
where
    T: ReborrowMut<'b>,
    E: ReborrowMut<'b>,
{
    type Target = Result<T::Target, E::Target>;

    fn rb_mut(&'b mut self) -> Self::Target {
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

    #[test]
    fn custom_view_type() {
        struct MyViewType<'a> {
            r: &'a mut i32,
        }

        impl<'b, 'a> ReborrowMut<'b> for MyViewType<'a>
        where
            'a: 'b,
        {
            type Target = MyViewType<'b>;

            fn rb_mut(&'b mut self) -> Self::Target {
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
}
