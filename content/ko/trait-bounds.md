# 사용자 정의 트레이트로 복잡한 바운드 줄이기

## 설명

제네릭 타입의 트레이트 바운드는 해당 타입이 어떤 동작을 제공해야 하는지 지정합니다. 클로저를 받는 API에서 반환값의 타입과 오류까지 제한하려고 하면 바운드가 길어질 수 있습니다. 같은 제약을 여러 구조체와 `impl`에 반복하면 코드를 읽고 수정하기도 어려워집니다.

이럴 때 반복되는 역할에 이름을 붙인 사용자 정의 트레이트를 도입할 수 있습니다. 그 트레이트에 연관 타입을 두면, 제네릭 매개변수로 따로 전달하던 반환 타입을 트레이트 구현에 연결할 수 있습니다. 결과적으로 사용하는 쪽은 여러 세부 제약을 반복하는 대신 `G: Getter`처럼 의도를 나타내는 한 가지 바운드를 표현할 수 있습니다.

## 예제의 상황

원문은 여러 출처에서 값을 가져와 상태를 보여 주는 모니터링 시스템을 예로 듭니다. 출처마다 값의 타입은 다를 수 있으며, 시스템은 값을 가져온 뒤 문제 여부를 나타내는 상태를 계산합니다. 예를 들어 사용 가능한 메모리가 임계치보다 높은지, 사용자 ID가 `0`인 계정의 이름이 `"root"`인지 확인할 수 있습니다.

값을 가져오는 작업은 클로저로 표현할 수 있고, 운영체제에서 값을 가져오는 과정은 실패할 수 있으므로 반환 타입에 `Result`를 사용합니다. 가져온 값의 상태를 판정하는 함수도 값 타입에 따라 달라집니다.

## 제약이 반복되는 원래 설계

아래는 원문의 첫 번째 형태입니다. `G`는 값을 가져오는 클로저, `S`는 값을 상태로 변환하는 함수, `T`는 실제 값 타입입니다. `G`의 반환 타입 `Result<T, Error>`를 표현하려고 `T`를 별도의 제네릭 매개변수로 두며, 같은 제약이 구조체와 `impl`에 반복됩니다.

```rust,ignore
use std::fmt::Display;

struct Value<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> {
    value: Option<T>,
    getter: G,
    status: S,
}

impl<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> Value<G, S, T> {
    pub fn update(&mut self) -> Result<(), Error> {
        (self.getter)().map(|v| self.value = Some(v))
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.value().map(&self.status)
    }
}

// ...

enum Status {
    // ...
}

struct Error {
    // ...
}
```

`FnMut() -> Result<T, Error>`는 `getter`가 인자 없이 호출되고, 호출될 때 캡처한 상태를 변경할 수도 있으며, 성공하면 `T`, 실패하면 `Error`를 돌려준다는 뜻입니다. `FnMut`를 사용하면 한 번만 호출하는 것이 아니라 반복해서 호출할 수 있고, 호출 과정에서 클로저의 상태도 변경할 수 있습니다.

상태 판정 함수는 `Fn(&T) -> Status`입니다. 값의 참조를 받아 상태를 계산하며, 값을 가져오는 클로저와 달리 상태 판정 자체는 클로저의 캡처 상태를 바꾸지 않는 `Fn`으로 제한합니다. `T: Display`는 값이 표시 가능한 타입이어야 한다는 요구입니다. 원문 발췌에서는 이 표시 요구가 예제의 나머지 코드에서 사용될 수 있도록 타입에 포함되어 있습니다.

이 설계도 동작하지만, 호출 시 타입 서명 곳곳에 `FnMut() -> Result<T, Error>`와 `T`가 드러납니다. 특히 getter의 `Result` 반환 타입이 읽기 흐름을 복잡하게 만들 수 있습니다.

## Getter 트레이트로 역할에 이름 붙이기

`Getter`는 값을 가져오는 동작을 하나의 개념으로 표현합니다. 연관 타입 `Output`은 각 getter가 돌려주는 값의 타입을 지정하고, `get_value`는 오류가 발생할 수 있는 호출을 나타냅니다.

