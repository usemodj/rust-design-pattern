# 함수형 옵틱스

옵틱스(optics)는 중첩된 자료 구조의 일부를 읽거나 갱신하는 동작을 조합 가능한 추상화로 표현하는 함수형 프로그래밍 개념입니다. Rust에서 흔히 쓰이는 기능은 아니지만, 방문자 패턴이나 Serde처럼 제네릭 타입과 여러 트레이트가 맞물린 API를 이해하는 데 도움이 됩니다.

## Iso: 두 타입 사이의 변환

Iso는 두 타입 사이의 양방향 변환이 서로 역함수로 동작하는 옵틱스입니다. `FromStr`와 `ToString`도 서로 다른 표현을 변환할 수 있지만, 파싱이 실패하거나 문자열이 정규화될 수 있으므로 그 자체로 Iso의 역변환 법칙을 보장하지는 않습니다.

## 다형 Iso

다형 Iso는 변환 과정에서 타입 매개변수가 달라질 수 있는 형태로 일반화합니다. Rust의 제네릭 변환 트레이트는 여러 타입을 다룰 수 있지만, 제네릭이라는 사실만으로 다형 Iso가 되지는 않습니다. 또한 `to_string()`만으로 출력 형식이 JSON인지 다른 형식인지 드러나지 않고, 모든 타입이 동일한 직렬화 형식에 합의하는 것도 현실적이지 않습니다.

## Prism과 Serde

여러 데이터 형식을 지원하고 파싱 오류도 표현하려면, 타입 자체의 변환과 형식별 변환을 분리해야 합니다. Serde는 Rust 타입이 구현하는 `Serialize`·`Deserialize`, 데이터 형식별 `Serializer`·`Deserializer`, 그리고 타입별 구조를 연결하는 `Visitor`를 조합합니다.

역할을 나누면 각 자료형은 자신의 필드를 일반 데이터 모델로 변환하는 방법을 제공하고, JSON 같은 형식은 그 모델을 읽고 쓰는 방법을 구현합니다. `Deserialize`의 `derive` 매크로는 역직렬화 구현 생성을 도와주며, Serde의 역직렬화 과정은 `Visitor`를 사용합니다. 이 구분은 실패 가능한 변환을 이해하는 데 도움이 되지만, Serde API 자체가 옵틱스나 Prism 법칙을 구현한다는 뜻은 아닙니다.

Rust의 제네릭과 트레이트는 함수형 언어의 일부 고차 타입 구조를 직접 표현하지 못하는 경우가 있지만, 연관 타입·트레이트·매크로를 조합해 유사한 경계를 구현할 수 있습니다.

