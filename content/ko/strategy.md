# Strategy 패턴 (Policy 패턴)

## 설명

[Strategy 패턴](https://en.wikipedia.org/wiki/Strategy_pattern)은 관심사를 분리하고 [의존성 역전 원칙(Dependency Inversion)](https://en.wikipedia.org/wiki/Dependency_inversion_principle)을 통해 소프트웨어 모듈 사이의 결합을 낮추는 기법입니다.

핵심 아이디어는 특정 문제를 해결하는 알고리즘의 **전체 흐름(골격)** 과 구체적인 알고리즘 구현을 분리하는 것입니다. 알고리즘을 사용하는 쪽은 일반적인 작업 순서를 유지하면서 필요한 구체 전략을 선택할 수 있습니다. 상위 수준의 추상화는 특정 구현에 의존하지 않고, 각각의 구체 구현은 추상화가 정한 규약을 따라야 합니다. 이러한 관계를 의존성 역전이라고 합니다.

Rust에서는 전략을 트레이트를 구현하는 타입으로 표현할 수 있습니다. 상황에 따라 클로저나 함수처럼 호출 가능한 값을 넘기는 방법도 있습니다.

## 동기

매달 보고서를 생성하는 프로젝트를 만든다고 생각해 봅시다. 보고서 데이터는 같아도 출력 형식은 JSON이나 일반 텍스트처럼 여러 가지가 필요할 수 있습니다. 요구사항은 시간이 지나며 바뀌기 때문에 앞으로 완전히 새로운 형식이 추가될 수도 있고, 기존 형식 중 하나의 구현만 수정해야 할 수도 있습니다.

보고서 데이터의 준비와 조회 흐름을 각 출력 형식에 중복해서 구현하는 대신, 데이터 생성은 공통 흐름으로 유지하고 결과를 표현하는 방식만 전략으로 분리할 수 있습니다.

## 예제: 트레이트로 전략 정의하기

이 예제에서 `Formatter`와 `Report`는 변하지 않아야 할 추상화이고, `Text`와 `Json`은 서로 교체 가능한 전략입니다. 각 전략은 `Formatter` 트레이트를 구현해야 합니다.

```rust
use std::collections::HashMap;

type Data = HashMap<String, u32>;

trait Formatter {
    fn format(&self, data: &Data, buf: &mut String);
}

struct Report;

impl Report {
    // Write should be used but we kept it as String to ignore error handling
    fn generate<T: Formatter>(g: T, s: &mut String) {
        // backend operations...
        let mut data = HashMap::new();
        data.insert("one".to_string(), 1);
        data.insert("two".to_string(), 2);
        // generate report
        g.format(&data, s);
    }
}

struct Text;
impl Formatter for Text {
    fn format(&self, data: &Data, buf: &mut String) {
        for (k, v) in data {
            let entry = format!("{k} {v}\n");
            buf.push_str(&entry);
        }
    }
}

struct Json;
impl Formatter for Json {
    fn format(&self, data: &Data, buf: &mut String) {
        buf.push('[');
        for (k, v) in data.into_iter() {
            let entry = format!(r#"{{"{}":"{}"}}"#, k, v);
            buf.push_str(&entry);
            buf.push(',');
        }
        if !data.is_empty() {
            buf.pop(); // remove extra , at the end
        }
        buf.push(']');
    }
}

fn main() {
    let mut s = String::from("");
    Report::generate(Text, &mut s);
    assert!(s.contains("one 1"));
    assert!(s.contains("two 2"));

    s.clear(); // reuse the same buffer
    Report::generate(Json, &mut s);
    assert!(s.contains(r#"{"one":"1"}"#));
    assert!(s.contains(r#"{"two":"2"}"#));
}
```

`Report::generate`는 데이터를 준비하는 공통 작업을 수행한 다음 `Formatter::format`을 호출합니다. `Report`는 실제 포맷터가 `Text`인지 `Json`인지 알 필요가 없습니다. 포맷터가 `Formatter` 계약을 만족하기만 하면 됩니다. 반대로 포맷터는 보고서 데이터가 어떤 저장소에서 왔거나 어떻게 준비됐는지 알 필요가 없으며, 받은 데이터를 어떻게 출력할지만 책임집니다.

`generate<T: Formatter>`는 제네릭을 사용하므로 이 예제에서는 구체 전략 타입이 컴파일 시점에 정해집니다. 같은 출력 버퍼 `s`를 텍스트 보고서와 JSON 보고서에 차례로 재사용하기 전에 `clear()`로 비웁니다. 예제는 오류 처리를 간소화하려고 `String`을 사용하며, 실제로 쓰기 오류를 처리해야 하는 코드라면 원문 주석처럼 `std::io::Write`와 같은 쓰기 인터페이스를 고려할 수 있습니다.

## 장점

가장 큰 장점은 **관심사 분리**입니다. `Report`는 `Json`과 `Text`의 구체적인 구현을 알지 못합니다. 출력 구현은 데이터가 어떻게 전처리되고 저장되며 조회되는지 신경 쓰지 않습니다. 두 부분이 알아야 할 것은 공통 트레이트와 결과 데이터를 처리하는 메서드, 즉 `Formatter`와 `format(...)`뿐입니다.

따라서 새 출력 형식을 추가할 때 공통 보고서 생성 흐름을 다시 작성하지 않고 새 전략을 구현할 수 있습니다. 기존 형식의 구현을 변경할 때도 보고서 데이터 준비 로직과 분리해 수정할 수 있습니다.

## 단점

전략마다 적어도 하나의 구현 모듈을 추가해야 하므로 전략의 수가 늘면 모듈 수도 증가합니다. 선택지가 많아지면 사용하는 사람도 각 전략이 어떻게 다른지 파악하고 적절한 구현을 선택해야 합니다.

## 논의

예제에서는 모든 전략을 한 파일에 두었지만, 프로젝트 구조와 기능 제공 방식에 따라 여러 방식으로 나눌 수 있습니다.

- **한 파일에 함께 두기:** 전략이 적거나 구현이 작을 때 간단합니다. 예제처럼 모듈로 분리하지 않은 형태입니다.
- **모듈로 분리하기:** 예를 들어 `formatter::json`, `formatter::text`처럼 각 전략을 별도 모듈로 둘 수 있습니다.
- **컴파일러 기능 플래그로 선택하기:** `json`, `text` 같은 Cargo feature를 두어 필요한 전략만 포함할 수 있습니다.
- **별도 크레이트로 분리하기:** `json` 크레이트와 `text` 크레이트처럼 각 구현을 독립 패키지로 제공할 수 있습니다.

[Serde](https://serde.rs/)는 Strategy 패턴의 좋은 사례입니다. 사용자 타입에 `Serialize`와 `Deserialize`를 직접 구현하면 직렬화와 역직렬화 동작을 세밀하게 사용자화할 수 있습니다. 또한 비슷한 메서드를 제공하는 `serde_json`과 `serde_cbor`처럼 직렬화 형식 구현을 바꿔 사용할 수 있습니다. 이처럼 여러 형식을 전략으로 분리하면 도우미 크레이트인 [`serde_transcode`](https://github.com/hyperium/serde-transcode)를 더욱 유용하고 편리하게 만들 수 있습니다.

Strategy 패턴을 Rust에서 구현하기 위해 반드시 트레이트를 사용해야 하는 것은 아닙니다. 동작이 작고 함수로 표현할 수 있다면 클로저를 전략으로 전달할 수도 있습니다.

### 클로저를 사용하는 예제

다음 예제의 `Adder::add`는 두 입력을 받아 계산을 수행하는 함수를 인자로 받습니다. 산술 덧셈, 두 값 중 하나라도 `1`이면 `1`을 반환하는 연산, 사용자 정의 연산을 각각 다른 클로저로 전달합니다. 입력을 받는 공통 메서드는 그대로 두고 계산 방법만 교체합니다.

```rust
struct Adder;
impl Adder {
    pub fn add<F>(x: u8, y: u8, f: F) -> u8
    where
        F: Fn(u8, u8) -> u8,
    {
        f(x, y)
    }
}

fn main() {
    let arith_adder = |x, y| x + y;
    let bool_adder = |x, y| {
        if x == 1 || y == 1 {
            1
        } else {
            0
        }
    };
    let custom_adder = |x, y| 2 * x + y;

    assert_eq!(9, Adder::add(4, 5, arith_adder));
    assert_eq!(0, Adder::add(0, 0, bool_adder));
    assert_eq!(5, Adder::add(1, 3, custom_adder));
}
```

`F: Fn(u8, u8) -> u8` 제약은 전달된 값이 두 `u8` 인자를 받아 `u8`을 반환하는 함수나 클로저임을 나타냅니다. 클로저가 상태를 캡처하거나 더 복잡한 인터페이스가 필요하다면 `FnMut` 또는 `FnOnce` 등 적절한 호출 트레이트를 선택할 수 있습니다.

### `Option::map`에서의 전략

Rust 표준 라이브러리의 `Option::map`도 같은 아이디어를 사용합니다. `Some` 안의 값을 변환하는 동작을 함수나 클로저로 전달하므로, 변환 전략을 선택할 수 있습니다.

```rust
fn main() {
    let val = Some("Rust");

    let len_strategy = |s: &str| s.len();
    assert_eq!(4, val.map(len_strategy).unwrap());

    let first_byte_strategy = |s: &str| s.bytes().next().unwrap();
    assert_eq!(82, val.map(first_byte_strategy).unwrap());
}
```

첫 번째 클로저는 문자열의 길이를 반환합니다. 두 번째 클로저는 문자열의 첫 바이트를 반환하며, `"Rust"`의 첫 바이트인 `R`의 값은 `82`입니다. `Option::map`은 값이 `Some`일 때 전달한 전략을 적용하고, 그 결과를 다시 `Option`으로 감쌉니다.

## 참고 자료

- [Strategy 패턴](https://en.wikipedia.org/wiki/Strategy_pattern)
- [의존성 주입(Dependency Injection)](https://en.wikipedia.org/wiki/Dependency_injection)
- [Policy 기반 설계](https://en.wikipedia.org/wiki/Modern_C%2B%2B_Design#Policy-based_design)
- [Strategy 패턴을 사용한 Rust 우주 응용 TCP 서버 구현](https://web.archive.org/web/20231003171500/https://robamu.github.io/posts/rust-strategy-pattern/)
