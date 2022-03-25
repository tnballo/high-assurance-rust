// ANCHOR: prime_test

//! This library does unoptimized primality testing.

/// Given a list of numbers, get the count of prime numbers present.
///
/// # Example
///
/// ```
/// use prime_test::count_primes;
///
/// let list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// assert_eq!(count_primes(&list), 4);
/// ```
#[doc(alias = "primality")]
pub fn count_primes(num_list: &[usize]) -> usize {
    // Unnecessary, unidiomatic check
    if num_list == [] {
        return 0;
    }

    num_list.iter().filter(|n| is_prime(**n)).count()
}

// Prime number check.
// This is a naive implementation,
// there are much more efficient implementations.
// Returns `true` if `n` is prime, `false` if not.
fn is_prime(n: usize) -> bool {
    if n <= 1 {
        return false;
    }

    for i in 2..n {
        if n % i == 0 {
            return false;
        }
    }

    true
}
// ANCHOR_END: prime_test

#[cfg(test)]
mod tests {
    use super::{count_primes, is_prime};

    #[test]
    fn test_count_primes() {
        let list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(count_primes(&list), 4);
    }

    #[test]
    fn test_is_prime() {
        // Positive
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(is_prime(5));
        assert!(is_prime(23));
        assert!(is_prime(83));
        assert!(is_prime(31));

        // Negative
        assert!(!is_prime(1));
        assert!(!is_prime(10));
        assert!(!is_prime(300));
        assert!(!is_prime(65));
        assert!(!is_prime(74));
        assert!(!is_prime(96));
    }
}
