# Observer 패턴

## 의도

Observer(관찰자) 패턴은 한 대상의 상태가 변경되면 그 대상에 등록된 여러 관찰자에게 변화를 알립니다. 대상은 구체적인 관찰자의 동작을 알 필요 없이 공통 인터페이스를 통해 알림을 전달합니다.

예를 들어 온도 측정값이 바뀌면 화면 표시기와 기록기가 각각 같은 변경 알림을 받아 자신의 작업을 수행할 수 있습니다.

## 구성 요소

- **Subject(주체):** 상태를 관리하고 관찰자를 등록·해제하며 변경을 알립니다.
- **Observer(관찰자):** 알림을 받는 공통 인터페이스를 정의합니다.
- **구체적인 관찰자:** 알림을 받아 화면 갱신, 기록 등 각자의 동작을 수행합니다.

일반적으로 Observer는 대상의 상태 변화에 따른 **알림**에 초점을 둡니다. [Mediator 패턴](/docs/mediator)은 여러 구성 요소가 **어떻게 협력하는지**를 조정하는 데 초점을 둡니다. 중재자 안에서 관찰자 방식으로 변화를 알리는 등 두 패턴을 함께 사용할 수도 있습니다.

## Rust에서 표현하기

Rust에서는 관찰자를 트레이트로 정의하고 `Vec<Box<dyn Observer>>`에 서로 다른 구현을 보관할 수 있습니다. 대상이 관찰자를 소유하므로 등록 해제 시 해당 관찰자는 제거됩니다. 공유 소유권이 필요하면 `Rc`나 `Arc`를 고려하되, 순환 참조와 스레드 안전성도 함께 설계해야 합니다.

알림을 언제 보낼지, 어떤 정보를 보낼지, 순서와 실패를 어떻게 처리할지는 애플리케이션의 정책입니다. 동기적으로 메서드를 호출하는 방식 외에도 채널을 이용해 작업 사이에 이벤트를 전달할 수 있습니다.

## Rust 예제: 온도 변경 알림

`TemperatureStation`은 정수 섭씨 온도를 보관하며, 값이 **실제로 변경된 경우에만** 등록된 관찰자를 등록 순서대로 호출합니다. 각 구독에는 고유한 이름을 사용합니다. 같은 이름을 다시 등록하면 오류가 반환되고, 등록되지 않은 이름의 해제는 `false`를 반환합니다.

`src/lib.rs`:

