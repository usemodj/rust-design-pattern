# Singleton 패턴

## 의도

Singleton(싱글턴) 패턴은 특정 타입의 인스턴스를 하나로 제한하고, 그 인스턴스에 접근하는 공통 경로를 제공하는 생성 패턴입니다. Rust에서는 `static` 항목과 표준 라이브러리의 초기화 동기화 타입을 조합해 이 요구를 표현할 수 있습니다.

상태가 불변인지, 여러 스레드에서 변경해야 하는지에 따라 구현을 나누는 것이 중요합니다.

- **불변 데이터만 공유:** `OnceLock<T>`로 한 번 초기화하고 공유 참조로 읽습니다. 변경을 막는 `Mutex`는 필요하지 않습니다.
- **변경 가능한 데이터를 여러 스레드가 공유:** `OnceLock<Mutex<T>>`로 인스턴스를 한 번 초기화하고, `Mutex`로 상태 접근을 동기화합니다.

두 예제는 선택 가능한 별도 구현입니다. 각 예제의 자료형과 `static`을 복사해 사용할 수 있습니다.

## 불변 데이터만 있을 때: `OnceLock` 단독 사용

읽기 전용 설정처럼 초기화 후 바뀌지 않는 값은 `OnceLock<T>`만으로 충분합니다. `OnceLock`은 스레드 안전한 초기화 셀이므로 여러 스레드가 동시에 처음 접근해도 하나의 초기화된 값이 공유됩니다. 값에 대한 접근은 `&'static T`로 제공되며, 별도 잠금 없이 읽을 수 있습니다.

`src/lib.rs`:

```rust
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq)]
pub struct AppConfig {
    service_name: &'static str,
    max_connections: usize,
}

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

impl AppConfig {
    pub fn instance() -> &'static Self {
        APP_CONFIG.get_or_init(|| Self {
            service_name: "pattern-service",
            max_connections: 32,
        })
    }

    pub fn service_name(&self) -> &'static str {
        self.service_name
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }
}
```

필드가 비공개이고 공개 생성 함수가 없으므로 외부 호출자는 별도의 `AppConfig` 값을 만들 수 없습니다. `instance()`는 항상 같은 전역 값의 공유 참조를 반환합니다. 이 설정은 불변 데이터이므로 `Mutex`로 감싸면 읽기마다 잠금을 획득하는 불필요한 복잡성만 생깁니다.

사용 예:

```rust,ignore
use singleton_example::AppConfig;

fn main() {
    let config = AppConfig::instance();
    println!(
        "{} listens with a connection limit of {}",
        config.service_name(),
        config.max_connections()
    );
}
```

`singleton_example`은 패키지 이름을 가정한 예시입니다. 실제 프로젝트에서는 크레이트 이름으로 바꾸세요.

### 불변 Singleton 테스트

전역 Singleton은 테스트 간 공유되고 초기화 전 상태로 되돌릴 수 없습니다. 테스트마다 새 인스턴스가 만들어진다고 가정하지 말고, 반복 호출과 여러 스레드 호출이 동일한 주소를 반환하는지 확인합니다.

```rust
#[cfg(test)]
mod tests {
    use super::AppConfig;
    use std::thread;

    #[test]
    fn repeated_calls_return_the_same_instance() {
        let first = AppConfig::instance();
        let second = AppConfig::instance();

        assert!(std::ptr::eq(first, second));
        assert_eq!(first.service_name(), "pattern-service");
        assert_eq!(first.max_connections(), 32);
    }

    #[test]
    fn concurrent_calls_return_the_same_instance() {
        let handles: Vec<_> = (0..8)
            .map(|_| thread::spawn(AppConfig::instance))
            .collect();

        let instances: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().expect("singleton thread panicked"))
            .collect();

        let first = instances[0];
        assert!(instances
            .iter()
            .all(|instance| std::ptr::eq(first, *instance)));
    }
}
```

## 변경 가능한 공유 상태: `OnceLock`과 `Mutex` 함께 사용

카운터처럼 Singleton 인스턴스를 여러 스레드에서 변경해야 한다면 `OnceLock<Mutex<T>>`를 사용할 수 있습니다.

- `OnceLock`은 `Mutex<T>` 인스턴스를 한 번 초기화하고 공유합니다.
- `Mutex`는 한 번에 한 스레드만 내부의 가변 상태를 변경하도록 합니다.
- `lock()`은 잠금 가드를 반환하며, 가드가 범위를 벗어나면 잠금이 해제됩니다.

`src/lib.rs`의 대안 구현:

