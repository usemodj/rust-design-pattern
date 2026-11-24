# Bridge 패턴

## 의도

Bridge(브리지) 패턴은 추상화와 그 구현을 분리해 양쪽을 독립적으로 확장할 수 있게 합니다. 여기서 추상화는 클라이언트가 사용하는 상위 수준의 기능이고, 구현은 그 기능을 실제로 수행하는 방식입니다. 둘을 하나의 계층 구조로 묶는 대신 합성과 인터페이스를 통해 연결합니다.

예를 들어 도형 편집기에서 도형 종류(원, 사각형)와 렌더링 방식(래스터, SVG)이 각각 독립적으로 늘어난다고 합시다. 두 축을 상속으로 조합하면 각 도형과 렌더링 방식의 모든 조합에 별도 타입을 만들 수 있습니다. Bridge는 `Shape` 추상화와 `Renderer` 구현을 나누고, 각 도형이 렌더러를 보유하게 해 이 조합을 런타임에 구성합니다.

## 구성 요소

- **추상화(Abstraction):** 클라이언트에 기능을 제공하면서 구현 인터페이스를 보유합니다.
- **구체 추상화(Refined Abstraction):** 상위 수준 기능을 구체 타입으로 제공합니다.
- **구현(Implementor):** 여러 추상화가 사용할 구현 인터페이스를 정의합니다.
- **구체 구현(Concrete Implementor):** 구현 인터페이스를 서로 다른 방식으로 수행합니다.

## Rust에서 표현하기

Rust에서는 구조체 합성과 트레이트를 이용해 Bridge를 표현할 수 있습니다. 추상화 타입은 구현 트레이트를 구현하는 값을 필드로 보유하고, 자신의 API 호출을 그 값에 위임합니다. 실행 중 구현을 바꿀 필요가 있다면 `Box<dyn Trait>` 같은 트레이트 객체를 사용할 수 있습니다. 구현 선택이 컴파일 시점에 정해진다면 제네릭 매개변수로 동적 디스패치와 힙 할당을 피할 수 있습니다.

Adapter와 Bridge는 모두 래퍼와 합성을 사용할 수 있지만 목적이 다릅니다. Adapter는 이미 존재하는 인터페이스를 클라이언트가 기대하는 형태로 맞춥니다. Bridge는 처음부터 두 변화 축을 분리해 각각 독립적으로 확장할 수 있도록 설계합니다.

## Rust 예제: 도형과 렌더러 분리하기

`Shape`는 원이나 사각형이라는 추상화를 나타내고, `Renderer`는 도형을 출력하는 구현 인터페이스입니다. `RasterRenderer`와 `SvgRenderer`는 같은 렌더러 인터페이스를 서로 다른 형식으로 구현합니다.

`src/lib.rs`:

