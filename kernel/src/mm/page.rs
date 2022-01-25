pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    /// The page size in bytes.
    const SIZE: usize;
   /// A string representation of the page size for debug output.
    const SIZE_AS_DEBUG_STR: &'static str;
}
