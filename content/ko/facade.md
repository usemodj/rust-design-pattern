# Facade 패턴

## 의도

Facade(퍼사드) 패턴은 여러 하위 시스템의 복잡한 API를 감싸 클라이언트가 자주 사용하는 작업을 위한 단순한 진입점을 제공합니다. 퍼사드는 하위 구성 요소를 대체하지 않고, 필요한 호출을 정해진 순서로 조정합니다.

예를 들어 홈 시어터에서 영화를 재생하려면 프로젝터 전원과 입력을 설정하고, 오디오 시스템을 켜고 볼륨을 조절한 다음 플레이어를 시작해야 할 수 있습니다. 클라이언트가 이 순서를 직접 관리하는 대신 `watch_movie` 같은 퍼사드 메서드 하나를 호출하도록 만들 수 있습니다.

## 구성 요소

- **퍼사드(Facade):** 클라이언트에 단순화된 상위 수준의 API를 제공합니다.
- **하위 시스템(Subsystems):** 실제 작업을 수행하는 여러 구성 요소입니다. 퍼사드 없이도 직접 사용할 수 있습니다.
- **클라이언트(Client):** 세부 작업 순서가 필요하지 않을 때 퍼사드만 호출합니다.

## Rust에서 표현하기

Rust에서는 퍼사드 구조체가 여러 하위 시스템을 소유하거나 빌리고, 여러 메서드 호출을 하나의 공개 메서드로 조정하도록 구성할 수 있습니다. 하위 시스템을 모듈 내부에 숨기면 클라이언트가 퍼사드 API를 통해서만 일반적인 작업을 수행하도록 제한할 수 있습니다.

Facade는 Adapter와 목적이 다릅니다. Adapter는 기존 인터페이스를 다른 인터페이스에 맞추고, Facade는 여러 기능을 갖춘 하위 시스템 위에 더 간단한 인터페이스를 추가합니다. 퍼사드는 기존의 모든 API를 없애거나 접근을 강제로 막는 패턴은 아닙니다.

## Rust 예제: 홈 시어터 시작과 종료

아래 예제의 퍼사드는 프로젝터, 오디오 시스템, 미디어 플레이어의 상세한 호출 순서를 감춥니다. 클라이언트는 영화를 시작하거나 종료하는 상위 수준 메서드만 호출합니다.

`src/lib.rs`:

```rust
#[derive(Default)]
struct Projector {
    powered_on: bool,
    input: Option<&'static str>,
}

impl Projector {
    fn power_on(&mut self) {
        self.powered_on = true;
    }

    fn set_input(&mut self, input: &'static str) {
        self.input = Some(input);
    }

    fn power_off(&mut self) {
        self.powered_on = false;
        self.input = None;
    }
}

#[derive(Default)]
struct SoundSystem {
    powered_on: bool,
    volume: u8,
}

impl SoundSystem {
    fn power_on(&mut self) {
        self.powered_on = true;
    }

    fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    fn power_off(&mut self) {
        self.powered_on = false;
        self.volume = 0;
    }
}

#[derive(Default)]
struct MediaPlayer {
    current_movie: Option<String>,
}

impl MediaPlayer {
    fn play(&mut self, title: &str) {
        self.current_movie = Some(title.to_owned());
    }

    fn stop(&mut self) {
        self.current_movie = None;
    }
}

#[derive(Default)]
pub struct HomeTheaterFacade {
    projector: Projector,
    sound: SoundSystem,
    player: MediaPlayer,
}

impl HomeTheaterFacade {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn watch_movie(&mut self, title: &str) {
        self.projector.power_on();
        self.projector.set_input("media player");
        self.sound.power_on();
        self.sound.set_volume(5);
        self.player.play(title);
    }

    pub fn end_movie(&mut self) {
        self.player.stop();
        self.sound.power_off();
        self.projector.power_off();
    }
}

#[cfg(test)]
mod tests {
    use super::HomeTheaterFacade;

    #[test]
    fn watch_movie_coordinates_all_subsystems() {
        let mut theater = HomeTheaterFacade::new();

        theater.watch_movie("Rust의 모험");

        assert!(theater.projector.powered_on);
        assert_eq!(theater.projector.input, Some("media player"));
        assert!(theater.sound.powered_on);
        assert_eq!(theater.sound.volume, 5);
        assert_eq!(
            theater.player.current_movie.as_deref(),
            Some("Rust의 모험")
        );
    }

    #[test]
    fn ending_movie_shuts_down_the_subsystems() {
        let mut theater = HomeTheaterFacade::new();
        theater.watch_movie("Rust의 모험");

        theater.end_movie();

        assert!(!theater.projector.powered_on);
        assert_eq!(theater.projector.input, None);
        assert!(!theater.sound.powered_on);
        assert_eq!(theater.sound.volume, 0);
        assert_eq!(theater.player.current_movie, None);
    }
}
```

`watch_movie`는 여러 하위 시스템의 설정 순서를 퍼사드 안에 모읍니다. 이 예제는 상태 전환을 단순화해 보여 줍니다. 실제 장치나 네트워크 서비스를 조정한다면 단계별 실패를 표현하는 `Result`를 사용하고, 중간 단계가 실패했을 때 상태를 복구하거나 재시도할 정책도 정해야 합니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 확인합니다.

1. 영화 시작 한 번으로 프로젝터 입력, 오디오 전원과 볼륨, 플레이어 상태가 함께 설정됩니다.
2. 종료 호출이 플레이어를 멈추고 나머지 하위 시스템을 종료 상태로 되돌립니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 자주 사용하는 작업을 간단하고 일관된 공개 인터페이스로 제공합니다.
- 클라이언트와 여러 하위 시스템 사이의 직접적인 결합을 줄입니다.
- 하위 시스템 호출 순서와 조정 로직을 한 곳에 모을 수 있습니다.

### 단점

- 퍼사드가 너무 많은 책임을 맡으면 거대한 조정 객체가 될 수 있습니다.
- 퍼사드 API만 사용하면 클라이언트가 하위 시스템의 세부 기능에 직접 접근하기 어려워질 수 있습니다.
- 하위 단계에서 오류가 발생하면 부분적으로 실행된 상태를 어떻게 처리할지 별도로 설계해야 합니다.
- 하위 시스템이 이미 단순하다면 퍼사드가 불필요한 간접 계층이 될 수 있습니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Facade 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/facade)
- [Facade 패턴 — 위키백과](https://en.wikipedia.org/wiki/Facade_pattern)
- [Rust Book: 모듈로 항목을 경로와 가시성으로 제어하기](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [Adapter 패턴](/docs/adapter): 호환되지 않는 인터페이스를 연결하는 패턴
- [Composite 패턴](/docs/composite): 객체와 객체의 트리를 동일한 인터페이스로 다루는 패턴
