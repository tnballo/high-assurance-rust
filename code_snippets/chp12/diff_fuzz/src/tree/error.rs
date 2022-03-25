/// Errors for fallible operations.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum SgError {
    /// Cannot construct instance, maximum supported capacity exceeded.
    MaximumCapacityExceeded,

    /// Requested operation cannot complete, stack storage is full.
    StackCapacityExceeded,

    /*
    /// Requested operation cannot complete, heap storage is full.
    HeapCapacityExceeded,
    */
    /// Reserved for future use
    #[doc(hidden)]
    Reserved3,

    /// Reserved for future use
    #[doc(hidden)]
    Reserved4,

    /// Reserved for future use
    #[doc(hidden)]
    Reserved5,

    /// Reserved for future use
    #[doc(hidden)]
    Reserved6,

    /// Reserved for future use
    #[doc(hidden)]
    Reserved7,

    /// Invalid rebalance factor requested, cannot set.
    RebalanceFactorOutOfRange,
}

/*

Requires nightly feature:

#[cfg(test)]
mod tests {
    use crate::SgError;
    use std::mem::variant_count;

    #[test]
    fn test_err_var_cnt() {
        assert_eq!(variant_count::<SgError>(), 8);
    }
}
*/
