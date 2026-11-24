# Factory Method 패턴

## 의도

Factory Method(팩토리 메서드) 패턴은 제품 객체를 직접 생성하는 코드와 그 제품을 사용하는 핵심 로직을 분리합니다. 생성자 역할을 하는 메서드를 공통 인터페이스에 두고, 구체 Creator가 그 메서드를 구현해 어떤 제품을 만들지 선택합니다.

제품은 공통 트레이트를 구현하므로 Creator의 핵심 로직은 구체 제품 타입을 몰라도 됩니다. 새 제품 종류가 추가되어도 기존 클라이언트 로직을 바꾸지 않고 새로운 Creator를 추가할 수 있습니다.

## 문제와 해결 방식

물류 애플리케이션이 처음에는 트럭 배송만 지원한다고 가정해 봅시다. 애플리케이션의 주요 로직이 `Truck`을 직접 생성하고 이에 의존하면, 이후 해상 운송을 추가할 때 곳곳의 생성 코드와 조건문을 수정해야 합니다. 다른 운송 수단을 추가할 때도 같은 변경을 반복하게 됩니다.

Factory Method는 클라이언트가 구체 타입의 생성자를 직접 호출하는 대신 `create_transport` 같은 생성 메서드를 호출하도록 합니다. `RoadLogistics`는 `Truck`을, `SeaLogistics`는 `Ship`을 생성할 수 있습니다. 공통 동작은 제품 트레이트에 의존하므로 어떤 구체 제품이 만들어졌는지 알 필요가 없습니다.

Factory Method에서 Creator의 주된 책임은 반드시 객체를 만드는 것만은 아닙니다. 제품을 사용하는 핵심 작업을 제공하고, 생성 메서드가 그 작업을 구체 제품에 덜 결합되게 해 줍니다. 또한 팩토리 메서드는 항상 새 값을 만들 필요는 없습니다. 조건에 따라 캐시나 객체 풀에서 기존 객체를 반환하도록 설계할 수도 있습니다.

## Rust에서 표현하기

클래스 상속을 전제로 하는 전통적인 구현과 달리, Rust에는 하위 클래스가 없습니다. 대신 제품의 공통 동작은 트레이트로, 생성 주체는 생성 메서드를 구현하는 타입으로 표현할 수 있습니다.

다음 예제는 런타임에 서로 다른 제품을 반환할 수 있도록 `Box<dyn Transport>`를 사용합니다. Creator 트레이트의 기본 `plan_delivery` 메서드는 생성된 제품의 동작을 호출하는 공통 로직을 제공합니다.

`src/lib.rs`:

```rust
pub trait Transport {
    fn deliver(&self) -> &'static str;
}

pub struct Truck;
pub struct Ship;

impl Transport for Truck {
    fn deliver(&self) -> &'static str {
        "육로로 화물을 배송합니다."
    }
}

impl Transport for Ship {
    fn deliver(&self) -> &'static str {
        "해상으로 화물을 배송합니다."
    }
}

pub trait Logistics {
    fn create_transport(&self) -> Box<dyn Transport>;

    fn plan_delivery(&self) -> &'static str {
        self.create_transport().deliver()
    }
}

pub struct RoadLogistics;
pub struct SeaLogistics;

impl Logistics for RoadLogistics {
    fn create_transport(&self) -> Box<dyn Transport> {
        Box::new(Truck)
    }
}

impl Logistics for SeaLogistics {
    fn create_transport(&self) -> Box<dyn Transport> {
        Box::new(Ship)
    }
}

#[cfg(test)]
mod tests {
    use super::{Logistics, RoadLogistics, SeaLogistics};

    #[test]
    fn road_creator_produces_a_truck_transport() {
        let logistics = RoadLogistics;

        assert_eq!(
            logistics.create_transport().deliver(),
            "육로로 화물을 배송합니다."
        );
    }

    #[test]
    fn sea_creator_produces_a_ship_transport() {
        let logistics = SeaLogistics;

        assert_eq!(
            logistics.create_transport().deliver(),
            "해상으로 화물을 배송합니다."
        );
    }

    #[test]
    fn creator_workflow_uses_its_factory_product() {
        let road = RoadLogistics;
        let sea = SeaLogistics;

        assert_eq!(road.plan_delivery(), "육로로 화물을 배송합니다.");
        assert_eq!(sea.plan_delivery(), "해상으로 화물을 배송합니다.");
    }
}
```

