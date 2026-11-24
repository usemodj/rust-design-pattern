# Prototype 패턴

## 의도

Prototype(프로토타입) 패턴은 이미 만들어진 원형(prototype) 객체를 복제해 새 객체를 생성합니다. 생성할 구체 타입을 클라이언트가 미리 알 필요가 없으며, 원형의 현재 설정이나 상태를 복사해 비슷한 객체를 빠르게 만들 수 있습니다.

예를 들어 그래픽 편집기에서 자주 사용하는 도형 템플릿을 등록해 두면, 사용자는 템플릿 이름을 선택해 새 도형을 복제하고 위치나 세부 설정을 변경할 수 있습니다. 템플릿의 실제 타입이 원이나 사각형 중 무엇인지는 레지스트리와 복제 구현이 처리합니다.

## 적용하기 좋은 상황

- 생성할 구체 타입이 런타임에 선택되며, 클라이언트에서 구체 타입별 생성 코드를 반복하고 싶지 않을 때
- 객체를 만드는 과정이 비용이 크거나 복잡해 기존 객체를 복제하는 편이 효율적일 때
- 기본 설정이 적용된 객체를 원형으로 보관하고, 이를 복제해 새 설정을 적용하고 싶을 때
- 클래스마다 별도의 생성 분기를 추가하지 않고 원형 레지스트리로 생성 대상을 확장하고 싶을 때

단순한 값을 만들기 위해 매번 복제 추상화를 도입할 필요는 없습니다. `Clone`, 생성 함수, Builder 또는 Factory가 더 단순하고 명확한 해법이라면 그 방식을 선택하세요.

## Rust에서 표현하기

Rust의 `Clone` 트레이트는 값을 복제하는 표준 방법입니다. 구체 타입을 알고 있다면 `#[derive(Clone)]`로 편리하게 구현할 수 있습니다. 그러나 `Clone::clone`은 반환 타입이 `Self`이므로 `dyn Clone` 트레이트 객체로 직접 사용할 수 없습니다.

런타임에 다른 구체 타입의 원형을 보관하고 복제해야 한다면, 아래 예제처럼 객체 안전한 `clone_box` 메서드를 공통 트레이트에 두고 `Box<dyn Shape>`의 `Clone`을 구현할 수 있습니다. 레지스트리는 키에 연결된 원형을 보관하고, `create`를 호출할 때 해당 객체를 복제해 반환합니다.

## Rust 예제: 도형 프로토타입 레지스트리

`src/lib.rs`:

```rust
use std::collections::HashMap;

pub trait Shape {
    fn kind(&self) -> &'static str;
    fn description(&self) -> String;
    fn position(&self) -> (i32, i32);
    fn move_to(&mut self, x: i32, y: i32);
    fn clone_box(&self) -> Box<dyn Shape>;
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct Circle {
    x: i32,
    y: i32,
    radius: u32,
    color: String,
}

impl Shape for Circle {
    fn kind(&self) -> &'static str {
        "원"
    }

    fn description(&self) -> String {
        format!("{} 원, 반지름 {}", self.color, self.radius)
    }

    fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct Rectangle {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: String,
}

impl Shape for Rectangle {
    fn kind(&self) -> &'static str {
        "사각형"
    }

    fn description(&self) -> String {
        format!(
            "{} 사각형, 너비 {}, 높이 {}",
            self.color, self.width, self.height
        )
    }

    fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

#[derive(Default)]
pub struct ShapeRegistry {
    prototypes: HashMap<String, Box<dyn Shape>>,
}

impl ShapeRegistry {
    pub fn register(&mut self, name: impl Into<String>, prototype: Box<dyn Shape>) {
        let _replaced = self.prototypes.insert(name.into(), prototype);
    }

    pub fn create(&self, name: &str) -> Option<Box<dyn Shape>> {
        self.prototypes.get(name).cloned()
    }
}

pub fn standard_shapes() -> ShapeRegistry {
    let mut registry = ShapeRegistry::default();

    registry.register(
        "red-circle",
        Box::new(Circle {
            x: 0,
            y: 0,
            radius: 12,
            color: String::from("빨간색"),
        }),
    );
    registry.register(
        "blue-rectangle",
        Box::new(Rectangle {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
            color: String::from("파란색"),
        }),
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::standard_shapes;

    #[test]
    fn registry_creates_a_copy_of_the_selected_prototype() {
        let registry = standard_shapes();
        let circle = registry.create("red-circle").unwrap();

        assert_eq!(circle.kind(), "원");
        assert_eq!(circle.description(), "빨간색 원, 반지름 12");
        assert_eq!(circle.position(), (0, 0));
    }

    #[test]
    fn modifying_a_copy_does_not_change_the_registered_prototype() {
        let registry = standard_shapes();
        let mut copy = registry.create("red-circle").unwrap();

        copy.move_to(15, 25);

        assert_eq!(copy.position(), (15, 25));
        assert_eq!(registry.create("red-circle").unwrap().position(), (0, 0));
    }

    #[test]
    fn registry_supports_multiple_concrete_prototype_types() {
        let registry = standard_shapes();
        let rectangle = registry.create("blue-rectangle").unwrap();

        assert_eq!(rectangle.kind(), "사각형");
        assert_eq!(rectangle.description(), "파란색 사각형, 너비 40, 높이 20");
    }

    #[test]
    fn missing_prototype_returns_none() {
        let registry = standard_shapes();

        assert!(registry.create("triangle").is_none());
    }
}
```