```rust
pub trait Observer {
    fn update(&mut self, temperature: i32);
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubscribeError {
    DuplicateName,
}

pub struct TemperatureStation {
    temperature: i32,
    observers: Vec<(String, Box<dyn Observer>)>,
}

impl TemperatureStation {
    pub fn new(temperature: i32) -> Self {
        Self { temperature, observers: Vec::new() }
    }

    pub fn temperature(&self) -> i32 {
        self.temperature
    }

    pub fn subscribe(
        &mut self,
        name: &str,
        observer: impl Observer + 'static,
    ) -> Result<(), SubscribeError> {
        if self.observers.iter().any(|(registered, _)| registered == name) {
            return Err(SubscribeError::DuplicateName);
        }
        self.observers.push((name.to_owned(), Box::new(observer)));
        Ok(())
    }

    pub fn unsubscribe(&mut self, name: &str) -> bool {
        if let Some(index) = self.observers.iter().position(|(registered, _)| registered == name) {
            self.observers.remove(index);
            true
        } else {
            false
        }
    }

    pub fn set_temperature(&mut self, temperature: i32) {
        if self.temperature == temperature {
            return;
        }
        self.temperature = temperature;
        for (_, observer) in &mut self.observers {
            observer.update(temperature);
        }
    }
}

pub struct Display;

impl Observer for Display {
    fn update(&mut self, temperature: i32) {
        println!("현재 온도: {temperature}°C");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Recorder(Rc<RefCell<Vec<i32>>>);

    impl Observer for Recorder {
        fn update(&mut self, temperature: i32) {
            self.0.borrow_mut().push(temperature);
        }
    }

    #[test]
    fn all_observers_receive_changes_but_not_unchanged_values() {
        let mut station = TemperatureStation::new(20);
        let first = Rc::new(RefCell::new(Vec::new()));
        let second = Rc::new(RefCell::new(Vec::new()));
        station.subscribe("first", Recorder(Rc::clone(&first))).unwrap();
        station.subscribe("second", Recorder(Rc::clone(&second))).unwrap();

        station.set_temperature(21);
        station.set_temperature(21);
        station.set_temperature(19);

        assert_eq!(*first.borrow(), vec![21, 19]);
        assert_eq!(*second.borrow(), vec![21, 19]);
        assert_eq!(station.temperature(), 19);
    }

    #[test]
    fn observers_are_notified_in_registration_order() {
        struct OrderedRecorder(&'static str, Rc<RefCell<Vec<&'static str>>>);
        impl Observer for OrderedRecorder {
            fn update(&mut self, _: i32) {
                self.1.borrow_mut().push(self.0);
            }
        }

        let mut station = TemperatureStation::new(20);
        let order = Rc::new(RefCell::new(Vec::new()));
        station.subscribe("first", OrderedRecorder("first", Rc::clone(&order))).unwrap();
        station.subscribe("second", OrderedRecorder("second", Rc::clone(&order))).unwrap();
        station.set_temperature(21);

        assert_eq!(*order.borrow(), vec!["first", "second"]);
    }

    #[test]
    fn unsubscribe_stops_only_that_observer() {
        let mut station = TemperatureStation::new(20);
        let first = Rc::new(RefCell::new(Vec::new()));
        let second = Rc::new(RefCell::new(Vec::new()));
        station.subscribe("first", Recorder(Rc::clone(&first))).unwrap();
        station.subscribe("second", Recorder(Rc::clone(&second))).unwrap();

        assert!(station.unsubscribe("first"));
        assert!(!station.unsubscribe("first"));
        station.set_temperature(22);

        assert!(first.borrow().is_empty());
        assert_eq!(*second.borrow(), vec![22]);
    }

    #[test]
    fn duplicate_name_does_not_replace_the_original_observer() {
        let mut station = TemperatureStation::new(20);
        let original = Rc::new(RefCell::new(Vec::new()));
        let duplicate = Rc::new(RefCell::new(Vec::new()));
        station.subscribe("display", Recorder(Rc::clone(&original))).unwrap();
        assert_eq!(
            station.subscribe("display", Recorder(Rc::clone(&duplicate))),
            Err(SubscribeError::DuplicateName)
        );

        station.set_temperature(23);
        assert_eq!(*original.borrow(), vec![23]);
        assert!(duplicate.borrow().is_empty());
    }

    #[test]
    fn station_updates_without_observers() {
        let mut station = TemperatureStation::new(20);
        station.set_temperature(25);
        assert_eq!(station.temperature(), 25);
        assert!(!station.unsubscribe("missing"));
    }
}
```

애플리케이션에서 `station.subscribe("display", Display)?`로 표시기를 등록할 수 있습니다. 예제의 테스트용 `Recorder`는 `Rc<RefCell<_>>`로 수신한 값을 공유해 검증하며, 이 조합은 단일 스레드용입니다. 새 관찰자에게 현재 온도를 즉시 전달하지 않고 다음 변경부터 알리는 정책도 이 예제에서 명시적으로 선택했습니다.

알림은 동기적으로 순서대로 실행됩니다. 따라서 느린 관찰자는 나머지 알림을 지연시키고, 관찰자가 패닉을 일으키면 다음 관찰자에게 전달되지 않을 수 있습니다. 오류 복구, 재진입, 비동기 처리나 멀티스레드 전달이 필요하면 별도의 정책과 구현을 설계해야 합니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 여러 관찰자에게 변경이 전달되는 경우와 알림 순서, 같은 값에 대한 중복 알림 방지, 개별 해제, 중복 이름 거부, 관찰자가 없는 경우를 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 주체가 구체적인 관찰자 타입을 알지 않아도 여러 구성 요소에 변화를 알릴 수 있습니다.
- 관찰자를 추가하거나 제거해 알림 수신 대상을 조정할 수 있습니다.
- 화면 갱신과 기록처럼 서로 다른 반응을 별도 관찰자로 분리할 수 있습니다.

### 단점

- 관찰자가 많거나 느리면 알림 처리 비용과 지연이 커질 수 있습니다.
- 알림 순서, 재진입, 오류나 패닉이 전체 전달에 미치는 영향을 정해야 합니다.
- 공유 참조를 사용하는 구현에서는 순환 참조나 동시성 문제가 발생할 수 있습니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Observer 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/observer)
- [Observer 패턴 — 위키백과](https://en.wikipedia.org/wiki/Observer_pattern)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [Rust Book: 메시지 전달로 스레드 간 데이터 전송하기](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [`std::rc` 표준 라이브러리 문서](https://doc.rust-lang.org/std/rc/index.html)