```rust
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
pub struct Metrics {
    request_count: usize,
}

static METRICS: OnceLock<Mutex<Metrics>> = OnceLock::new();

impl Metrics {
    fn shared() -> &'static Mutex<Self> {
        METRICS.get_or_init(|| Mutex::new(Self::default()))
    }

    pub fn record_request() {
        let mut metrics = Self::shared()
            .lock()
            .expect("metrics mutex poisoned");
        metrics.request_count += 1;
    }

    pub fn request_count() -> usize {
        Self::shared()
            .lock()
            .expect("metrics mutex poisoned")
            .request_count
    }
}
```

호출자는 직접 `Metrics`를 생성하거나 필드를 변경하지 않고, 공개 메서드로 상태를 읽고 갱신합니다. 예제는 Mutex가 패닉으로 poison된 경우 `expect`로 오류를 명시적으로 처리합니다. 애플리케이션의 복구 정책이 필요하다면 `PoisonError`를 반환하거나 내부 값을 복구하는 방식을 별도로 설계할 수 있습니다.

### 동시 갱신 테스트

테스트는 여러 스레드가 동시에 카운터를 증가시켜도 갱신이 유실되지 않는지 확인합니다. 전역 상태는 테스트 간 유지되므로 처음 값이 반드시 0이라고 가정하지 않고 테스트 시작 시점의 값을 기준으로 결과를 검사합니다.

```rust
#[cfg(test)]
mod tests {
    use super::Metrics;
    use std::thread;

    #[test]
    fn concurrent_updates_share_one_instance_and_preserve_counts() {
        const THREADS: usize = 8;
        const UPDATES_PER_THREAD: usize = 1_000;

        let initial_count = Metrics::request_count();
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                thread::spawn(|| {
                    for _ in 0..UPDATES_PER_THREAD {
                        Metrics::record_request();
                    }
                    Metrics::shared()
                })
            })
            .collect();

        let instances: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().expect("metrics thread panicked"))
            .collect();

        let first = instances[0];
        assert!(instances
            .iter()
            .all(|instance| std::ptr::eq(first, *instance)));
        assert_eq!(
            Metrics::request_count(),
            initial_count + THREADS * UPDATES_PER_THREAD
        );
    }
}
```

`Metrics::shared`는 모듈 내부 테스트에서 동일한 `Mutex` 인스턴스인지 확인하기 위해 사용합니다. 실제 외부 API에서는 이 핸들을 공개하지 않고 `record_request`와 `request_count`만 노출합니다.

## 테스트 실행

각 구현을 별도의 예제로 사용하거나, 두 타입을 하나의 크레이트에 함께 넣고 각각의 테스트 모듈을 추가한 뒤 다음을 실행합니다.

```console
cargo test
```

## 선택 기준과 주의점

- 읽기 전용 설정은 `OnceLock<T>`로 충분합니다. 동기화된 초기화 이후에는 공유 참조로 접근하므로 `Mutex`가 필요하지 않습니다.
- 변경 가능한 전역 상태에 `Mutex`를 사용하면 동시 변경은 보호되지만, 전역 의존성과 잠금 경합이 생깁니다. 잠금을 잡은 채 오래 작업하거나 잠금 안에서 다시 같은 잠금을 획득하지 않도록 주의하세요.
- `OnceLock`은 프로세스 전체의 모든 실행 파일·동적 라이브러리에서 무조건 하나의 인스턴스를 보장하는 범용 서비스 레지스트리는 아닙니다. 인스턴스의 범위는 해당 정적 항목이 포함된 프로그램 실행 이미지에 한정해 이해하는 것이 좋습니다.
- 전역 Singleton은 테스트 격리와 대체 구현 주입을 어렵게 하고 컴포넌트의 의존성을 숨길 수 있습니다. 설정이나 서비스가 달라질 수 있다면 인스턴스를 생성해 필요한 곳에 전달하는 의존성 주입을 우선 고려하세요.
- Singleton 패턴의 의도는 인스턴스 유일성과 공통 접근 경로입니다. 단지 전역 변수에 접근하기 쉽다는 이유만으로 도입하지 마세요.

## 참고 자료

- [생성 패턴 개요](/docs/creational-patterns)
- [Singleton 패턴 — 위키백과](https://en.wikipedia.org/wiki/Singleton_pattern)
- [Singleton 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/singleton)
- [`std::sync::OnceLock` 문서](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)
- [`std::sync::Mutex` 문서](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
- [`std::sync::LazyLock` 문서](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
- [The Rust Programming Language: 공유 상태와 동시성](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