더 자세한 예시는 [Serde 역직렬화 안내](https://serde.rs/deserialize-struct.html)를 참고하세요.

## Lens와 Prism 구분하기

- **Lens:** 전체 값에서 항상 존재하는 부분을 읽고 갱신하는 관점을 표현합니다. 잘 구성된 Lens는 값을 설정한 뒤 읽으면 설정한 값이 나오고, 읽은 값을 다시 설정하면 전체가 그대로 유지되는 등의 법칙을 따릅니다.
- **Prism:** 열거형의 한 변형처럼 일부 값에만 적용되는 관점을 표현합니다. 일치하지 않는 값은 명시적으로 실패하거나 원래 값으로 남습니다.
- **Iso:** 위에서 설명한 것처럼 두 표현이 정보 손실 없이 서로 되돌릴 수 있어야 합니다.

## 실행 가능한 예제: 중첩된 선택적 주소

다음은 전용 옵틱스 라이브러리 없이 Rust의 참조와 `Option`으로 부분 접근을 표현한 예입니다. 주소가 없으면 읽기는 `None`, 갱신은 오류를 반환합니다. 아래 함수들은 Lens/Prism 라이브러리 자체가 아니라 그 목적을 이해하기 위한 간단한 접근자입니다.

`src/lib.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct Address {
    pub city: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Profile {
    pub address: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MissingAddress;

pub fn city(profile: &Profile) -> Option<&str> {
    profile.address.as_ref().map(|address| address.city.as_str())
}

pub fn set_city(profile: &mut Profile, value: &str) -> Result<(), MissingAddress> {
    let address = profile.address.as_mut().ok_or(MissingAddress)?;
    address.city = value.to_owned();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_updates_an_existing_address() {
        let mut profile = Profile {
            address: Some(Address { city: "서울".into() }),
        };
        assert_eq!(city(&profile), Some("서울"));
        set_city(&mut profile, "부산").unwrap();
        assert_eq!(city(&profile), Some("부산"));
    }

    #[test]
    fn missing_address_is_explicit_and_unchanged() {
        let mut profile = Profile { address: None };
        assert_eq!(city(&profile), None);
        assert_eq!(set_city(&mut profile, "부산"), Err(MissingAddress));
        assert_eq!(profile.address, None);
    }
}
```

`as_ref`와 `as_mut`는 `Option` 안의 값을 각각 공유·가변 참조로 빌립니다. 원본 구조체를 교체하거나 기본 주소를 암묵적으로 생성하지 않습니다. 예제를 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 `cargo test`로 테스트할 수 있습니다.

## 원문 소스 코드 예제

아래 원문 예제에는 개념을 설명하기 위한 의사 코드, 생략된 정의, `todo!()`가 포함되어 있습니다. 그대로 실행할 수 있는 완성된 프로그램은 아닙니다. 위의 실행 가능한 예제와 구분해 참고하세요.

### 예제 1

```rust,ignore
pub trait Deserializer<'de>: Sized {
    type Error: Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    // remainder omitted
}
```

### 예제 2

```rust,ignore
pub trait Visitor<'de>: Sized {
    type Value;

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error;

    // remainder omitted
}
```

### 예제 3

```text
case class ConcordanceSerDe {
  serialize: Concordance -> String
  deserialize: String -> Concordance
}
```

### 예제 4

```rust
use std::collections::HashMap;

struct Concordance {
    keys: HashMap<String, usize>,
    value_table: Vec<(usize, usize)>,
}

struct ConcordanceSerde {}

impl ConcordanceSerde {
    fn serialize(value: Concordance) -> String {
        todo!()
    }
    // invalid concordances are empty
    fn deserialize(value: String) -> Concordance {
        todo!()
    }
}
```

### 예제 5

```text
case class Serde[T] {
    deserialize(String) -> T
    serialize(T) -> String
}
```

### 예제 6

```rust,ignore
pub trait FromStr: Sized {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

pub trait ToString {
    fn to_string(&self) -> String;
}
```

### 예제 7

```rust,ignore
use anyhow;

use std::str::FromStr;

struct TestStruct {
    a: usize,
    b: String,
}

impl FromStr for TestStruct {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<TestStruct, Self::Err> {
        todo!()
    }
}

impl ToString for TestStruct {
    fn to_string(&self) -> String {
        todo!()
    }
}

fn main() {
    let a = TestStruct {
        a: 5,
        b: "hello".to_string(),
    };
    println!("Our Test Struct as JSON: {}", a.to_string());
}
```

### 예제 8

```text
case class Serde[T] {
    deserialize: Visitor[T] -> T
    serialize: T -> Visitor[T]
}

case class Visitor[T] {
    toJson: Visitor[T] -> String
    fromJson: String -> Visitor[T]
}
```

### 예제 9

```rust
trait Serde {
    type V;
    fn deserialize(visitor: Self::V) -> Self;
    fn serialize(self) -> Self::V;
}

trait Visitor {
    fn to_json(self) -> String;
    fn from_json(json: String) -> Self;
}
```

### 예제 10

```rust,ignore
#[derive(Default, Serde)] // the "Serde" derive creates the trait impl block
struct TestStruct {
    a: usize,
    b: String,
}

// user writes this macro to generate an associated visitor type
generate_visitor!(TestStruct);
```

### 예제 11

```rust,ignore
fn main() {
    let a = TestStruct { a: 5, b: "hello".to_string() };
    let a_data = a.serialize().to_json();
    println!("Our Test Struct as JSON: {a_data}");
    let b = TestStruct::deserialize(
        generated_visitor_for!(TestStruct)::from_json(a_data));
}
```

### 예제 12

```text
case class Serde[T, F] {
    serialize: T, F -> String
    deserialize: String, F -> Result[T, Error]
}
```

### 예제 13

```text
case class Serde[T] {
    serialize: F -> String
    deserialize F, String -> Result[T, Error]
}

case class VisitorForT {
    build: F, String -> Result[T, Error]
    decompose: F, T -> String
}

case class SerdeFormat[T, V] {
    toString: T, V -> String
    fromString: V, String -> Result[T, Error]
}
```

### 예제 14

```rust,ignore
pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}
```

### 예제 15

```rust,ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct IdRecord {
    name: String,
    customer_id: String,
}
```

## 참고 자료

- [Rust의 함수형 기능](/docs/functional-overview)
- [`Option::as_ref` 표준 라이브러리 문서](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_ref)
- [`Option::as_mut` 표준 라이브러리 문서](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_mut)
- [Serde: 직접 `Deserialize` 구현하기](https://serde.rs/deserialize-struct.html)
