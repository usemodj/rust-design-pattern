# 작은 크레이트를 선호하기

## 설명

하나의 일을 잘 수행하는 작은 크레이트를 선호하세요. Cargo와 crates.io 덕분에 Rust에서는 C나 C++ 같은 언어에 비해 외부 라이브러리를 프로젝트에 추가하고 관리하기가 쉽습니다. 따라서 큰 라이브러리에 여러 기능을 몰아넣기보다, 책임이 명확하고 조합하기 쉬운 크레이트를 활용할 수 있습니다.

crates.io에 공개된 특정 버전의 패키지 소스는 게시 후 임의로 수정할 수 없습니다. 이러한 특성은 버전 관리와 함께 사용될 때 의존성의 재현성을 높이는 데 도움이 됩니다. 다만 “한 번 빌드되면 언제나 그대로 빌드된다”는 뜻은 아닙니다. 의존성 버전 범위, 애플리케이션의 `Cargo.lock`, Rust 도구 체인과 빌드 환경도 함께 고려해야 합니다.

여기서 크레이트(crate)는 Rust 컴파일 단위인 라이브러리 또는 실행 프로그램을 뜻합니다. Cargo 패키지는 `Cargo.toml`로 관리되는 배포·빌드 단위이며, 하나의 패키지가 여러 크레이트를 포함할 수 있습니다. 일상 대화에서는 패키지와 크레이트를 느슨하게 같은 뜻으로 부르기도 하지만, 설계나 Cargo 설정을 설명할 때는 구별하는 편이 정확합니다.

## 작은 라이브러리 크레이트 예제

다음 예제는 여러 패키지를 하나의 Cargo 워크스페이스에서 함께 관리하면서, 공백으로 구분된 토큰의 개수를 세는 기능을 작은 라이브러리 크레이트로 분리합니다. `catalog` 애플리케이션은 이 기능을 경로 의존성으로 가져다 씁니다.

### 디렉터리 구조

```text
small-crates/
├── Cargo.toml
├── apps/
│   └── catalog/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── crates/
    └── text-count/
        ├── Cargo.toml
        ├── src/
        │   └── lib.rs
        └── tests/
            └── count_tokens.rs
```

### 워크스페이스 매니페스트

루트 `Cargo.toml`은 워크스페이스의 멤버 패키지를 선언합니다. 워크스페이스의 패키지들은 공통 `Cargo.lock`과 빌드 출력 디렉터리를 사용하며, Cargo 명령으로 함께 검사하고 테스트할 수 있습니다.

```toml
[workspace]
members = ["crates/text-count", "apps/catalog"]
resolver = "3"
```

### `text-count` 라이브러리 크레이트

`crates/text-count/Cargo.toml`:

```toml
[package]
name = "text-count"
version = "0.1.0"
edition = "2024"

[lib]
name = "text_count"
path = "src/lib.rs"
```

`crates/text-count/src/lib.rs`:

```rust
/// 공백으로 나뉜 토큰의 개수를 반환합니다.
pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::count_tokens;

    #[test]
    fn empty_or_whitespace_input_has_no_tokens() {
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens(" \t\n "), 0);
    }

    #[test]
    fn counts_tokens_separated_by_whitespace() {
        assert_eq!(count_tokens("Rust makes small crates"), 4);
    }
}
```

`split_whitespace`를 사용하므로 이 함수는 자연어의 형태소나 문장 부호를 분석하지 않습니다. 연속된 공백 문자를 구분자로 삼아 토큰 수를 세는 작은 기능만 담당합니다. 라이브러리 내부 단위 테스트는 구현 가까이에서 빈 입력과 일반 입력을 검사합니다.

### 공개 API를 확인하는 통합 테스트

`crates/text-count/tests/count_tokens.rs`:

```rust
use text_count::count_tokens;

#[test]
fn public_function_counts_whitespace_delimited_tokens() {
    assert_eq!(count_tokens("one  two\tthree\n"), 3);
}
```

통합 테스트는 라이브러리 외부에서 공개 API를 사용하는 관점으로 실행됩니다. 외부 테스트 크레이트에서는 내부 구현에 접근하지 않고 `text_count::count_tokens`를 가져와 호출하므로, 함수가 실제 사용자에게 노출되는 형태를 확인할 수 있습니다.

### 다른 패키지에서 재사용하기

`apps/catalog/Cargo.toml`:

```toml
[package]
name = "catalog"
version = "0.1.0"
edition = "2024"

[dependencies]
text-count = { path = "../../crates/text-count" }
```

`apps/catalog/src/main.rs`:

```rust
use text_count::count_tokens;

fn main() {
    let description = "Rust crates can be small and focused";
    println!("토큰 수: {}", count_tokens(description));
}
```

