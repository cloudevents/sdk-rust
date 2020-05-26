macro_rules! assert_match_pattern (
    ($e:expr, $p:pat) => (
        match $e {
            $p => (),
            _ => panic!(r#"assertion failed (value doesn't match pattern):
value: `{:?}`,
pattern: `{}`"#, $e, stringify!($p))
        }
    )
);
