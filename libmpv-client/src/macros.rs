#[cfg(debug_assertions)]
macro_rules! build_debug_loc {
    ($var:expr) => {
        Some(crate::error::DebugLoc {
            file: file!(),
            line: line!(),
            function: {
                fn f() {}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                type_name_of(f)
            },
            variable: Some(stringify!($var)),
        })
    };
}

#[cfg(not(debug_assertions))]
macro_rules! build_debug_loc {
    ($var:expr) => {
        None
    };
}

macro_rules! check_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            return Err($crate::Error::Rust($crate::error::RustError::Pointer(build_debug_loc!($ptr))));
        }
    };
}

#[macro_export]
/// Construct a [`NodeArray`](crate::NodeArray) from a list of items that implement [`Into<Node>`].
///
/// See the [list of traits on `Node`](crate::Node#trait-implementations).
///
/// # Example
/// ```
///# use libmpv_client::{node_array, Node, NodeArray};
/// let array_node = node_array!("one", 2, 3.14);
///
/// let test_array_node = Node::Array(vec![
///     Node::String("one".to_string()),
///     Node::Int64(2),
///     Node::Double(3.14)
/// ]);
///
/// assert_eq!(array_node, test_array_node);
/// ```
macro_rules! node_array {
    (
        $($elem:expr),*$(,)?
    ) => {
        $crate::Node::Array(vec![$($crate::Node::from($elem),)*])
    };
}

#[macro_export]
/// Construct a [`NodeMap`](crate::NodeMap) from a list of `(Into<String>, Into<Node>)` tuples.
///
/// See the [list of traits on `Node`](crate::Node#trait-implementations).
///
/// # Example
/// ```
///# use std::collections::HashMap;
///# use libmpv_client::{node_map, Node, NodeMap};
/// let array_node = node_map! {
///     ("1", "one"),
///     ("two", 2),
///     ("pi", 3.14),
/// };
///
/// let test_array_node = Node::Map(HashMap::from([
///     ("1".to_string(), Node::String("one".to_string())),
///     ("two".to_string(), Node::Int64(2)),
///     ("pi".to_string(), Node::Double(3.14))
/// ]));
///
/// assert_eq!(array_node, test_array_node);
/// ```
macro_rules! node_map {
    (
        $(($key:expr, $val:expr)),*$(,)?
    ) => {
        $crate::Node::Map(std::collections::HashMap::from([$(($key.to_string(), $crate::Node::from($val)),)*]))
    };
}