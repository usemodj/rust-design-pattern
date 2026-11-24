# Decorator 패턴

## 의도

Decorator(데코레이터) 패턴은 객체를 같은 인터페이스를 구현하는 래퍼로 감싸 기존 동작에 책임을 동적으로 추가합니다. 여러 기능을 조합하기 위해 기능 조합마다 별도의 하위 타입을 만들 필요가 없습니다. 런타임에 데코레이터를 겹쳐 필요한 조합을 구성할 수 있습니다.

예를 들어 기본 에스프레소에 우유나 휘핑크림을 추가할 수 있습니다. 각 데코레이터는 `Beverage` 인터페이스를 그대로 제공하면서 내부 음료에 비용과 설명을 더합니다.

## 구성 요소

- **컴포넌트(Component):** 기본 객체와 데코레이터가 공유하는 인터페이스를 정의합니다.
- **구체 컴포넌트(Concrete Component):** 데코레이터 없이도 사용할 수 있는 기본 객체입니다.
- **데코레이터(Decorator):** 컴포넌트를 보유하고 같은 인터페이스를 구현하며, 호출을 위임하거나 결과를 확장합니다.
- **구체 데코레이터(Concrete Decorator):** 구체적인 추가 책임을 구현합니다.
- **클라이언트(Client):** 구체 타입 대신 컴포넌트 인터페이스를 사용합니다.

## Rust에서 표현하기

Rust에서는 데코레이터가 내부 컴포넌트를 `Box<dyn Trait>`으로 소유하고, 같은 트레이트를 구현하도록 구성할 수 있습니다. 생성 시 어떤 래퍼를 몇 겹으로 둘지 정해 기능을 조합합니다.

예제는 런타임에 데코레이터를 조합할 수 있도록 트레이트 객체를 사용합니다. 조합이 컴파일 시점에 정해진다면 제네릭 래퍼를 이용해 동적 디스패치와 힙 할당을 피할 수도 있습니다.

Adapter와 달리 Decorator는 보통 인터페이스를 바꾸지 않습니다. 클라이언트가 보던 인터페이스를 그대로 유지하면서 동작을 추가합니다. Composite도 같은 인터페이스를 재귀적 구조에 사용하지만, 그 목적은 단일 객체와 트리의 묶음을 동일하게 다루는 것입니다.

## Rust 예제: 음료에 추가 옵션 조합하기

`src/lib.rs`:

```rust
pub trait Beverage {
    fn description(&self) -> String;
    fn cost_cents(&self) -> u32;
}

pub struct Espresso;

impl Beverage for Espresso {
    fn description(&self) -> String {
        String::from("에스프레소")
    }

    fn cost_cents(&self) -> u32 {
        200
    }
}

pub struct Milk {
    beverage: Box<dyn Beverage>,
}

impl Milk {
    pub fn new(beverage: impl Beverage + 'static) -> Self {
        Self {
            beverage: Box::new(beverage),
        }
    }
}

impl Beverage for Milk {
    fn description(&self) -> String {
        format!("{} + 우유", self.beverage.description())
    }

    fn cost_cents(&self) -> u32 {
        self.beverage.cost_cents() + 50
    }
}

pub struct WhippedCream {
    beverage: Box<dyn Beverage>,
}

impl WhippedCream {
    pub fn new(beverage: impl Beverage + 'static) -> Self {
        Self {
            beverage: Box::new(beverage),
        }
    }
}

impl Beverage for WhippedCream {
    fn description(&self) -> String {
        format!("{} + 휘핑크림", self.beverage.description())
    }

    fn cost_cents(&self) -> u32 {
        self.beverage.cost_cents() + 70
    }
}

#[cfg(test)]
mod tests {
    use super::{Beverage, Espresso, Milk, WhippedCream};

    #[test]
    fn concrete_component_has_its_base_behavior() {
        let espresso = Espresso;

        assert_eq!(espresso.description(), "에스프레소");
        assert_eq!(espresso.cost_cents(), 200);
    }

    #[test]
    fn one_decorator_adds_its_behavior() {
        let drink = Milk::new(Espresso);

        assert_eq!(drink.description(), "에스프레소 + 우유");
        assert_eq!(drink.cost_cents(), 250);
    }

    #[test]
    fn decorators_can_be_stacked_at_runtime() {
        let drink = WhippedCream::new(Milk::new(Espresso));

        assert_eq!(drink.description(), "에스프레소 + 우유 + 휘핑크림");
        assert_eq!(drink.cost_cents(), 320);
    }
}
```

각 데코레이터는 내부 `Beverage`에 먼저 작업을 위임한 다음 자신의 설명과 비용을 추가합니다. `WhippedCream`은 내부 값의 구체 타입이 `Espresso`인지 `Milk`인지 알 필요가 없습니다. 같은 인터페이스를 구현하는 다른 데코레이터도 같은 방식으로 쌓을 수 있습니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 확인합니다.

1. 기본 음료는 추가 옵션이 없는 설명과 비용을 제공합니다.
2. 우유 데코레이터는 음료 인터페이스를 유지하면서 설명과 비용을 확장합니다.
3. 우유와 휘핑크림 데코레이터를 겹쳐 각 책임이 누적됩니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 상속 없이 객체의 기능을 조합해 확장할 수 있습니다.
- 사용하지 않는 기능 조합까지 모두 별도 타입으로 만들 필요가 없습니다.
- 데코레이터를 쌓는 순서에 따라 서로 다른 조합을 만들 수 있습니다.
- 기본 객체와 데코레이터가 같은 인터페이스를 제공하므로 클라이언트 변경을 줄입니다.

### 단점

- 래퍼가 여러 겹 쌓이면 실제 동작의 흐름을 추적하기 어려워질 수 있습니다.
- 각 래퍼가 추가되어 타입과 런타임 객체 구성이 복잡해집니다.
- `Box<dyn Trait>`을 쓰면 힙 할당과 동적 디스패치 비용이 발생합니다.
- 데코레이터의 순서가 결과를 바꿀 수 있으므로 조합 순서를 명확히 해야 합니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Decorator 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/decorator)
- [Decorator 패턴 — 위키백과](https://en.wikipedia.org/wiki/Decorator_pattern)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [Rust Reference: 트레이트 객체 타입](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Adapter 패턴](/docs/adapter): 기존 인터페이스를 기대되는 API에 맞추는 패턴
- [Composite 패턴](/docs/composite): 객체와 객체의 트리를 같은 인터페이스로 다루는 패턴
