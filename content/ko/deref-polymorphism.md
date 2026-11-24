# `Deref` 다형성

## 설명

`Deref` 다형성은 구조체 사이에 상속 관계가 있는 것처럼 보이게 하려고 `Deref` 트레이트를 이용해 메서드를 재사용하는 안티 패턴입니다. Rust에는 구조체 상속이 없으며, `Deref`는 포인터형 타입을 역참조해 대상 타입을 가리키도록 설계된 트레이트입니다. 래퍼를 다른 타입으로 암묵 변환하는 일반적인 수단으로 사용하는 것은 의도된 용법이 아닙니다.

## 객체 지향 상속과 비교

Java처럼 상속을 지원하는 언어에서는 다음과 같이 `Bar`가 `Foo`의 하위 타입이 되고, `Foo`가 정의한 메서드를 `Bar`에서 사용할 수 있습니다.

```java
class Foo {
    void m() { ... }
}

class Bar extends Foo {}

public static void main(String[] args) {
    Bar b = new Bar();
    b.m();
}
```

Rust에서는 합성(composition)을 이용해 `Bar` 안에 `Foo` 값을 넣을 수 있습니다.

```rust
use std::ops::Deref;

struct Foo {}

impl Foo {
    fn m(&self) {
        // ...
    }
}

struct Bar {
    f: Foo,
}

impl Deref for Bar {
    type Target = Foo;

    fn deref(&self) -> &Foo {
        &self.f
    }
}

fn main() {
    let b = Bar { f: Foo {} };
    b.m();
}
```

`Bar`에 `Deref<Target = Foo>`를 구현하면 컴파일러의 자동 역참조 및 점 표기법의 메서드 탐색을 통해 `b.m()`이 `Foo::m`을 찾을 수 있습니다. 이 문법이 상속처럼 보일 수 있지만 실제로는 `Bar`를 `Foo`로 역참조하는 동작을 선언한 것입니다.

`Bar`는 `Foo`를 필드 값으로 소유하며, 그 값은 `Bar` 안에 포함됩니다. 그러나 Rust의 기본 메모리 레이아웃은 안정적으로 고정되어 있지 않습니다. 외부 ABI와 구조체 레이아웃을 맞춰야 하는 경우에만 요구 사항에 따라 `#[repr(C)]` 등을 검토해야 하며, `repr(C)`는 C 호환 레이아웃을 위한 것이지 Java 객체의 메모리 배치와 동일하다는 보장은 아닙니다.

보일러플레이트를 줄이려면 필요한 메서드를 명시적으로 위임할 수도 있습니다.

```rust
impl Bar {
    fn m(&self) {
        self.f.m()
    }
}
```

## 장점

명시적인 위임 메서드를 여러 개 작성하는 대신 `Deref`를 한 번 구현하면, 메서드 호출을 대상 필드에 자동으로 전달할 수 있어 코드가 조금 짧아집니다.

## 단점

- **놀라움을 줍니다.** 독자는 `Bar`의 메서드가 `Deref`를 따라 `Foo`에서 온다는 점을 코드에서 바로 알아차리기 어렵습니다. 암묵적인 메서드 탐색은 공개 API를 읽고 이해하기 어렵게 만들 수 있습니다.
- **상속이나 하위 타입 관계를 만들지 않습니다.** `Bar`는 `Foo`가 아니며, `Foo`를 요구하는 함수에 `Bar`를 하위 타입처럼 전달할 수 없습니다.
- **트레이트 구현을 상속하지 않습니다.** `Foo`가 어떤 트레이트를 구현하더라도 그 트레이트가 자동으로 `Bar`에 구현되지는 않습니다. 따라서 `T: SomeTrait` 같은 제네릭 바운드에서는 `Deref`만으로 `Bar`가 `SomeTrait`을 만족하지 않습니다.
- **`self`의 의미가 다릅니다.** 대상 타입에 정의된 메서드를 자동 역참조로 호출하면 메서드의 수신자는 `Foo`입니다. 상속을 지원하는 언어에서 하위 객체를 통해 상위 메서드를 호출할 때 기대하는 수신자 타입과 다를 수 있습니다.
- **표현 가능한 관계가 제한됩니다.** `Deref`는 하나의 `Target` 타입을 지정합니다. 상속 계층, 여러 인터페이스 구현, 클래스 기반 접근 제어 같은 기능을 제공하지 않습니다. 따라서 Java 등의 상속 모델에 익숙한 사용자에게 미묘하게 다른 동작을 기대하게 만들 수 있습니다.

## 논의와 대안

모든 경우에 적용되는 하나의 대안은 없습니다. 재사용하려는 행위가 공통 계약이라면 트레이트로 표현하고, `Bar`의 공개 API 일부만 제공하려면 위임 메서드를 명시적으로 작성하는 방법이 있습니다. 보일러플레이트가 실제 문제라면 [`delegate`](https://crates.io/crates/delegate)나 [`ambassador`](https://crates.io/crates/ambassador) 같은 위임 매크로 크레이트를 검토할 수도 있습니다.

Rust는 타입 간 관계를 가능한 한 명시적으로 표현하는 것을 선호합니다. `Deref`의 자동 역참조는 포인터형 타입을 편하게 사용하도록 점 표기법과 함수 인자 등에서 암묵적으로 적용되지만, 임의의 서로 다른 타입 간 변환을 위한 기능은 아닙니다. 표준 라이브러리 문서도 대상 타입처럼 투명하게 동작하고, 역참조가 저렴하며, 사용자가 자동 변환에 놀라지 않을 때 `Deref`를 구현할 것을 권합니다.

따라서 `Deref`를 구현하기 전에 다음을 확인하세요.

- 래퍼가 대상 타입을 사실상 포인터처럼 가리키거나 빌린 뷰를 제공하나요?
- `deref` 호출은 저렴하고 예상치 못하게 실패하지 않나요?
- 대상 타입의 메서드가 래퍼 타입의 메서드와 충돌하지 않나요?
- 이 암묵적 변환을 공개 API의 일부로 장기간 유지할 의향이 있나요?

원문은 이와 비슷한 상속 기능을 Rust에 추가하려는 과거 논의를 언급합니다. 당시 제안과 RFC는 역사적 맥락으로만 참고하고, 안정 Rust에서 구조체 상속을 사용할 수 있다고 가정하지 마세요.

## 참고 자료

- [Rust Design Patterns 원문: Deref polymorphism](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html)
- [`Deref` 표준 라이브러리 문서와 구현 지침](https://doc.rust-lang.org/std/ops/trait.Deref.html)
- [The Rust Programming Language: `Deref` 트레이트와 역참조 강제 변환](https://doc.rust-lang.org/book/ch15-02-deref.html)
- [The Rust Programming Language: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [컬렉션은 스마트 포인터](/docs/collections-deref): 소유 컬렉션이 빌린 뷰를 제공하는 적절한 `Deref` 사용 사례
- [위임 매크로 `delegate`](https://crates.io/crates/delegate)
- [위임 매크로 `ambassador`](https://crates.io/crates/ambassador)
- [Rust의 코드 재사용과 상속에 관한 과거 논의](https://aturon.github.io/blog/2015/09/18/reuse/)
- [Virtual structs: 확장 열거형과 thin trait에 관한 과거 논의](https://smallcultfollowing.com/babysteps/blog/2015/10/08/virtual-structs-part-4-extended-enums-and-thin-traits/)
- [RFC 349: Virtual structs](https://github.com/rust-lang/rfcs/issues/349)