이 예제의 작은 라이브러리는 특정 애플리케이션에 묶이지 않고 다른 패키지에서도 호출할 수 있습니다. 워크스페이스 루트에서 다음 명령을 실행하면 모든 멤버 패키지의 단위 테스트와 통합 테스트를 실행하고 애플리케이션을 시작할 수 있습니다.

```console
cargo test --workspace
cargo run -p catalog
```

작은 크레이트가 유용한 이유는 단순히 파일을 잘게 쪼개기 위해서가 아닙니다. `text-count`처럼 기능이 응집된 경계를 정의하면 API와 책임을 작게 유지하고, 다른 프로그램이 그 기능을 독립적으로 재사용할 수 있습니다. 반대로 한두 줄짜리 내부 함수를 모두 별도의 크레이트로 만드는 것은 빌드·버전·의존성 관리 비용만 키울 수 있습니다.

## 장점

- **이해하기 쉽습니다.** 크레이트가 한 가지 역할에 집중하면 유지보수자가 그 목적과 API를 파악하기 쉽습니다.
- **모듈성을 촉진합니다.** 책임이 분리되면 애플리케이션 코드가 특정 기능의 구현 세부 사항에 덜 의존하게 됩니다.
- **재사용할 수 있습니다.** 공통 기능을 다른 프로젝트에서 의존성으로 가져와 반복 구현을 줄일 수 있습니다. 예를 들어 `url` 크레이트는 Servo 브라우저 엔진에서 개발되었고, 이후 다른 Rust 프로젝트에서도 URL 처리에 널리 쓰이게 되었습니다.
- **빌드를 병렬화할 수 있습니다.** Rust의 크레이트는 컴파일 단위이므로, 의존 관계가 허용하는 범위에서 서로 독립된 크레이트를 병렬로 빌드할 수 있습니다. 실제 빌드 시간은 의존성 그래프, 캐시, 작업량에 따라 달라집니다.

## 단점과 주의점

- **의존성 충돌이 생길 수 있습니다.** 한 애플리케이션이 같은 크레이트의 서로 다른 호환되지 않는 버전을 동시에 사용하면, 각 버전에서 정의한 타입은 이름이 같아도 서로 다른 타입입니다. 예를 들어 `url`의 서로 다른 주요 버전을 사용하는 두 라이브러리 사이에서 `Url` 값을 직접 전달할 수 없는 상황이 생길 수 있습니다. 이는 의존성 버전 선택을 어렵게 만드는 이른바 “dependency hell”의 한 사례입니다.
- **외부 크레이트의 품질을 확인해야 합니다.** crates.io에 게시되어 있다는 사실만으로 코드 품질, 문서, 유지보수 상태, 라이선스 또는 보안이 보장되지는 않습니다. 직접 의존하는 크레이트와 그 전이 의존성을 검토하세요.
- **최적화와 빌드 비용을 고려해야 합니다.** 크레이트를 나누면 각 컴파일 단위의 경계가 생깁니다. Cargo의 기본 설정에서 링크 타임 최적화(LTO)를 사용하지 않는 빌드는 크레이트 경계를 넘는 최적화를 충분히 하지 못할 수 있습니다. LTO를 사용하도록 설정할 수 있지만 빌드 시간과 실행 성능 사이의 균형을 측정해야 합니다.
- **경계를 지나치게 잘게 나누지 마세요.** 작은 크레이트마다 공개 API, 버전 관리, 문서화와 테스트가 필요합니다. 응집도가 낮고 한 곳에서만 사용되는 코드를 분리하면 오히려 구조를 이해하고 변경하기 어려워질 수 있습니다.

## 원문에 나온 크레이트 예시

- [`url`](https://crates.io/crates/url): URL을 구문 분석하고 다루는 기능을 제공합니다.
- [`num_cpus`](https://crates.io/crates/num_cpus): 시스템의 CPU 수를 조회하는 함수를 제공합니다.
- [`ref_slice`](https://crates.io/crates/ref_slice): `&T`를 `&[T]`로 변환하는 기능을 제공한 예시입니다. 원문에 인용된 과거 사례이며, 현재 프로젝트에 도입하기 전에는 유지보수 상태와 필요성을 확인하세요.

## 참고 자료

- [Rust Design Patterns 원문: Prefer small crates](https://rust-unofficial.github.io/patterns/patterns/structural/small-crates.html)
- [Cargo 문서: 패키지와 크레이트 용어 설명](https://doc.rust-lang.org/cargo/appendix/glossary.html)
- [Cargo 문서: 워크스페이스](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Cargo 문서: 의존성 지정과 버전 요구 사항](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Cargo 문서: `cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Cargo 문서: 빌드 프로필과 최적화 설정](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [crates.io: Rust 커뮤니티의 크레이트 레지스트리](https://crates.io/)
