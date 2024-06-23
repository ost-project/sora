/// possible compiler hint that a branch is likely
///
/// Technique borrowed from here: <https://github.com/rust-lang/hashbrown/pull/209>
macro_rules! likely {
    ($e:expr) => {{
        #[inline]
        #[cold]
        fn cold() {}

        let cond = $e;

        if !cond {
            cold();
        }

        cond
    }};
}

/// possible compiler hint that a branch is unlikely
///
/// Technique borrowed from here: <https://github.com/rust-lang/hashbrown/pull/209>
macro_rules! unlikely {
    ($e:expr) => {{
        #[inline]
        #[cold]
        fn cold() {}

        let cond = $e;

        if cond {
            cold();
        }

        cond
    }};
}

pub(crate) use likely;
pub(crate) use unlikely;
