//! Runtime-owned builtin and intrinsic hook support.

use crate::{
    containers::{FolArray, FolMap, FolSeq, FolSet, FolVec},
    shell::{FolError, FolOption},
    strings::FolStr,
    value::FolInt,
};

pub trait FolLength {
    fn fol_length(&self) -> FolInt;
}

pub trait FolEchoFormat {
    fn fol_echo_format(&self) -> String;
}

fn join_echo<I>(items: I) -> String
where
    I: IntoIterator<Item = String>,
{
    items.into_iter().collect::<Vec<_>>().join(", ")
}

pub fn render_echo<T: FolEchoFormat + ?Sized>(value: &T) -> String {
    value.fol_echo_format()
}

pub fn echo<T: FolEchoFormat>(value: T) -> T {
    println!("{}", value.fol_echo_format());
    value
}

impl FolEchoFormat for i64 {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for f64 {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for bool {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for char {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for FolStr {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl<T: FolEchoFormat, const N: usize> FolEchoFormat for FolArray<T, N> {
    fn fol_echo_format(&self) -> String {
        format!("arr[{}]", join_echo(self.iter().map(render_echo)))
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolVec<T> {
    fn fol_echo_format(&self) -> String {
        format!(
            "vec[{}]",
            join_echo(self.as_slice().iter().map(render_echo))
        )
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolSeq<T> {
    fn fol_echo_format(&self) -> String {
        format!(
            "seq[{}]",
            join_echo(self.as_slice().iter().map(render_echo))
        )
    }
}

impl<T: FolEchoFormat + Ord> FolEchoFormat for FolSet<T> {
    fn fol_echo_format(&self) -> String {
        format!("set{{{}}}", join_echo(self.as_set().iter().map(render_echo)))
    }
}

impl<K: FolEchoFormat + Ord, V: FolEchoFormat> FolEchoFormat for FolMap<K, V> {
    fn fol_echo_format(&self) -> String {
        format!(
            "map{{{}}}",
            join_echo(
                self.as_map()
                    .iter()
                    .map(|(key, value)| format!("{}: {}", render_echo(key), render_echo(value)))
            )
        )
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolOption<T> {
    fn fol_echo_format(&self) -> String {
        match self {
            FolOption::Some(value) => format!("some({})", render_echo(value)),
            FolOption::Nil => "nil".to_string(),
        }
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolError<T> {
    fn fol_echo_format(&self) -> String {
        format!("err({})", render_echo(self.as_ref()))
    }
}

impl FolLength for FolStr {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T, const N: usize> FolLength for FolArray<T, N> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T> FolLength for FolVec<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T> FolLength for FolSeq<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T: Ord> FolLength for FolSet<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<K: Ord, V> FolLength for FolMap<K, V> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

pub fn module_name() -> &'static str {
    "builtins"
}

#[cfg(test)]
mod tests {
    use super::{echo, render_echo, FolEchoFormat, FolLength};
    use crate::{
        containers::{FolArray, FolMap, FolSeq, FolSet, FolVec},
        shell::{FolError, FolOption},
        strings::FolStr,
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn runtime_length_trait_covers_current_v1_families() {
        let text = FolStr::from("Ada");
        let array: FolArray<i64, 3> = [1, 2, 3];
        let vector = FolVec::from_items(vec![1, 2]);
        let sequence = FolSeq::from_items(vec![1, 2, 3, 4]);
        let set = FolSet::new(BTreeSet::from([1, 2, 3]));
        let map = FolMap::new(BTreeMap::from([("ada", 1), ("lin", 2)]));

        assert_eq!(text.fol_length(), 3);
        assert_eq!(array.fol_length(), 3);
        assert_eq!(vector.fol_length(), 2);
        assert_eq!(sequence.fol_length(), 4);
        assert_eq!(set.fol_length(), 3);
        assert_eq!(map.fol_length(), 2);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DemoEcho(&'static str);

    impl FolEchoFormat for DemoEcho {
        fn fol_echo_format(&self) -> String {
            format!("demo({})", self.0)
        }
    }

    #[test]
    fn runtime_echo_trait_and_helpers_freeze_backend_hook_boundary() {
        let value = DemoEcho("trace");

        assert_eq!(render_echo(&value), "demo(trace)");
        assert_eq!(echo(value.clone()), value);
    }

    #[test]
    fn runtime_echo_formats_builtin_scalars_and_strings() {
        let text = FolStr::from("Ada");

        assert_eq!(render_echo(&7i64), "7");
        assert_eq!(render_echo(&3.5f64), "3.5");
        assert_eq!(render_echo(&true), "true");
        assert_eq!(render_echo(&'x'), "x");
        assert_eq!(render_echo(&text), "Ada");
        assert_eq!(echo(text.clone()), text);
    }

    #[test]
    fn runtime_echo_formats_current_v1_container_families() {
        let array: FolArray<i64, 3> = [1, 2, 3];
        let vector = FolVec::from_items(vec![1, 2, 3]);
        let sequence = FolSeq::from_items(vec![1, 2, 3]);
        let set = FolSet::from_items(vec![3, 1, 2]);
        let map = FolMap::from_pairs(vec![(FolStr::from("lin"), 2), (FolStr::from("ada"), 1)]);

        assert_eq!(render_echo(&array), "arr[1, 2, 3]");
        assert_eq!(render_echo(&vector), "vec[1, 2, 3]");
        assert_eq!(render_echo(&sequence), "seq[1, 2, 3]");
        assert_eq!(render_echo(&set), "set{1, 2, 3}");
        assert_eq!(render_echo(&map), "map{ada: 1, lin: 2}");
    }

    #[test]
    fn runtime_echo_formats_current_v1_shell_families() {
        let some = FolOption::some(FolStr::from("Ada"));
        let nil = FolOption::<FolStr>::nil();
        let error = FolError::new(FolStr::from("broken"));

        assert_eq!(render_echo(&some), "some(Ada)");
        assert_eq!(render_echo(&nil), "nil");
        assert_eq!(render_echo(&error), "err(broken)");
    }

    #[test]
    fn runtime_echo_formats_nested_v1_values_stably() {
        let nested_seq = FolSeq::from_items(vec![
            FolOption::some(FolStr::from("Ada")),
            FolOption::nil(),
        ]);
        let nested_map = FolMap::from_pairs(vec![
            (
                FolStr::from("left"),
                FolError::new(FolSeq::from_items(vec![1i64, 2, 3])),
            ),
            (
                FolStr::from("right"),
                FolError::new(FolSeq::from_items(vec![4i64, 5])),
            ),
        ]);

        assert_eq!(render_echo(&nested_seq), "seq[some(Ada), nil]");
        assert_eq!(
            render_echo(&nested_map),
            "map{left: err(seq[1, 2, 3]), right: err(seq[4, 5])}"
        );
    }
}
