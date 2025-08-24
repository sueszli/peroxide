// Function application in reverse order.
// `T x f = f x` (apply function f to value x)
pub trait Thrush {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized;
}

impl<T> Thrush for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized,
    {
        f(self)
    }
}

// Performs side effects while preserving the original value.
// `K x y = x` (Kestrel ignores the second argument)
pub trait Kestrel {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self);
}

impl<T> Kestrel for T {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }
}

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_thrush_basic_function_application() {
        let result = 5.pipe(|x| x * 2);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_thrush_chaining() {
        let result = 10.pipe(|x| x + 5).pipe(|x| x * 2).pipe(|x| x - 3);
        assert_eq!(result, 27);
    }

    #[test]
    fn test_thrush_type_conversion() {
        let result = 42.pipe(|x| x.to_string());
        assert_eq!(result, "42");
    }

    #[test]
    fn test_kestrel_basic_tap() {
        let captured = RefCell::new(0);
        let result = 42.tap(|x| {
            *captured.borrow_mut() = *x;
        });

        assert_eq!(result, 42);
        assert_eq!(*captured.borrow(), 42);
    }

    #[test]
    fn test_kestrel_preserves_value() {
        let original = vec![1, 2, 3];
        let result = original.clone().tap(|v| {
            println!("Vector length: {}", v.len());
        });

        assert_eq!(result, original);
    }

    #[test]
    fn test_kestrel_chaining() {
        let side_effects = RefCell::new(Vec::new());

        let result = "hello"
            .tap(|s| side_effects.borrow_mut().push(s.len()))
            .tap(|s| side_effects.borrow_mut().push(s.chars().count()))
            .tap(|_| side_effects.borrow_mut().push(999));

        assert_eq!(result, "hello");
        assert_eq!(*side_effects.borrow(), vec![5, 5, 999]);
    }
}
