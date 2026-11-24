# 생성자

## 설명

Rust에는 언어 문법으로 제공되는 생성자가 없습니다. 대신 관례적으로 연관 함수(associated function) `new`를 사용해 값을 만듭니다. 연관 함수는 특정 타입에 속하지만 `self`를 받지 않으므로 `Second::new(42)`와 같이 타입 이름으로 호출합니다.

````rust
/// Time in seconds.
///
/// # Example
///
/// ```
/// let s = Second::new(42);
/// assert_eq!(42, s.value());
/// ```
pub struct Second {
    value: u64,
}

impl Second {
    // Constructs a new instance of [`Second`].
    // Note this is an associated function - no self.
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// Returns the value in seconds.
    pub fn value(&self) -> u64 {
        self.value
    }
}
````

## 기본 생성자

Rust에서는 `Default` 트레이트를 구현해 타입의 기본값을 제공할 수 있습니다. 호출자는 `Second::default()`를 사용해 0초를 나타내는 값을 만들 수 있습니다.

````rust
/// Time in seconds.
///
/// # Example
///
/// ```
/// let s = Second::default();
/// assert_eq!(0, s.value());
/// ```
pub struct Second {
    value: u64,
}

impl Second {
    /// Returns the value in seconds.
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Default for Second {
    fn default() -> Self {
        Self { value: 0 }
    }
}
````

모든 필드의 타입이 `Default`를 구현한다면 직접 구현하는 대신 `Default`를 파생할 수도 있습니다. `Second`의 `u64` 필드는 기본값을 제공하므로 아래처럼 작성할 수 있습니다.

````rust
/// Time in seconds.
///
/// # Example
///
/// ```
/// let s = Second::default();
/// assert_eq!(0, s.value());
/// ```
#[derive(Default)]
pub struct Second {
    value: u64,
}

impl Second {
    /// Returns the value in seconds.
    pub fn value(&self) -> u64 {
        self.value
    }
}
````

타입이 `Default`와 인자 없는 `new` 생성자를 모두 제공하는 것은 흔하고 기대되는 관례입니다. Rust에서 값을 만드는 대표적인 관례는 `new`이고, 사용자는 해당 함수가 있기를 기대합니다. 기본 생성자가 인자를 받지 않는 것이 합리적이라면 그 기능이 `Default`와 같더라도 `new()`를 제공하는 편이 좋습니다.

`Default`를 구현하거나 파생하면 해당 구현을 요구하는 곳에서 타입을 사용할 수 있습니다. 특히 표준 라이브러리의 `*_or_default` 계열 함수는 값이 없을 때 `Default` 구현으로 기본값을 만듭니다.

## 참고 자료

- `Default` 트레이트에 대한 자세한 내용은 [Default 관용구](/docs/default-trait)를 참고하세요.
- 여러 설정을 조합해 객체를 만들어야 한다면 [Builder 패턴](/docs/builder)을 참고하세요.
- `Default`와 `new`를 함께 구현하는 방법은 [Rust API 가이드라인 C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)를 참고하세요.
- [연관 함수](https://doc.rust-lang.org/stable/book/ch05-03-method-syntax.html#associated-functions)
- [`Default` 트레이트 문서](https://doc.rust-lang.org/stable/std/default/trait.Default.html)
- 표준 라이브러리의 [`or_default` 검색 결과](https://doc.rust-lang.org/stable/std/?search=or_default)
