# `mem::take`와 `mem::replace`

## 설명

`A { name: String, x: u8 }`와 `B { name: String }`라는 변형을 가진 `MyEnum`을 생각해 봅시다. `&mut MyEnum`으로 빌린 값을 제자리에서 바꾸되, `x`가 0일 때만 `A`를 `B`로 바꾸고 기존 `name`은 그대로 유지하고 싶습니다.

`name`을 복제하지 않고도 이 작업을 할 수 있습니다.

## 예제

```rust
use std::mem;

enum MyEnum {
    A { name: String, x: u8 },
    B { name: String },
}

fn a_to_b(e: &mut MyEnum) {
    if let MyEnum::A { name, x: 0 } = e {
        // This takes out our `name` and puts in an empty String instead
        // (note that empty strings don't allocate).
        // Then, construct the new enum variant (which will
        // be assigned to `*e`).
        *e = MyEnum::B {
            name: mem::take(name),
        }
    }
}
```

이 방식은 변형이 더 많은 열거형에도 적용할 수 있습니다. 아래에서는 `A`와 `B`를 서로 바꾸고, 필드가 없는 `C`와 `D`도 서로 바꿉니다.

```rust
use std::mem;

enum MultiVariateEnum {
    A { name: String },
    B { name: String },
    C,
    D,
}

fn swizzle(e: &mut MultiVariateEnum) {
    use MultiVariateEnum::*;
    *e = match e {
        // Ownership rules do not allow taking `name` by value, but we cannot
        // take the value out of a mutable reference, unless we replace it:
        A { name } => B {
            name: mem::take(name),
        },
        B { name } => A {
            name: mem::take(name),
        },
        C => D,
        D => C,
    }
}
```

## 동기

열거형을 다룰 때 현재 값을 확인하고, 경우에 따라 다른 변형으로 제자리에서 바꾸고 싶을 수 있습니다. 보통 이 작업은 두 단계로 나눕니다. 먼저 기존 값과 그 필드를 확인해 다음 동작을 결정하고, 그다음 조건에 따라 값을 변경합니다.

빌림 검사기는 열거형에서 `name`을 값으로 꺼내는 것을 허용하지 않습니다. 필드를 꺼낸 뒤에는 열거형 내부에 유효한 값이 남지 않기 때문입니다. 물론 `name.clone()`으로 복제해 새 변형에 넣을 수 있지만, 이 경우 불필요한 복제가 생깁니다. 이는 [빌림 검사기를 만족시키기 위한 Clone](/docs/clone-borrow-checker) 안티 패턴에 해당할 수 있습니다. 대신 가변 대여 상태에서 기존 값을 교체해 소유권을 옮길 수 있습니다.

`mem::take`는 기존 값을 꺼내 반환하고, 그 자리에 해당 타입의 기본값을 넣습니다. `String`의 기본값은 빈 `String`이며, 빈 문자열은 별도 메모리 할당이 필요하지 않습니다. 따라서 원래 `name`을 소유 값으로 얻어 새 열거형 변형 안에 넣을 수 있습니다.

`mem::replace`도 비슷하지만, 자리에 넣을 대체 값을 직접 지정할 수 있습니다. 예를 들어 `mem::take(name)`과 같은 작업은 다음처럼 작성할 수 있습니다.

```rust
mem::replace(name, String::new())
```

단, `Option`의 값을 `None`으로 바꾸면서 내부 값을 꺼내려는 경우에는 `Option::take()`가 더 짧고 관용적인 방법입니다.

## 장점

기존 값을 복제하거나 새 문자열 버퍼를 할당하지 않고 소유권을 옮길 수 있습니다. `String`처럼 기본값이 할당을 일으키지 않는 타입에서는 특히 유용합니다.

## 단점

코드가 다소 장황해질 수 있습니다. 이 패턴을 반복해서 잘못 사용하면 빌림 검사기와 씨름하게 될 수도 있습니다. 또한 컴파일러가 값을 두 번 저장하는 작업을 최적화하지 못하면, `unsafe`를 사용하는 구현에 비해 성능이 낮아질 수 있습니다. 성능이 중요하다면 실제 사용 사례를 측정해야 합니다.

`mem::take`를 사용하려면 꺼내려는 타입이 [`Default`](https://doc.rust-lang.org/stable/std/default/trait.Default.html)를 구현해야 합니다. 해당 타입이 `Default`를 구현하지 않는다면 원하는 대체 값을 넣을 수 있는 `mem::replace`를 사용할 수 있습니다.

## 논의

이 패턴은 Rust의 소유권 모델 때문에 특히 의미가 있습니다. 가비지 컬렉션 언어에서는 참조를 가져가도 가비지 컬렉터가 참조를 추적합니다. C 같은 저수준 언어에서는 포인터를 별칭으로 두고 나중에 상태를 정리할 수도 있습니다.

Rust에서는 소유 값에 소유자가 하나만 있을 수 있으므로, 값을 꺼내려면 그 자리에 다른 값을 넣어야 합니다. 마치 인디아나 존스가 유물을 가져가면서 모래주머니를 대신 올려놓는 것처럼, `mem::take`는 원래 값 대신 기본값을 넣고 소유권을 옮깁니다.

## 참고 자료

- 이 패턴은 [빌림 검사기를 만족시키기 위한 Clone](/docs/clone-borrow-checker) 안티 패턴을 특정 상황에서 피하는 방법입니다.
