# `unsafe`를 작은 모듈에 가두기

## 설명

`unsafe` 코드가 필요하다면, 먼저 그 코드가 의존하는 불변식을 유지할 수 있는 가장 작은 모듈을 만드세요. 내부 모듈은 안전하지 않은 연산을 사용하되 그 전제 조건을 지키고, 바깥에는 그 불변식을 보장하는 최소한의 안전한 인터페이스를 제공합니다. 애플리케이션의 나머지 코드는 내부 구현을 직접 다루지 않고, 검토하기 쉬운 일반적인 안전 Rust API를 사용하게 됩니다.

이 경계가 중요한 이유는 `unsafe` 블록만 좁게 표시한다고 안전성 검토 범위까지 그 블록 안으로 한정되는 것은 아니기 때문입니다. 안전하지 않은 연산이 구조체의 상태나 이전에 호출된 메서드가 지킨 조건에 의존한다면, 그 상태를 변경할 수 있는 코드도 함께 검토해야 합니다. 비공개 필드와 모듈 경계는 불변식을 깨뜨릴 수 있는 경로를 제한하는 데 도움이 됩니다.

모듈 바깥에 `unsafe fn`이나 `unsafe` 메서드를 명시적으로 노출할 수도 있습니다. 이 경우 구현은 안전하지 않은 연산을 모듈 안에 둘 수 있지만, 호출자는 함수의 안전 조건을 만족해야 합니다. 따라서 “`unsafe`를 가둔다”는 것은 모든 `unsafe` 사용을 감추라는 뜻이 아니라, 가능한 곳에서는 불변식을 내부에서 관리하고, 호출자에게 맡기는 조건은 명확히 문서화하라는 뜻입니다.

## 예제: UTF-8 불변식을 유지하는 내부 모듈

표준 라이브러리의 `String`은 UTF-8 바이트를 저장하고 문자열 내용이 항상 올바른 UTF-8이라는 불변식을 유지합니다. 이 조건이 지켜지기 때문에 `String`은 내용을 안전하게 `str`로 제공할 수 있습니다. 원문은 `String`을 UTF-8 불변식을 추가한 `Vec<u8>` 래퍼의 사례로 소개합니다.

아래는 같은 설계 원리를 보여 주는 학습용 예제입니다. `Utf8Text`의 바이트 벡터는 비공개이며, 생성 시 입력의 UTF-8 유효성을 검사하고 이후에는 유효한 `&str`만 추가합니다. 따라서 `as_str` 안의 `from_utf8_unchecked`가 요구하는 조건을 이 모듈이 보장합니다.

### 디렉터리 구조

```text
utf8-text/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    └── utf8_text.rs
```

### 패키지 매니페스트

`Cargo.toml`:

```toml
[package]
name = "utf8-text"
version = "0.1.0"
edition = "2024"

[lib]
name = "utf8_text"
path = "src/lib.rs"
```

### 작은 내부 모듈과 안전한 외부 API

`src/lib.rs`:

```rust
mod utf8_buffer {
    use std::string::FromUtf8Error;

    pub struct Utf8Text {
        bytes: Vec<u8>,
    }

    impl Utf8Text {
        pub fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, FromUtf8Error> {
            let text = String::from_utf8(bytes)?;
            Ok(Self {
                bytes: text.into_bytes(),
            })
        }

        pub fn as_str(&self) -> &str {
            // SAFETY: 생성 시 UTF-8을 검사하고, push_str은 유효한 &str만 추가합니다.
            // bytes는 비공개이며 다른 변경 경로가 없어 이 불변식은 계속 유지됩니다.
            unsafe { std::str::from_utf8_unchecked(&self.bytes) }
        }

        pub fn push_str(&mut self, suffix: &str) {
            self.bytes.extend_from_slice(suffix.as_bytes());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Utf8Text;

        #[test]
        fn valid_bytes_can_be_read_as_text() {
            let text = Utf8Text::try_from_bytes("안녕".as_bytes().to_vec()).unwrap();

            assert_eq!(text.as_str(), "안녕");
        }

        #[test]
        fn appending_str_preserves_the_utf8_invariant() {
            let mut text = Utf8Text::try_from_bytes("Rust".as_bytes().to_vec()).unwrap();
            text.push_str(" 언어");

            assert_eq!(text.as_str(), "Rust 언어");
        }
    }
}

pub use utf8_buffer::Utf8Text;
```

`utf8_buffer` 모듈 자체는 비공개이고, 바이트 필드도 외부에서 접근할 수 없습니다. 외부에는 `Utf8Text` 타입만 다시 내보냅니다. `try_from_bytes`는 표준 라이브러리의 안전한 `String::from_utf8`를 이용해 임의의 바이트를 검사한 뒤, 검증된 바이트를 저장합니다. `push_str`는 이미 UTF-8인 문자열의 바이트만 덧붙이므로 유효성을 보존합니다.

