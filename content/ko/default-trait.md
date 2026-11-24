# `Default` 트레이트

## 설명

Rust의 많은 타입은 값을 만드는 생성자를 제공합니다. 그러나 생성자는 해당 타입에만 한정됩니다. Rust에는 `new()` 메서드가 있는 모든 타입을 하나로 추상화하는 일반적인 방법이 없습니다. 이러한 목적을 위해 `Default` 트레이트가 마련되었습니다. `Default`는 컨테이너나 다른 제네릭 타입에서 사용할 수 있습니다. 예를 들어 [`Option::unwrap_or_default()`](https://doc.rust-lang.org/stable/std/option/enum.Option.html#method.unwrap_or_default)는 값이 `None`일 때 내부 타입의 기본값을 반환합니다. 일부 컨테이너 타입은 조건이 맞으면 이미 `Default`를 구현하고 있습니다.

`Cow`, `Box`, `Arc` 같은 단일 값 컨테이너는 내부 타입이 `Default`를 구현할 때 자신도 기본값을 제공할 수 있습니다. 또한 구조체의 모든 필드 타입이 `Default`를 구현한다면 `#[derive(Default)]`를 사용해 기본값 구현을 자동으로 만들 수 있습니다. 더 많은 타입이 `Default`를 구현할수록 이를 활용할 수 있는 컨테이너와 제네릭 코드도 많아집니다.

반면 생성자는 여러 인자를 받을 수 있지만 `default()` 메서드는 인자를 받지 않습니다. 생성자는 서로 다른 이름으로 여러 개 정의할 수도 있지만, 한 타입에는 `Default` 구현이 하나만 존재할 수 있습니다. 따라서 기본 상태를 만드는 일에는 `Default`를 사용하고, 입력을 받거나 서로 다른 방식으로 값을 초기화해야 한다면 타입에 맞는 생성자를 별도로 제공하는 것이 적절합니다.

## 예제

아래 설정 구조체의 모든 필드 타입은 `Default`를 구현하므로 `#[derive(Default)]`를 사용할 수 있습니다. 기본값은 `Option`의 `None`, `Vec`의 빈 벡터, `Duration`의 0, `bool`의 `false`입니다. `MyConfiguration::default()`로 전체 기본값을 만든 다음 일부 값을 수정할 수 있습니다. 또한 구조체 업데이트 문법을 사용하면 일부 필드만 지정하고 나머지는 기본값으로 채울 수 있습니다. 두 방식으로 만든 값은 동일합니다.

```rust
use std::{path::PathBuf, time::Duration};

// note that we can simply auto-derive Default here.
#[derive(Default, Debug, PartialEq)]
struct MyConfiguration {
    // Option defaults to None
    output: Option<PathBuf>,
    // Vecs default to empty vector
    search_path: Vec<PathBuf>,
    // Duration defaults to zero time
    timeout: Duration,
    // bool defaults to false
    check: bool,
}

impl MyConfiguration {
    // add setters here
}

fn main() {
    // construct a new instance with default values
    let mut conf = MyConfiguration::default();
    // do something with conf here
    conf.check = true;
    println!("conf = {conf:#?}");

    // partial initialization with default values, creates the same instance
    let conf1 = MyConfiguration {
        check: true,
        ..Default::default()
    };
    assert_eq!(conf, conf1);
}
```

## 참고 자료

- [생성자 관용구](/docs/constructor): 기본값이 아닌 상태를 포함해 인스턴스를 만드는 또 다른 방법입니다.
- [`Default` 표준 라이브러리 문서](https://doc.rust-lang.org/stable/std/default/trait.Default.html): 구현 타입 목록도 확인할 수 있습니다.
- [`Option::unwrap_or_default()` 문서](https://doc.rust-lang.org/stable/std/option/enum.Option.html#method.unwrap_or_default)
- [`derive(new)` 크레이트](https://crates.io/crates/derive-new): 생성자 파생을 지원하는 외부 크레이트입니다.
