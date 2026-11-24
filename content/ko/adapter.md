# Adapter 패턴

## 의도

Adapter(어댑터) 패턴은 이미 존재하는 타입의 인터페이스를 클라이언트가 기대하는 인터페이스로 변환합니다. 어댑터는 기존 타입(Adaptee)을 감싸고, 클라이언트가 사용하는 대상 인터페이스(Target)를 구현합니다. 이렇게 하면 기존 구현을 수정하지 않고도 서로 맞지 않는 API를 연결할 수 있습니다.

구성 요소는 다음과 같습니다.

- **대상 인터페이스(Target):** 클라이언트가 사용하기를 기대하는 API입니다.
- **기존 타입(Adaptee):** 필요한 기능은 제공하지만, 대상 인터페이스와 API가 맞지 않는 타입입니다.
- **어댑터(Adapter):** 기존 타입을 보유하거나 참조하며, 대상 인터페이스 호출을 기존 타입의 동작으로 변환합니다.
- **클라이언트(Client):** 구체적인 기존 타입이 아니라 대상 인터페이스에 의존합니다.

## Rust에서 표현하기

전통적인 객체 지향 구현은 상속을 이용한 클래스 어댑터와 합성을 이용한 객체 어댑터를 구분합니다. Rust에는 클래스 상속이 없으므로, 보통 구조체가 기존 타입을 소유하거나 빌리고 대상 트레이트를 구현하는 합성 방식으로 Adapter를 표현합니다.

단순한 값 변환만 필요하다면 `From`이나 `TryFrom`으로 충분할 수 있습니다. 기존 API의 메서드 호출을 번역하거나, 단위·오류·호출 순서처럼 여러 규칙을 함께 변환해야 한다면 별도의 어댑터 타입이 더 적절합니다.

## Rust 예제: 화씨 온도 센서를 섭씨 인터페이스에 연결하기

기존 센서는 온도를 화씨로만 제공하지만, 방 온도 모니터는 섭씨 단위의 `CelsiusSensor`를 기대한다고 가정합니다. `FahrenheitAdapter`는 기존 센서를 감싸고, 섭씨로 변환한 결과를 대상 인터페이스로 제공합니다.

`src/lib.rs`:

```rust
pub trait CelsiusSensor {
    fn read_celsius(&self) -> f64;
}

pub struct LegacyFahrenheitSensor {
    fahrenheit: f64,
}

impl LegacyFahrenheitSensor {
    pub fn new(fahrenheit: f64) -> Self {
        Self { fahrenheit }
    }

    pub fn read_fahrenheit(&self) -> f64 {
        self.fahrenheit
    }
}

pub struct FahrenheitAdapter {
    sensor: LegacyFahrenheitSensor,
}

impl FahrenheitAdapter {
    pub fn new(sensor: LegacyFahrenheitSensor) -> Self {
        Self { sensor }
    }
}

impl CelsiusSensor for FahrenheitAdapter {
    fn read_celsius(&self) -> f64 {
        (self.sensor.read_fahrenheit() - 32.0) * 5.0 / 9.0
    }
}

pub fn room_status(sensor: &dyn CelsiusSensor) -> &'static str {
    if sensor.read_celsius() >= 24.0 {
        "냉방이 필요합니다."
    } else {
        "온도가 적절합니다."
    }
}

#[cfg(test)]
mod tests {
    use super::{room_status, CelsiusSensor, FahrenheitAdapter, LegacyFahrenheitSensor};

    #[test]
    fn adapter_converts_fahrenheit_to_celsius() {
        let legacy_sensor = LegacyFahrenheitSensor::new(68.0);
        let adapter = FahrenheitAdapter::new(legacy_sensor);

        assert!((adapter.read_celsius() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn client_uses_the_target_interface_without_knowing_the_legacy_type() {
        let legacy_sensor = LegacyFahrenheitSensor::new(77.0);
        let adapter = FahrenheitAdapter::new(legacy_sensor);

        assert_eq!(room_status(&adapter), "냉방이 필요합니다.");
    }
}
```

`room_status`는 화씨 센서의 존재를 알지 못하며, 섭씨 인터페이스만 사용합니다. 기존 라이브러리나 장치 드라이버의 타입을 수정할 필요 없이 어댑터를 통해 클라이언트와 연결할 수 있습니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 확인합니다.

1. 어댑터가 화씨 값을 올바른 섭씨 값으로 변환합니다.
2. 클라이언트가 기존 센서 타입이 아닌 대상 트레이트를 사용해 온도에 따른 동작을 결정합니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

실제 센서 API가 오류를 반환할 수 있다면, 대상 트레이트도 `Result<f64, SensorError>`처럼 오류를 표현하고 어댑터에서 오류를 보존하거나 명시적으로 변환해야 합니다. 예제의 단순한 `f64` 반환은 변환 구조를 보여 주기 위한 것입니다.

## 장점과 단점

### 장점

- 기존 타입을 수정하지 않고 새 인터페이스와 연결할 수 있습니다.
- 클라이언트가 특정 구현의 API나 단위 체계에 직접 의존하지 않게 합니다.
- 기존 API가 여러 개여도 각 어댑터를 통해 하나의 대상 인터페이스로 통일할 수 있습니다.
- 단위 변환, 오류 변환, 호출 순서 조정 같은 경계 규칙을 한 곳에 모을 수 있습니다.

### 단점

- 래퍼 타입과 변환 코드가 추가되어 시스템의 구조가 복잡해집니다.
- 어댑터가 변환 규칙이나 오류를 잘못 처리하면 의미가 달라질 수 있습니다.
- 단순한 타입 변환에 별도 추상화를 만들면 `From`이나 작은 변환 함수보다 장황할 수 있습니다.
- 어댑터가 너무 많은 책임을 맡으면 기존 API와 대상 API의 차이를 파악하기 어려워집니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Adapter 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/adapter)
- [Adapter 패턴 — 위키백과](https://en.wikipedia.org/wiki/Adapter_pattern)
- [Rust Book: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [`std::convert::From` 문서](https://doc.rust-lang.org/std/convert/trait.From.html)
- [`std::convert::TryFrom` 문서](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)
