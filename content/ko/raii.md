# RAII 가드

## 설명

RAII는 *Resource Acquisition Is Initialization*(자원 획득은 초기화)의 약자입니다. 이름만 보면 초기화 자체가 핵심처럼 보이지만, 패턴의 핵심은 객체를 만들 때 자원을 획득하고 객체가 파괴될 때 자원을 마무리하는 데 있습니다.

Rust에서는 RAII 객체를 자원의 **가드(guard)** 로 사용하고, 타입 시스템이 해당 가드를 통해서만 자원에 접근하도록 할 수 있습니다. 가드는 자원을 보유하거나 참조하고, 가드의 `Drop`이 실행될 때 자원을 정리합니다. 따라서 자원의 사용 가능 기간을 가드의 스코프와 연결할 수 있습니다.

## 예제: 뮤텍스 가드

표준 라이브러리의 뮤텍스 가드는 이 패턴의 대표적인 예입니다. 아래 코드는 실제 표준 라이브러리 구현이 아니라 핵심 아이디어만 보여 주도록 단순화한 예시입니다. `//..`로 표시한 부분은 실제 잠금 상태나 데이터 보관 등에 필요한 구현을 생략합니다.

```rust,ignore
use std::ops::Deref;

struct Foo {}

struct Mutex<T> {
    // We keep a reference to our data: T here.
    //..
}

struct MutexGuard<'a, T: 'a> {
    data: &'a T,
    //..
}

// Locking the mutex is explicit.
impl<T> Mutex<T> {
    fn lock(&self) -> MutexGuard<T> {
        // Lock the underlying OS mutex.
        //..

        // MutexGuard keeps a reference to self
        MutexGuard {
            data: self,
            //..
        }
    }
}

// Destructor for unlocking the mutex.
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // Unlock the underlying OS mutex.
        //..
    }
}

// Implementing Deref means we can treat MutexGuard like a pointer to T.
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

fn baz(x: Mutex<Foo>) {
    let xx = x.lock();
    xx.foo(); // foo is a method on Foo.
              // The borrow checker ensures we can't store a reference to the underlying
              // Foo which will outlive the guard xx.

    // x is unlocked when we exit this function and xx's destructor is executed.
}
```

`lock()`을 호출해 잠금을 얻는 동작은 코드에 명시적으로 나타납니다. 반환된 `MutexGuard`는 뮤텍스의 데이터에 대한 참조를 보유하고, `Deref`를 구현하므로 `xx.foo()`처럼 가드를 통해 내부 값의 메서드를 호출할 수 있습니다.

함수가 끝나면 지역 변수 `xx`가 스코프를 벗어나고 그 소멸자가 실행됩니다. 이 예제에서 `Drop` 구현은 그 시점에 운영체제 뮤텍스의 잠금을 해제합니다. 호출자가 잠금 해제를 잊더라도 가드의 수명이 끝나면 정리가 이루어지며, 가드가 살아 있는 동안에는 보호된 데이터에 접근할 수 있습니다.

## 실행 가능한 예제: 표준 뮤텍스 가드

다음 예제는 표준 라이브러리의 `Mutex`와 `MutexGuard`를 사용합니다. 가드가 살아 있는 동안 `try_lock()`은 `WouldBlock`을 반환하고, 가드가 스코프를 벗어나거나 `drop`으로 일찍 해제되면 다시 잠글 수 있습니다.

`src/main.rs`:

```rust
use std::sync::{Mutex, TryLockError};

fn increment_if(counter: &Mutex<u32>, enabled: bool) -> bool {
    let mut guard = counter.lock().expect("뮤텍스가 오염되었습니다");
    if !enabled {
        return false;
    }

    *guard += 1;
    true
}

fn main() {
    let counter = Mutex::new(0);

    {
        let mut guard = counter.lock().expect("뮤텍스가 오염되었습니다");
        *guard += 1;
        assert!(matches!(counter.try_lock(), Err(TryLockError::WouldBlock)));
    }

    let guard = counter.lock().expect("뮤텍스가 오염되었습니다");
    println!("스코프 종료 후: {}", *guard); // 1
    drop(guard);

    increment_if(&counter, true);
    println!(
        "다시 잠근 후: {}",
        *counter.lock().expect("뮤텍스가 오염되었습니다")
    ); // 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_exit_releases_the_lock_and_preserves_changes() {
        let counter = Mutex::new(0);
        {
            let mut guard = counter.lock().unwrap();
            *guard = 42;
            assert!(matches!(counter.try_lock(), Err(TryLockError::WouldBlock)));
        }

        assert_eq!(*counter.try_lock().unwrap(), 42);
    }

    #[test]
    fn explicit_drop_releases_the_lock_early() {
        let counter = Mutex::new(1);
        let guard = counter.lock().unwrap();
        assert!(matches!(counter.try_lock(), Err(TryLockError::WouldBlock)));

        drop(guard);
        assert_eq!(*counter.try_lock().unwrap(), 1);
    }

    #[test]
    fn early_return_releases_the_lock_without_changing_the_value() {
        let counter = Mutex::new(5);

        assert!(!increment_if(&counter, false));
        assert_eq!(*counter.try_lock().unwrap(), 5);
        assert!(increment_if(&counter, true));
        assert_eq!(*counter.try_lock().unwrap(), 6);
    }
}
```