`RoadLogistics`와 `SeaLogistics`는 같은 `Logistics` 인터페이스를 따르지만 서로 다른 구체 제품을 만듭니다. 호출자는 제품에 필요한 `deliver` 동작만 사용하고, 트럭인지 선박인지는 Creator가 결정합니다. 테스트는 각 Creator가 올바른 제품을 반환하는지와 Creator의 공통 흐름이 생성된 제품을 사용하는지 확인합니다.

사용 예:

```rust,ignore
use factory_method_example::{Logistics, RoadLogistics, SeaLogistics};

fn main() {
    let road = RoadLogistics;
    let sea = SeaLogistics;

    println!("{}", road.plan_delivery());
    println!("{}", sea.plan_delivery());
}
```

패키지 루트에서 다음 명령으로 단위 테스트를 실행합니다.

```console
cargo test
```

## 동적 디스패치와 정적 디스패치

예제의 `Box<dyn Transport>`는 제품 종류를 런타임에 바꿀 수 있도록 트레이트 객체를 사용합니다. 그 결과 동적 디스패치가 일어나고 제품을 힙에 할당합니다. 런타임에 서로 다른 Creator의 제품을 같은 타입으로 다뤄야 할 때 유용하지만, 힙 할당과 간접 호출 비용이 생깁니다.

제품 종류가 컴파일 시점에 정해져 있고 동적 디스패치가 필요 없다면 Creator 트레이트의 연관 타입으로 제품을 나타낼 수도 있습니다.

```rust,ignore
trait Logistics {
    type Product: Transport;

    fn create_transport(&self) -> Self::Product;
}
```

이 방식은 정적 디스패치를 사용하고 별도 `Box` 할당이 필요 없지만, 서로 다른 Creator를 하나의 동일한 구체 타입으로 직접 보관하려면 제네릭이나 열거형 같은 추가 구조가 필요합니다. 런타임에 제품 종류를 선택해야 하는지, 정적 타입 정보와 성능이 더 중요한지에 따라 방식을 선택하세요.

## Factory Method와 단순 팩토리 비교

단순 팩토리는 보통 입력값을 `match`나 조건문으로 분기해 제품을 반환하는 함수 하나입니다. 제품 선택 기준이 한 곳에 모이는 것은 유용하지만, 그 자체로는 Creator의 공통 알고리즘을 다른 생성 구현에 맡기는 Factory Method와 같지 않습니다.

Factory Method는 Creator 인터페이스의 생성 메서드와 제품 인터페이스를 중심으로 확장됩니다. Rust에서는 트레이트와 구체 Creator 타입으로 이 관계를 표현할 수 있습니다. 다만 제품 종류가 적고 생성 조건이 단순하다면 `match`를 사용하는 작은 함수가 더 명확할 수 있습니다. 패턴을 적용하기 전에 실제로 구체 생성과 핵심 로직의 결합이 문제가 되는지 확인하세요.

## 장점과 단점

### 장점

- 제품 생성과 제품을 사용하는 핵심 로직을 분리합니다.
- 클라이언트는 구체 제품이 아니라 제품 트레이트에 의존할 수 있습니다.
- 새 제품이 추가되어도 기존 Creator의 공통 동작을 유지하면서 새로운 Creator를 추가할 수 있습니다.
- 테스트에서 특정 Creator나 제품을 대체하기 쉬워집니다.

### 단점

- 제품 인터페이스, Creator 트레이트와 구체 Creator 타입이 추가되어 설계가 복잡해집니다.
- 제품을 런타임에 다형적으로 반환하려면 `Box<dyn Trait>`의 힙 할당과 동적 디스패치가 필요할 수 있습니다.
- 단순한 생성 분기에는 과도한 추상화가 될 수 있습니다.
- 관련 제품군을 여러 개 함께 생성해야 한다면 Factory Method만으로는 부족하고 Abstract Factory가 더 적절할 수 있습니다.

## 참고 자료

- [생성 패턴 개요](/docs/creational-patterns)
- [Factory Method 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/factory-method)
- [Factory Method 패턴 — 위키백과](https://en.wikipedia.org/wiki/Factory_method_pattern)
- [Rust의 트레이트 문서: 공통 동작 정의](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust Reference: 트레이트 객체와 동적 디스패치](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Builder 패턴](/docs/builder): 복잡한 값의 단계별 구성이 목적일 때
- [Singleton 패턴](/docs/singleton): 인스턴스를 하나로 제한하는 것이 목적일 때
