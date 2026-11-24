# Proxy 패턴

## 의도

Proxy(프록시) 패턴은 실제 객체와 같은 인터페이스를 제공하는 대리 객체를 두어, 실제 객체에 대한 접근을 중개하거나 제어합니다. 프록시는 요청을 실제 객체에 전달하면서 생성 시점, 접근 권한, 캐싱, 원격 호출 같은 부가 처리를 담당할 수 있습니다.

예를 들어 큰 이미지 파일을 목록 화면에 표시할 때 모든 파일을 즉시 메모리에 읽으면 초기 화면이 느려질 수 있습니다. 가상 프록시는 이미지 경로만 먼저 보관하고, 실제 표시가 요청될 때 이미지를 로드한 뒤 이후 요청에는 로드된 객체를 재사용합니다.

## 구성 요소

- **서비스 인터페이스(Service):** 실제 객체와 프록시가 함께 구현하는 인터페이스입니다.
- **실제 객체(Real Subject):** 요청을 실제로 처리하는 객체입니다.
- **프록시(Proxy):** 실제 객체를 보유하거나 생성하고, 호출을 중개합니다.
- **클라이언트(Client):** 구체적인 실제 객체 대신 서비스 인터페이스를 사용합니다.

## 프록시의 대표적인 유형

- **가상 프록시(Virtual Proxy):** 비용이 큰 객체의 생성을 실제 사용 시점까지 늦춥니다.
- **보호 프록시(Protection Proxy):** 사용자나 호출 권한에 따라 실제 객체에 대한 접근을 제한합니다.
- **원격 프록시(Remote Proxy):** 다른 프로세스나 컴퓨터에 있는 객체를 로컬 객체처럼 호출하도록 중개합니다.
- **캐싱 프록시(Caching Proxy):** 이전 호출의 결과를 저장해 반복 작업을 줄입니다.

모든 프록시는 같은 목적이 아닙니다. 실제로 필요한 중개 책임만 선택해 적용해야 합니다.

## Rust에서 표현하기

Rust에서는 서비스 트레이트를 실제 객체와 프록시가 모두 구현하도록 하고, 프록시가 실제 객체를 `Option<T>`로 보유할 수 있습니다. `None`은 아직 실제 객체가 만들어지지 않았다는 상태를 나타냅니다. 클라이언트가 트레이트에만 의존하면 실제 객체와 프록시를 교체해 사용할 수 있습니다.

## Rust 예제: 이미지 로드를 지연하는 가상 프록시

아래 예제에서 `ImageProxy`는 경로만 보관하다가 처음 `display`가 호출될 때 `RealImage`를 생성합니다. 이후 호출은 생성된 이미지를 재사용합니다. 실제 애플리케이션에서는 `RealImage::load` 안에서 파일이나 네트워크 데이터를 읽을 수 있습니다.

`src/lib.rs`:

```rust
pub trait Image {
    fn display(&mut self) -> String;
}

struct RealImage {
    path: String,
}

impl RealImage {
    fn load(path: String) -> Self {
        Self { path }
    }

    fn render(&self) -> String {
        format!("이미지 표시: {}", self.path)
    }
}

impl Image for RealImage {
    fn display(&mut self) -> String {
        self.render()
    }
}

pub struct ImageProxy {
    path: String,
    real_image: Option<RealImage>,
    load_count: usize,
}

impl ImageProxy {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            real_image: None,
            load_count: 0,
        }
    }

    pub fn load_count(&self) -> usize {
        self.load_count
    }
}

impl Image for ImageProxy {
    fn display(&mut self) -> String {
        if let Some(image) = self.real_image.as_ref() {
            return image.render();
        }

        self.load_count += 1;
        let image = RealImage::load(self.path.clone());
        let rendered = image.render();
        self.real_image = Some(image);
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::{Image, ImageProxy};

    #[test]
    fn creating_proxy_does_not_load_the_real_image() {
        let proxy = ImageProxy::new("/images/forest.png");

        assert!(proxy.real_image.is_none());
        assert_eq!(proxy.load_count(), 0);
    }

    #[test]
    fn first_display_loads_image_and_later_calls_reuse_it() {
        let mut proxy = ImageProxy::new("/images/forest.png");

        assert_eq!(proxy.display(), "이미지 표시: /images/forest.png");
        assert_eq!(proxy.load_count(), 1);
        assert_eq!(proxy.display(), "이미지 표시: /images/forest.png");
        assert_eq!(proxy.load_count(), 1);
    }
}
```

`ImageProxy`와 `RealImage`는 모두 `Image`를 구현하지만 생성 시점은 다릅니다. 프록시를 만들 때 실제 이미지 파일을 읽지 않으며, 첫 표시 요청에서 한 번 로드하고 결과를 보관합니다. 이 예제는 로드 시점을 보여 주기 위해 호출 수를 기록합니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 확인합니다.

1. 프록시를 생성해도 실제 이미지는 아직 만들어지지 않습니다.
2. 첫 표시 호출에서 이미지를 한 번 만들고, 다음 호출은 그 객체를 재사용합니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

실제 파일이나 네트워크 로드가 실패할 수 있다면 `display`의 반환 타입을 `Result<String, ImageError>`처럼 설계해 실패를 호출자에게 전달해야 합니다. 예제의 `RealImage::load`는 실제 입출력 대신 단순한 객체 생성을 사용합니다.

## 장점과 단점

### 장점

- 비싼 객체의 생성이나 자원 접근을 필요한 시점까지 늦출 수 있습니다.
- 접근 권한, 캐시, 원격 호출 등의 제어 로직을 실제 객체와 분리할 수 있습니다.
- 같은 인터페이스를 사용하므로 클라이언트의 구체 구현 의존을 줄입니다.

### 단점

- 프록시라는 간접 계층이 추가되어 호출 경로와 디버깅이 복잡해집니다.
- 실제 객체 생성이나 원격 호출이 실패할 때 오류를 전달하고 상태를 관리해야 합니다.
- 캐싱 프록시는 데이터 최신성, 무효화, 동시 접근 정책을 별도로 고려해야 합니다.
- 비용이 작은 객체에 프록시를 추가하면 얻는 이점보다 복잡성이 더 커질 수 있습니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Proxy 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/proxy)
- [Proxy 패턴 — 위키백과](https://en.wikipedia.org/wiki/Proxy_pattern)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [Rust Reference: 트레이트 객체 타입](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Decorator 패턴](/docs/decorator): 객체를 감싸 기능을 추가하는 패턴
- [Facade 패턴](/docs/facade): 복잡한 하위 시스템에 단순한 진입점을 제공하는 패턴