```rust,ignore
use std::fmt::Display;

trait Getter {
    type Output: Display;

    fn get_value(&mut self) -> Result<Self::Output, Error>;
}

impl<F: FnMut() -> Result<T, Error>, T: Display> Getter for F {
    type Output = T;

    fn get_value(&mut self) -> Result<Self::Output, Error> {
        self()
    }
}

struct Value<G: Getter, S: Fn(&G::Output) -> Status> {
    value: Option<G::Output>,
    getter: G,
    status: S,
}

// ...
enum Status {}
struct Error;
```

이 일반 구현은 `FnMut() -> Result<T, Error>`를 만족하고 결과 타입 `T`가 `Display`인 모든 클로저에 `Getter`를 구현합니다. `Output = T`로 연결했기 때문에 `Value`는 `T`를 별도 제네릭 매개변수로 받지 않고 `G::Output`으로 참조할 수 있습니다.

이제 구조체의 바운드는 `G: Getter`로 짧아집니다. 상태 함수의 입력도 `&G::Output`으로 표현하므로 두 제네릭 타입 사이의 관계가 명확합니다. 같은 `Getter` 트레이트를 구현하는 별도 타입도 받을 수 있으므로, 이 추상화는 클로저에만 한정되지 않습니다.

## 완전한 예제와 테스트

다음 예제는 원문의 `Getter`와 `Value` 설계를 실행 가능한 라이브러리 코드로 확장합니다. 모니터링 값의 상태를 판정하고, 성공 및 실패 처리와 클로저·사용자 정의 getter 구현을 테스트합니다.

### 디렉터리 구조

```text
getter-monitor/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    └── monitor.rs
```

### 패키지 매니페스트

```toml
[package]
name = "getter-monitor"
version = "0.1.0"
edition = "2024"

[lib]
name = "getter_monitor"
path = "src/lib.rs"
```

### 라이브러리 구현과 단위 테스트

`src/lib.rs`:

```rust
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Healthy,
    Warning,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unavailable,
}

pub trait Getter {
    type Output: Display;

    fn get_value(&mut self) -> Result<Self::Output, Error>;
}

impl<F, T> Getter for F
where
    F: FnMut() -> Result<T, Error>,
    T: Display,
{
    type Output = T;

    fn get_value(&mut self) -> Result<Self::Output, Error> {
        self()
    }
}

pub struct Value<G, S>
where
    G: Getter,
    S: Fn(&G::Output) -> Status,
{
    value: Option<G::Output>,
    getter: G,
    status: S,
}

impl<G, S> Value<G, S>
where
    G: Getter,
    S: Fn(&G::Output) -> Status,
{
    pub fn new(getter: G, status: S) -> Self {
        Self {
            value: None,
            getter,
            status,
        }
    }

    pub fn update(&mut self) -> Result<(), Error> {
        let value = self.getter.get_value()?;
        self.value = Some(value);
        Ok(())
    }

    pub fn value(&self) -> Option<&G::Output> {
        self.value.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.value().map(&self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Status, Value};

    #[test]
    fn closure_getter_updates_value_and_status() {
        let getter = || Ok(512_u64);
        let status = |free_memory: &u64| {
            if *free_memory >= 1024 {
                Status::Healthy
            } else {
                Status::Warning
            }
        };
        let mut monitor = Value::new(getter, status);

        assert_eq!(monitor.value(), None);
        assert_eq!(monitor.status(), None);

        monitor.update().unwrap();

        assert_eq!(monitor.value(), Some(&512));
        assert_eq!(monitor.status(), Some(Status::Warning));
    }

    #[test]
    fn failed_update_does_not_replace_the_last_value() {
        let mut calls = 0;
        let getter = move || {
            calls += 1;
            if calls == 1 {
                Ok(2048_u64)
            } else {
                Err(Error::Unavailable)
            }
        };
        let status = |free_memory: &u64| {
            if *free_memory >= 1024 {
                Status::Healthy
            } else {
                Status::Warning
            }
        };
        let mut monitor = Value::new(getter, status);

        monitor.update().unwrap();
        assert_eq!(monitor.update(), Err(Error::Unavailable));

        assert_eq!(monitor.value(), Some(&2048));
        assert_eq!(monitor.status(), Some(Status::Healthy));
    }
}
```