`ShapeRegistry`는 원의 구체 타입이나 사각형의 구체 타입을 외부에 노출하지 않고 `Box<dyn Shape>`로 보관합니다. 새 도형 타입을 추가할 때 `Shape`와 `clone_box`를 구현하고 레지스트리에 등록하면, 클라이언트의 생성 코드는 바꾸지 않아도 됩니다. 알 수 없는 이름은 `None`으로 반환하므로 호출자가 누락된 템플릿을 명시적으로 처리할 수 있습니다.

여기서는 도형의 `String` 설정을 `Clone`하기 때문에 복제된 도형이 독립된 문자열 소유권을 가집니다. 하지만 `Clone`이 항상 깊은 복사를 뜻하는 것은 아닙니다. 예를 들어 `Arc<T>`를 복제하면 내부 데이터는 공유됩니다. 복제 후 원형과 사본이 어떤 데이터를 공유해야 하는지 타입별로 정해야 합니다.

## 테스트 코드

위 `src/lib.rs`에는 다음 동작을 확인하는 단위 테스트가 포함되어 있습니다.

1. 레지스트리에서 키로 원형을 선택해 해당 도형의 복사본을 만듭니다.
2. 복사본을 수정해도 레지스트리에 보관된 원형의 위치는 바뀌지 않습니다.
3. 서로 다른 구체 도형 타입을 같은 `Shape` 인터페이스로 보관하고 생성할 수 있습니다.
4. 등록되지 않은 키를 요청하면 `None`을 반환합니다.

프로젝트 루트에서 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 클라이언트가 복제할 구체 타입을 몰라도 기존 객체를 바탕으로 새 객체를 만들 수 있습니다.
- 미리 구성한 템플릿을 복제하므로, 복잡한 초기화 로직이나 설정을 재사용할 수 있습니다.
- 레지스트리를 통해 새로운 프로토타입을 등록할 수 있어 생성 대상 확장이 유연합니다.

### 단점

- 각 프로토타입의 복제 의미를 정의해야 합니다. 내부 가변 상태나 공유 소유권이 있으면 의도하지 않은 상태 공유가 발생할 수 있습니다.
- `Box<dyn Trait>` 복제를 위해 `clone_box` 같은 보조 인터페이스가 필요합니다.
- 프로토타입의 구조가 깊거나 큰 자원을 소유하면 복제 비용이 오히려 커질 수 있습니다.
- 단순한 데이터 타입에서는 `Clone`이나 생성 함수만 사용하는 편이 더 읽기 쉽습니다.

## 참고 자료

- [생성 패턴 개요](/docs/creational-patterns)
- [Prototype 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/prototype)
- [Prototype 패턴 — 위키백과](https://en.wikipedia.org/wiki/Prototype_pattern)
- [Rust 표준 라이브러리: `Clone` 트레이트](https://doc.rust-lang.org/std/clone/trait.Clone.html)
- [Rust Book: 공유 상태와 `Clone`](https://doc.rust-lang.org/book/ch15-04-rc.html)
- [Abstract Factory 패턴](/docs/abstract-factory): 관련 제품군을 함께 생성하는 패턴
- [Factory Method 패턴](/docs/factory-method): 구체 Creator에 제품 생성을 위임하는 패턴
