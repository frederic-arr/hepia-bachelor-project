#![feature(decl_macro)]

/// Like [`assert!`], but panics with `"INVARIANT VIOLATION: ..."`
/// prefix.
///
/// # Examples
///
/// Passes when the expression is `true`:
///
/// ```
/// # use invariant_macros::invariant;
/// fn some_computation() -> bool {
///     // Some expensive computation here
///     true
/// }
///
/// invariant!(some_computation(), "computation failed");
/// ```
///
/// Panics when the expression is `false`:
///
/// ```should_panic
/// # use invariant_macros::invariant;
/// fn some_computation() -> bool {
///     // Some expensive computation here
///     false
/// }
///
/// invariant!(some_computation(), "computation failed");
/// ```
pub macro invariant {
    ($cond:expr $(,)?) => {
        ::core::assert!($cond, "INVARIANT VIOLATION")
    },

    ($cond:expr, $($arg:tt)+) => {
        ::core::assert!($cond, "INVARIANT VIOLATION: {}", ::core::format_args!($($arg)+))
    }
}

/// Like [`invariant_not!`], but inverts the condition.
///
/// # Examples
///
/// Passes when the expression is `false`:
///
/// ```
/// # use invariant_macros::invariant_not;
/// fn some_computation() -> bool {
///     // Some expensive computation here
///     false
/// }
///
/// invariant_not!(some_computation(), "computation succeedeed");
/// ```
///
/// Panics when the expression is `true`:
///
/// ```should_panic
/// # use invariant_macros::invariant_not;
/// fn some_computation() -> bool {
///     // Some expensive computation here
///     true
/// }
///
/// invariant_not!(some_computation(), "computation succeedeed");
/// ```
pub macro invariant_not {
    ($cond:expr $(,)?) => {
        ::core::assert!(!($cond), "INVARIANT VIOLATION")
    },

    ($cond:expr, $($arg:tt)+) => {
        ::core::assert!(!($cond), "INVARIANT VIOLATION: {}", ::core::format_args!($($arg)+))
    }
}

/// Like [`assert_eq!`], but panics with `"INVARIANT VIOLATION: ..."`
/// prefix.
///
/// # Examples
///
/// Passes when the values are equal:
///
/// ```
/// # use invariant_macros::invariant_eq;
/// let a = 3;
/// let b = 3;
/// invariant_eq!(a, b, "unexpected inequality");
/// ```
///
/// Panics when the values are not equal:
///
/// ```should_panic
/// # use invariant_macros::invariant_eq;
/// let a = 3;
/// let b = 2;
/// invariant_eq!(a, b, "unexpected inequality");
/// ```
pub macro invariant_eq {
    ($left:expr, $right:expr $(,)?) => {
        ::core::assert_eq!($left, $right, "INVARIANT VIOLATION")
    },

    ($left:expr, $right:expr, $($arg:tt)+) => {
        ::core::assert_eq!($left, $right, "INVARIANT VIOLATION: {}", ::core::format_args!($($arg)+))
    }
}

/// Like [`assert_ne!`], but panics with `"INVARIANT VIOLATION: ..."`
/// prefix.
///
/// # Examples
///
/// Passes when the values are not equal:
///
/// ```
/// # use invariant_macros::invariant_ne;
/// let a = 3;
/// let b = 2;
/// invariant_ne!(a, b, "unexpected equality");
/// ```
///
/// Panics when the values are equal:
///
/// ```should_panic
/// # use invariant_macros::invariant_ne;
/// let a = 3;
/// let b = 3;
/// invariant_ne!(a, b, "unexpected equality");
/// ```
pub macro invariant_ne {
    ($left:expr, $right:expr $(,)?) => {
        ::core::assert_ne!($left, $right, "INVARIANT VIOLATION")
    },

    ($left:expr, $right:expr, $($arg:tt)+) => {
        ::core::assert_ne!($left, $right, "INVARIANT VIOLATION: {}", ::core::format_args!($($arg)+))
    }
}

/// Like [`assert_matches!`], but panics with `"INVARIANT VIOLATION: ..."`
/// prefix.
///
/// # Examples
///
/// Passes when the value matches:
///
/// ```
/// # use invariant_macros::invariant_matches;
/// # #[derive(Debug)] enum State { Ready, Running, Stopped }
/// let state = State::Ready;
/// invariant_matches!(
///     state,
///     State::Ready | State::Running,
///     "unexpected state"
/// );
/// ```
///
/// Panics when the value does not match:
///
/// ```should_panic
/// # use invariant_macros::invariant_matches;
/// # #[derive(Debug)] enum State { Ready, Running, Stopped }
/// let state = State::Stopped;
/// invariant_matches!(
///     state,
///     State::Ready | State::Running,
///     "unexpected state"
/// );
/// ```
pub macro invariant_matches {
    ($left:expr, $(|)? $($pattern:pat_param)|+ $(if $guard:expr)? $(,)?) => {
        ::core::assert_matches!(
            $left,
            $($pattern)|+ $(if $guard)?,
            "INVARIANT VIOLATION"
        )
    },

    ($left:expr, $(|)? $($pattern:pat_param)|+ $(if $guard:expr)?, $($arg:tt)+) => {
        ::core::assert_matches!(
            $left,
            $($pattern)|+ $(if $guard)?,
            "INVARIANT VIOLATION: {}",
            ::core::format_args!($($arg)+)
        )
    }
}