`update`는 getter의 결과를 먼저 얻은 다음 성공한 경우에만 저장합니다. 따라서 getter가 실패하면 기존 값은 유지됩니다. 단위 테스트는 초기 상태, 성공 후 상태 판정, 실패한 갱신이 이전 값을 덮어쓰지 않는 동작을 확인합니다.

### 별도의 Getter 타입과 통합 테스트

클로저가 아닌 명시적인 타입도 같은 트레이트 계약을 구현할 수 있습니다. 다음 통합 테스트의 `FixedGetter`는 고정된 메모리 값을 제공합니다.

`tests/monitor.rs`:

```rust
use getter_monitor::{Error, Getter, Status, Value};

struct FixedGetter(u64);

impl Getter for FixedGetter {
    type Output = u64;

    fn get_value(&mut self) -> Result<Self::Output, Error> {
        Ok(self.0)
    }
}

#[test]
fn custom_getter_uses_the_same_value_api() {
    let status = |free_memory: &u64| {
        if *free_memory >= 1024 {
            Status::Healthy
        } else {
            Status::Warning
        }
    };
    let mut monitor = Value::new(FixedGetter(4096), status);

    monitor.update().unwrap();

    assert_eq!(monitor.value(), Some(&4096));
    assert_eq!(monitor.status(), Some(Status::Healthy));
}
```

패키지 루트에서 다음 명령을 실행하면 단위 테스트와 통합 테스트를 모두 실행할 수 있습니다.

```console
cargo test
```

이 통합 테스트는 외부 사용자가 `Getter`를 구현하고, 동일한 `Value` API를 사용할 수 있는지도 확인합니다. 즉 `Getter`는 단지 긴 타입 바운드를 줄이는 별칭이 아니라, 여러 값 공급자가 공유할 수 있는 명시적인 확장 지점입니다.

## 장점

- **타입 바운드를 단순화합니다.** 반복되던 `FnMut() -> Result<T, Error>` 제약을 `G: Getter`로 줄이고, 별도의 타입 매개변수 `T`를 연관 타입으로 표현할 수 있습니다.
- **의도를 잘 드러냅니다.** `Getter`라는 이름은 함수형 제약의 세부 문법 대신 타입의 역할을 설명합니다. 적절한 이름은 제네릭 API를 더 표현력 있게 만듭니다.
- **확장 지점이 생깁니다.** 클로저에 대한 포괄 구현 외에도 상태를 가진 별도 타입이나 다른 도메인 타입이 트레이트를 구현할 수 있습니다. 필요하다면 모든 구현에 공통으로 적용할 기본 메서드도 트레이트에 추가할 수 있습니다.

## 단점과 주의점

- **이름과 위치를 정해야 합니다.** 새 트레이트를 공개 API 어디에 둘지, 어떤 이름이 역할을 가장 잘 나타내는지 결정하고 문서화해야 합니다.
- **사용자가 알아야 할 개념이 늘어납니다.** 원래 기능을 사용하려는 사람은 `Getter`가 무엇인지 알아야 하므로 관련 문서가 필요합니다.
- **클로저 지원이 바로 드러나지 않을 수 있습니다.** `Getter`만 보면 단순한 클로저도 사용할 수 있다는 점이 분명하지 않을 수 있습니다. 예제에서 클로저를 직접 보여 주고, 일반 구현이 클로저를 지원한다는 점을 설명하세요.
- **항상 더 단순한 것은 아닙니다.** 트레이트가 한 번만 쓰이고 바운드도 짧다면 새 추상화가 오히려 이해 비용을 늘릴 수 있습니다. 반복되는 복잡성이 실제 문제인지 확인한 뒤 도입하세요.

## 참고 자료

- [Rust Design Patterns 원문: Use custom traits to avoid complex type bounds](https://rust-unofficial.github.io/patterns/patterns/structural/trait-for-bounds.html)
- [The Rust Programming Language: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Rust Programming Language: 연관 타입을 사용하는 고급 트레이트](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [The Rust Reference: 트레이트 및 수명 바운드](https://doc.rust-lang.org/reference/trait-bounds.html)
- [`FnMut` 표준 라이브러리 문서](https://doc.rust-lang.org/std/ops/trait.FnMut.html)
- [The Rust Programming Language: 클로저](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [`Result` 표준 라이브러리 문서](https://doc.rust-lang.org/std/result/enum.Result.html)
