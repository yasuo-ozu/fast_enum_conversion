use fast_enum_conversion::{convert_to, ConvertTo};

#[test]
fn test() {
    #[convert_to(Dest<'a>)]
    #[repr(u8)]
    #[derive(PartialEq, Eq, Debug)]
    enum Src<'a> {
        E2 { item1: u8, item2: bool } = 5,
        E1(usize, &'a str, bool) = 7,
        E3 = 1,
    }
    #[convert_to]
    #[repr(u8)]
    #[derive(PartialEq, Eq, Debug)]
    enum Dest<'a> {
        E1(usize, &'a str, bool) = 7,
        E2 { item1: u8, item2: bool } = 5,
        E3 = 1,
        E4,
    }

    assert!(<Src<'static> as ConvertTo<Dest<'static>>>::is_zerocost());
    assert_eq!(
        Src::E1(123, "hello", true).convert_to(),
        Dest::E1(123, "hello", true)
    );
    assert_eq!(
        Src::E1(usize::MAX, "hello", true).convert_to(),
        Dest::E1(usize::MAX, "hello", true)
    );
    assert_eq!(
        Src::E2 {
            item1: 0,
            item2: false
        }
        .convert_to(),
        Dest::E2 {
            item1: 0,
            item2: false
        }
    );
    assert_eq!(Src::E3.convert_to(), Dest::E3);
    assert_eq!(ConvertTo::<Dest>::try_convert_from(Dest::E3), Ok(Src::E3));
    assert_eq!(
        <Src<'static> as ConvertTo::<Dest>>::try_convert_from(Dest::E4),
        Err(Dest::E4)
    );
}

#[test]
fn non_zerocost() {
    #[convert_to(Dest)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Src {
        E1(usize, bool, &'static str) = 1,
    }
    #[convert_to(Dest)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Dest {
        E1(usize, bool, &'static str) = 2,
    }
    assert!(!<Src as ConvertTo<Dest>>::is_zerocost());
    assert_eq!(
        &Src::E1(123, true, "hello").convert_to(),
        &Dest::E1(123, true, "hello")
    );
    assert_eq!(
        &Src::try_convert_from(Dest::E1(123, true, "hello")),
        &Ok(Src::E1(123, true, "hello"))
    );
}

#[test]
fn with_args() {
    use fast_enum_conversion as mycrate;
    #[convert_to(Dest1<T>, Dest2<T, u8> @ mycrate)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Src<T> {
        E1(T) = 1,
        E2 = 2,
    }

    #[convert_to(Dest3<T, u8>)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Dest1<T> {
        E1(T) = 1,
        E2 = 2,
        E3 = 3,
    }

    #[convert_to(Dest3<T, U> @ mycrate)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Dest2<T, U> {
        E1(T) = 1,
        E2 = 2,
        E4(U) = 3,
    }

    #[convert_to(@ mycrate)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(u8)]
    enum Dest3<T, U> {
        E1(T) = 1,
        E2 = 2,
        E3 = 3,
        E4(U) = 4,
    }
    assert!(<Src<usize> as ConvertTo<Dest1<usize>>>::is_zerocost());
    assert_eq!(
        <Src<bool> as ConvertTo<Dest1<bool>>>::convert_to(Src::E1(true)),
        Dest1::E1(true)
    );
    assert_eq!(
        <Src<bool> as ConvertTo<Dest1<bool>>>::convert_to(Src::E2),
        Dest1::E2
    );
    assert_eq!(
        <Src<bool> as ConvertTo<Dest2<bool, u8>>>::convert_to(Src::E2),
        Dest2::E2
    );
}
