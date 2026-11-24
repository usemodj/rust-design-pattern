# 스택에서의 동적 디스패치

## 설명

여러 값 중 하나를 조건에 따라 선택하고 동적 디스패치로 다룰 수 있습니다. 각 값의 구체 타입이 서로 다르므로, 공통 트레이트 객체 타입으로 묶어야 합니다. 이때 조건부 초기화(deferred conditional initialization)를 사용하면 각 값을 담을 변수를 미리 선언하지 않고도 필요한 값의 수명을 충분히 유지할 수 있습니다.

## 예제

아래에서는 인자가 `"-"`이면 표준 입력을, 그렇지 않으면 파일을 선택합니다. `Stdin`과 `File`은 서로 다른 구체 타입이지만 둘 다 `io::Read`를 구현하므로 `&mut dyn io::Read`로 묶어 이후 로직에서 공통으로 다룰 수 있습니다. `?`는 파일을 여는 과정에서 발생할 수 있는 오류를 호출자에게 전달합니다.

```rust
use std::io;
use std::fs;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let arg = "-";

// We need to describe the type to get dynamic dispatch.
let readable: &mut dyn io::Read = if arg == "-" {
    &mut io::stdin()
} else {
    &mut fs::File::open(arg)?
};

// Read from `readable` here.

# Ok(())
# }
```

## 동기

Rust는 기본적으로 모노모피제이션을 사용합니다. 제네릭 코드를 여러 타입에 사용하면 타입마다 별도의 코드가 생성되고 각각 독립적으로 최적화됩니다. 이 방식은 성능이 중요한 실행 경로에서 빠른 코드를 만드는 데 도움이 되지만, 성능이 중요하지 않은 코드까지 타입별로 복제하면 실행 파일의 코드 크기가 커지고 컴파일 시간과 캐시 사용량이 늘어날 수 있습니다.

Rust에서는 동적 디스패치를 사용할 수 있지만, 이를 명시적으로 요청해야 합니다. `&mut dyn io::Read`는 구체 타입 대신 트레이트 객체를 사용해 호출 시점에 구현을 선택하도록 합니다.

## 장점

- `Box`를 사용해 객체를 힙에 할당할 필요가 없습니다.
- 이후에 사용하지 않을 수도 있는 값을 미리 초기화하지 않아도 됩니다.
- 이후의 전체 로직을 `File`과 `Stdin` 각각에 대해 모노모피제이션할 필요가 없습니다.

## 단점

Rust 1.79.0 이전에는 조건에 따라 타입이 다른 값을 빌리려면 지연 초기화(deferred initialization)를 사용하는 두 개의 `let` 바인딩이 필요했습니다. 이 방식은 `Box`를 사용하는 구현보다 구성 요소가 많았습니다.

```rust,ignore
// We still need to ascribe the type for dynamic dispatch.
let readable: Box<dyn io::Read> = if arg == "-" {
    Box::new(io::stdin())
} else {
    Box::new(fs::File::open(arg)?)
};
// Read from `readable` here.
```

다행히 이 단점은 이제 사라졌습니다.

## 논의

Rust 1.79.0부터 컴파일러는 `&` 또는 `&mut` 안에 있는 임시 값의 수명을 함수 스코프 안에서 가능한 한 길게 자동 연장합니다. 따라서 임시 값을 별도의 `let` 바인딩에 넣지 않고도 위와 같이 `&mut` 참조를 사용할 수 있습니다. 이 변경 전에는 조건에 따라 값을 지연 초기화하는 방식이 필요했습니다.

각 값에는 저장될 장소가 있습니다. 그 장소가 임시 장소여도 괜찮습니다. 컴파일러는 각 값의 크기를 알고 있으며, 빌린 값은 그 값에서 만들어진 참조가 사용되는 동안 살아 있습니다.

## 참고 자료

- 수명을 세밀하게 제어해야 하는 경우 [`Drop`을 이용한 마무리 작업](/docs/destructor-finalization)과 [RAII 가드](/docs/raii)를 참고하세요.
- 변경 가능한 참조를 포함한 `Option<&T>`를 조건에 따라 채워야 한다면, `Option<T>`를 직접 초기화한 뒤 [`.as_ref()`](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_ref)를 사용해 선택적 참조를 얻을 수 있습니다.
