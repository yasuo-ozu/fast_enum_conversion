#![doc = include_str!("README.md")]

#[doc(hidden)]
pub use addr_of_enum;
use core::mem::Discriminant;

/// This macro implements [`ConvertTo`] trait for the enum, with given types as its targets.
/// Both source and destination should be annotated.
///
/// ```
/// # use fast_enum_conversion::{convert_to, ConvertTo};
/// #[convert_to(Dest1<'a>, Dest2<'a>)]
/// #[repr(u8)]
/// #[derive(PartialEq, Eq, Debug)]
/// enum Src<'a> {
///     E1(usize, &'a str, bool) = 1,
/// }
/// #[convert_to]
/// #[repr(u8)]
/// #[derive(PartialEq, Eq, Debug)]
/// enum Dest1<'a> {
///     E1(usize, &'a str, bool) = 1,
///     E2,
/// }
/// #[convert_to]
/// #[repr(u8)]
/// #[derive(PartialEq, Eq, Debug)]
/// enum Dest2<'a> {
///     E1(usize, &'a str, bool) = 1,
///     E3,
/// }
/// assert_eq!(
///     &<Src<'static> as ConvertTo<Dest1<'static>>>::convert_to(Src::E1(123, "hello", true)),
///     &Dest1::E1(123, "hello", true)
/// );
/// ```
///
/// You can specify the crate path using `@` syntax, like `#[convert_to(@
/// my_crate::fast_enum_conversion)]`.
pub use fast_enum_conversion_macro::convert_to;

#[doc(hidden)]
pub mod _tstr {
    macro_rules! chars {
        () => {};
        ($id:ident $($rem:tt)*) => {
            #[allow(non_camel_case_types)]
            pub struct $id(::core::convert::Infallible);
            chars!($($rem)*);
        };
    }
    chars! {
        _A _B _C _D _E _F _G _H _I _J _K _L _M _N _O _P _Q _R _S _T _U
        _V _W _X _Y _Z
        _a _b _c _d _e _f _g _h _i _j _k _l _m _n _o _p _q _r _s _t _u
        _v _w _x _y _z
        _0 _1 _2 _3 _4 _5 _6 _7 _8 _9
        __
    }
}

#[doc(hidden)]
/// For internal use.
///
/// This trait implies that the enum has a specific tag, discribed by `TSName`
/// type-level string.
pub unsafe trait HasVariant<TSName>: Sized {
    type Fields: Sized;
    type Offsets;
    fn discriminant() -> Discriminant<Self>;
    fn offsets() -> Self::Offsets;
}

/// It defines conversion between enums.
///
/// This trait is implemented by [`convert_to`] macro if the `Self` type is ~convertable~ to
/// `Target`. See crate level documentation for details.
pub trait ConvertTo<Target>: Sized {
    /// Convert `Self` to `Target`. It consumes the ownership of `Self` and return ownership of
    /// `Target`.
    ///
    /// If [`ConvertTo::is_zerocost()`] is `true`, the conversion is zero cost. Otherwise, it is
    /// equivalent to [`ConvertTo::convert_to_slow()`].
    fn convert_to(self) -> Target;

    #[doc(hidden)]
    fn convert_to_slow(self) -> Target;

    /// Tries to convert `Target` to `Self`. If `Src` contains no variant which is equivalent to
    /// the given `Target` variable, it fails.
    fn try_convert_from(_: Target) -> Result<Self, Target>;

    #[doc(hidden)]
    fn try_convert_from_slow(_: Target) -> Result<Self, Target>;

    /// Check that the conversion from `Self` to `Target` is zero cost. See
    /// [`ConvertTo::convert_to()`] for details.
    fn is_zerocost() -> bool;
}