`as_str`의 `unsafe` 블록은 그 불변식을 이용해 매번 UTF-8 검사를 다시 하지 않고 문자열 슬라이스를 만듭니다. 안전성의 근거는 블록 자체가 아니라 생성과 변경 경로 전체가 불변식을 지키는 데 있습니다. 새로운 가변 접근자나 바이트 교체 메서드를 추가한다면 그 메서드도 반드시 같은 조건을 보존해야 합니다.

이 타입은 모듈 경계를 설명하기 위한 예제이지 `std::String`을 대체하기 위한 구현이 아닙니다. 실제 애플리케이션에서 문자열이 필요하다면 표준 라이브러리의 `String`을 우선 사용하세요.

### 외부 통합 테스트

`tests/utf8_text.rs`:

```rust
use utf8_text::Utf8Text;

#[test]
fn safe_api_rejects_invalid_utf8() {
    let result = Utf8Text::try_from_bytes(vec![0xF0, 0x9F]);

    assert!(result.is_err());
}

#[test]
fn public_api_keeps_appended_text_valid() {
    let mut text = Utf8Text::try_from_bytes(vec![0xF0, 0x9F, 0x92, 0x96]).unwrap();
    text.push_str(" Rust");

    assert_eq!(text.as_str(), "💖 Rust");
}
```

통합 테스트는 크레이트 외부의 사용자가 접근할 수 있는 API만 사용합니다. 잘못된 UTF-8 입력이 안전하게 거부되는지, 유효한 문자열을 추가한 뒤에도 안전하게 읽을 수 있는지를 확인합니다. 테스트는 불변식을 유지하는 모든 가능한 실행 경로를 증명하지는 않으므로, 필드의 비공개성, 생성·변경 메서드와 `unsafe` 블록의 전제 조건도 함께 검토해야 합니다.

패키지 루트에서 다음 명령으로 단위 테스트와 통합 테스트를 실행합니다.

```console
cargo test
```

## 장점

- **감사할 코드의 범위를 제한합니다.** 안전하지 않은 연산과 그에 필요한 상태 변경을 작은 비공개 모듈에 모으면 검토자가 불변식과 가능한 변경 경로를 함께 확인하기 쉽습니다.
- **바깥 코드를 단순하게 만듭니다.** 바깥 모듈은 내부의 안전 보장을 신뢰하고, 안전한 인터페이스만 사용하면 됩니다.
- **사용성을 유지할 수 있습니다.** 내부 구현에 `unsafe`가 필요하더라도 API를 안전하게 만들 수 있다면, 일반 호출자가 별도의 안전 조건을 증명할 필요가 없습니다.

## 단점과 주의점

- **적절한 추상화를 찾기 어려울 수 있습니다.** 안전한 API가 충분히 유용하면서도 불변식을 깨뜨리지 않도록 경계와 메서드를 설계해야 합니다.
- **추상화 비용이 생길 수 있습니다.** 추가적인 검사, 복사 또는 간접 호출이 성능에 영향을 줄 수 있습니다. 안전한 구현이 충분히 빠른지 먼저 확인하고, 필요한 경우 실제 사용 환경에서 측정하세요.
- **모듈 경계만으로 안전성이 증명되지는 않습니다.** 비공개 모듈 안의 안전한 코드도 불변식을 깨뜨릴 수 있고, 그 결과 `unsafe` 연산이 잘못될 수 있습니다. 모든 변경 경로를 함께 검토해야 합니다.
- **테스트만으로 `unsafe`의 정확성을 보장할 수 없습니다.** 경계 사례 테스트는 회귀를 찾는 데 도움이 되지만, 호출자가 만들 수 있는 모든 상태와 실행 순서를 대신 증명하지는 않습니다.

## 원문에 소개된 사례

- [`toolshed` 크레이트](https://docs.rs/toolshed)는 하위 모듈에 `unsafe` 연산을 두고 사용자에게 안전한 인터페이스를 제공하는 사례로 소개됩니다.
- 표준 라이브러리의 [`String`](https://doc.rust-lang.org/std/string/struct.String.html)은 내부 바이트가 유효한 UTF-8이라는 조건을 유지합니다. 안전한 메서드는 이 조건을 보존하고, `String::from_utf8_unchecked`처럼 `unsafe` API를 직접 사용하는 호출자는 유효한 UTF-8을 제공할 책임을 집니다.

## 참고 자료

- [Rust Design Patterns 원문: Contain unsafety in small modules](https://rust-unofficial.github.io/patterns/patterns/structural/unsafe-mods.html)
- [The Rustonomicon: Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
- [Rust Reference: `unsafe` 키워드와 안전 조건](https://doc.rust-lang.org/reference/unsafe-keyword.html)
- [`std::str::from_utf8_unchecked` 안전 조건](https://doc.rust-lang.org/std/str/fn.from_utf8_unchecked.html)
- [`String` 표준 라이브러리 문서](https://doc.rust-lang.org/std/string/struct.String.html)
- [Ralf Jung: Two Kinds of Invariants — Safety and Validity](https://www.ralfj.de/blog/2018/08/22/two-kinds-of-invariants.html)