```rust
pub trait Renderer {
    fn render_circle(&self, x: i32, y: i32, radius: u32) -> String;
    fn render_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> String;
}

pub struct RasterRenderer;

impl Renderer for RasterRenderer {
    fn render_circle(&self, x: i32, y: i32, radius: u32) -> String {
        format!("raster circle at ({x}, {y}) radius={radius}")
    }

    fn render_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> String {
        format!("raster rectangle at ({x}, {y}) size={width}x{height}")
    }
}

pub struct SvgRenderer;

impl Renderer for SvgRenderer {
    fn render_circle(&self, x: i32, y: i32, radius: u32) -> String {
        format!(r#"<circle cx="{x}" cy="{y}" r="{radius}" />"#)
    }

    fn render_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> String {
        format!(r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" />"#)
    }
}

pub trait Shape {
    fn draw(&self) -> String;
}

pub struct Circle {
    x: i32,
    y: i32,
    radius: u32,
    renderer: Box<dyn Renderer>,
}

impl Circle {
    pub fn new(x: i32, y: i32, radius: u32, renderer: Box<dyn Renderer>) -> Self {
        Self {
            x,
            y,
            radius,
            renderer,
        }
    }
}

impl Shape for Circle {
    fn draw(&self) -> String {
        self.renderer.render_circle(self.x, self.y, self.radius)
    }
}

pub struct Rectangle {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    renderer: Box<dyn Renderer>,
}

impl Rectangle {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        renderer: Box<dyn Renderer>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            renderer,
        }
    }
}

impl Shape for Rectangle {
    fn draw(&self) -> String {
        self.renderer
            .render_rectangle(self.x, self.y, self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::{Circle, RasterRenderer, Rectangle, Shape, SvgRenderer};

    #[test]
    fn each_shape_can_use_either_renderer() {
        let raster_circle = Circle::new(1, 2, 3, Box::new(RasterRenderer));
        let svg_circle = Circle::new(1, 2, 3, Box::new(SvgRenderer));
        let raster_rectangle = Rectangle::new(4, 5, 6, 7, Box::new(RasterRenderer));
        let svg_rectangle = Rectangle::new(4, 5, 6, 7, Box::new(SvgRenderer));

        assert_eq!(raster_circle.draw(), "raster circle at (1, 2) radius=3");
        assert_eq!(svg_circle.draw(), r#"<circle cx="1" cy="2" r="3" />"#);
        assert_eq!(
            raster_rectangle.draw(),
            "raster rectangle at (4, 5) size=6x7"
        );
        assert_eq!(
            svg_rectangle.draw(),
            r#"<rect x="4" y="5" width="6" height="7" />"#
        );
    }

    #[test]
    fn client_can_use_shapes_through_the_abstraction_interface() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle::new(0, 0, 5, Box::new(RasterRenderer))),
            Box::new(Rectangle::new(2, 3, 10, 4, Box::new(SvgRenderer))),
        ];

        assert_eq!(shapes[0].draw(), "raster circle at (0, 0) radius=5");
        assert_eq!(
            shapes[1].draw(),
            r#"<rect x="2" y="3" width="10" height="4" />"#
        );
    }
}
```

`Circle`과 `Rectangle`은 구체 렌더러를 직접 참조하지 않고 `Renderer` 인터페이스에 위임합니다. 따라서 새로운 도형은 기존 렌더러를 재사용할 수 있고, 새로운 렌더러도 기존 도형을 구현 수정 없이 지원할 수 있습니다. 실제 그래픽 라이브러리에서는 결과 문자열 대신 픽셀 버퍼에 그리거나 SVG 문서를 작성하는 식으로 구현을 바꿀 수 있습니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 다음 동작을 확인합니다.

1. 원과 사각형 각각을 래스터와 SVG 렌더러에 연결할 수 있습니다.
2. 클라이언트가 구체 도형 타입 대신 `Shape` 인터페이스를 통해 여러 도형을 사용할 수 있습니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 추상화와 구현을 서로 독립적으로 확장할 수 있습니다.
- 상속 계층에서 두 변화 축의 모든 조합을 별도 타입으로 만드는 조합 폭발을 줄입니다.
- 구현을 객체로 주입할 수 있어 테스트에서 대체 구현을 사용하기 쉽습니다.
- 클라이언트가 구체 구현이 아니라 안정된 인터페이스에 의존합니다.

### 단점

- 추상화와 구현을 나누는 트레이트와 합성 구조가 추가되어 단순한 문제에는 과할 수 있습니다.
- `Box<dyn Trait>`를 사용하면 힙 할당과 동적 디스패치 비용이 발생합니다.
- 구조가 익숙하지 않은 독자에게 두 계층의 역할과 연결 관계를 설명해야 합니다.
- 두 축이 실제로 독립적으로 변경될 가능성이 없다면 단순한 단일 구조가 더 명확할 수 있습니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Bridge 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/bridge)
- [Bridge 패턴 — 위키백과](https://en.wikipedia.org/wiki/Bridge_pattern)
- [Rust Book: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust Reference: 트레이트 객체](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Adapter 패턴](/docs/adapter): 이미 존재하는 인터페이스를 맞춰 연결하는 패턴