`increment_if`에서 조건이 거짓이면 가드를 가진 채 일찍 반환합니다. 이때도 가드가 드롭되어 잠금이 해제됩니다. `drop(guard)`는 가드를 소비하므로 이후에는 그 가드를 사용할 수 없습니다. `try_lock()`은 기다리지 않고 잠금 가능 여부를 확인하며, `lock()`은 잠금을 얻을 때까지 기다립니다.

### 실행 및 테스트

위 코드를 Rust 바이너리 프로젝트의 `src/main.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo run
cargo test
```

테스트는 스코프 종료와 명시적 `drop`에 따른 잠금 해제, 조기 반환 시 잠금 해제와 값 보존을 확인합니다.

## 동기

사용이 끝난 뒤 반드시 정리하거나 마무리해야 하는 자원에 RAII를 적용할 수 있습니다. 자원을 정리한 뒤에는 더 이상 접근하면 안 되는 경우, 가드의 수명과 자원 접근을 묶어 이미 마무리된 자원을 사용하는 오류를 방지할 수 있습니다.

## 장점

RAII 가드는 다음 오류를 방지하는 데 도움을 줍니다.

- 사용이 끝난 자원을 정리하지 않고 남겨 두는 오류
- 정리 또는 마무리가 끝난 자원을 다시 사용하는 오류

## 논의

RAII는 자원을 적절히 해제하거나 마무리하도록 보장하는 데 유용합니다. Rust에서는 빌림 검사기를 활용해 자원이 마무리된 뒤에도 그 자원을 사용하려는 오류를 정적으로 방지할 수 있습니다.

다만 `Drop`은 모든 상황에서 실행되는 것은 아닙니다. 예를 들어 `std::mem::forget`으로 가드를 누출하거나 프로세스가 중단되면 정리가 수행되지 않을 수 있습니다. 패닉이 스택을 풀면서 발생할 때는 일반적으로 가드가 드롭되지만, 패닉 중 잠긴 `Mutex`는 오염되어 이후 잠금 시 오류를 반환할 수 있습니다.

빌림 검사기의 핵심 목표는 참조가 참조 대상 데이터보다 오래 살아남지 못하게 하는 것입니다. RAII 가드 패턴에서는 가드가 실제 자원에 대한 참조를 보유하고, 사용자는 가드를 통해 자원에 대한 참조를 얻습니다. Rust는 가드가 기반 자원보다 오래 살지 못하게 하며, 가드를 통해 얻은 참조도 가드보다 오래 살지 못하게 합니다.

이 관계를 이해하려면 생략된 수명을 모두 표시한 `Deref::deref`의 형태를 살펴보면 좋습니다.

```rust,ignore
fn deref<'a>(&'a self) -> &'a T {
    //..
}
```

반환되는 자원 참조의 수명은 `self`를 빌린 수명인 `'a`와 같습니다. 따라서 반환 참조는 가드에 대한 빌림보다 오래 살 수 없고, 빌림 검사기가 이 제약을 확인합니다. 결과적으로 가드를 통한 자원 참조를 가드가 사라진 뒤까지 보관할 수 없습니다.

`Deref` 구현은 RAII 가드 패턴의 필수 요소가 아닙니다. 가드에서 `get` 같은 메서드를 제공해도 같은 방식으로 자원에 접근할 수 있습니다. `Deref`는 가드를 내부 값처럼 다룰 수 있게 해 사용을 더 편리하게 만들 뿐입니다.

## 참고 자료

- [소멸자에서의 마무리 작업](https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html)
- [C++의 RAII: cppreference](https://web.archive.org/web/20260918073742/https://en.cppreference.com/cpp/language/raii)
- [RAII](https://en.wikipedia.org/wiki/Resource_Acquisition_Is_Initialization)
- [`Mutex` 표준 라이브러리 문서](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
- [`MutexGuard` 표준 라이브러리 문서](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html)
- [Rust 스타일 가이드의 RAII 항목](https://doc.rust-lang.org/1.0.0/style/ownership/raii.html) — 현재는 자리 표시자 문서입니다.
