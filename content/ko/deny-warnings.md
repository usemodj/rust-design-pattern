# `#![deny(warnings)]`

## 설명

크레이트 작성자는 코드에 경고가 없는 상태를 유지하고 싶을 수 있습니다. 이때 크레이트 루트에 `#![deny(warnings)]`를 추가하면 현재 크레이트에 발생하는 모든 린트 경고를 오류로 처리할 수 있습니다.

```rust,ignore
#![allow(unused)]
#![deny(warnings)]

fn main() {
    // 경고가 없다면 정상적으로 빌드됩니다.
}
```

이 설정은 짧고, 경고가 발생하면 빌드를 중단시킵니다. 다만 “오늘 경고가 없는 코드”와 “앞으로도 변경 없이 빌드되는 코드”는 같은 뜻이 아닙니다. 라이브러리 크레이트의 소스에 이 정책을 고정하면 Rust 컴파일러나 린트 도구의 변화가 사용자의 빌드 실패로 이어질 수 있습니다.

## 장점

- 설정이 한 줄로 간단합니다.
- 현재 경고를 오류로 처리하므로, 문제가 있는 상태로 빌드가 진행되는 일을 막을 수 있습니다.

## 단점

Rust는 일부 새 진단이나 기존 코드의 문제를 처음에는 경고로 알리고, 개발자가 수정할 시간을 준 뒤 나중에 더 엄격한 오류로 바꿀 수 있습니다. `#![deny(warnings)]`는 이 완화 기간(grace period)도 빌드 실패로 바꿉니다.

원문은 같은 메서드를 가진 중복 inherent `impl`이 발견된 사례를 설명합니다. 해당 코드를 문제로 판단했을 때, 먼저 린트 경고로 사용자에게 알리고 이후 더 엄격하게 처리하는 방식으로 전환할 수 있습니다. 이런 린트 수준 변화에 모든 경고를 거부하는 크레이트는 영향을 받을 수 있습니다.

기존 API가 폐기 예정(deprecated)으로 지정되는 경우에도 새 경고가 발생할 수 있습니다. 코드를 아직 변경하지 않았더라도 새 컴파일러나 의존 도구를 사용하는 순간 빌드가 중단될 수 있습니다. 즉 이 설정은 경고를 없애는 정책인 동시에, 컴파일러 및 도구 버전 변화에 민감한 빌드 정책이기도 합니다.

Clippy 같은 추가 린트 도구를 함께 실행할 때도 주의해야 합니다. Clippy가 제공하는 경고까지 모두 오류로 승격하면, 의도적으로 검토하려던 새 린트가 즉시 실패를 일으킬 수 있습니다. 린트 수준을 상한으로 제한하는 `--cap-lints` 옵션을 사용할 수 있으며, 예를 들어 `--cap-lints=warn`은 `deny` 수준 린트도 경고로 제한합니다. 이 방식은 임시 호환성 조치이지, 각 경고를 실제로 검토하는 일을 대신하지는 않습니다.

## 대안 1: 소스 코드와 빌드 정책 분리하기

개발자나 CI에서 빌드 플래그로 경고를 오류로 처리하면 소스 코드에 정책을 고정하지 않고 엄격한 검사를 수행할 수 있습니다. 원문은 다음 Unix 계열 셸 명령을 예로 듭니다.

```console
RUSTFLAGS="-D warnings" cargo build
```

Windows PowerShell에서는 환경 변수를 다음처럼 설정할 수 있습니다.

```powershell
$env:RUSTFLAGS = "-D warnings"
cargo build
Remove-Item Env:RUSTFLAGS
```

이 설정은 개인 개발 환경이나 CI에서 적용할 수 있고, 라이브러리 소스의 lint attribute를 수정하지 않아도 됩니다. 다만 CI에서 사용하는 Rust 및 Clippy 버전을 올릴 때 새 경고가 생길 수 있으므로, 도구 체인 업데이트와 빌드 정책을 함께 관리해야 합니다. `RUSTFLAGS`는 Cargo가 실행하는 여러 rustc 호출에 영향을 줄 수 있으므로 워크스페이스와 의존성 전체에 미치는 효과도 확인하세요.

## 대안 2: 프로젝트가 책임질 린트만 지정하기

모든 향후 경고를 한꺼번에 거부하는 대신, 프로젝트가 실제로 강제하려는 린트를 이름으로 지정할 수 있습니다. 예를 들어 공개 라이브러리가 공개 항목의 문서와 `Debug` 구현을 요구한다면 다음과 같이 정책을 표현할 수 있습니다.

```rust,ignore
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
```

이 방식은 새로 추가되는 모든 린트를 자동으로 오류로 취급하지 않고, 팀이 선택한 품질 기준을 코드에 명시합니다. 린트의 이름과 기본 수준은 Rust 버전마다 달라질 수 있으므로 현재 사용 중인 컴파일러에서 `rustc -W help` 또는 공식 린트 목록으로 확인하세요.

원문은 `deprecated` 린트를 예시 목록에 넣지 않았습니다. 앞으로 폐기 예정 API가 추가될 수 있으므로, 이를 일괄 거부하면 컴파일러나 의존성 변경으로 경고가 추가될 때 빌드가 깨질 수 있기 때문입니다.

## 원문에 제시된 린트 목록

아래 목록은 **원문이 Rust 1.48.0 기준으로 소개한 예시**입니다. 일부 린트는 현재 이름이 바뀌었거나 제거되었을 수 있으며, 프로젝트에 그대로 복사하지 마세요. 현재 컴파일러가 인식하는 이름과 수준을 확인한 뒤 필요한 항목만 선택해야 합니다.

```rust,ignore
#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
```

원문은 기본 수준이 `allow`인 린트 중에서도 라이브러리에 유용할 수 있는 항목을 별도로 제안합니다.

```rust,ignore
#![deny(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
```

필요에 따라 `missing_copy_implementations`도 추가할 수 있다고 원문은 언급합니다. 이 역시 역사적 예시이므로 현재 컴파일러에서 해당 린트가 유효한지, 팀이 실제로 원하는 정책인지 확인하세요. 모든 린트를 켜는 것이 항상 더 좋은 품질을 뜻하지는 않습니다.

## 적용할 때 고려할 점

- **애플리케이션과 라이브러리의 요구를 구분합니다.** 애플리케이션 팀은 특정 CI 환경에서 경고 없는 빌드를 강제하고 싶을 수 있습니다. 반면 공개 라이브러리는 다른 사용자의 컴파일러 버전과 린트 설정에 영향을 줄 수 있으므로 소스에 광범위한 정책을 고정할 때 더 신중해야 합니다.
- **컴파일러 업데이트를 따로 검증합니다.** Rust 버전을 갱신할 때 새 경고가 생길 수 있으므로, 경고 정책이 엄격하다면 도구 체인 업데이트를 별도 변경으로 검토하고 진단 원인을 확인하세요.
- **경고를 무작정 억제하거나 무조건 거부하지 않습니다.** 경고가 실제 오류 가능성을 가리키는지, 의도된 코드인지, 선택한 린트가 프로젝트에 적합한지 각각 판단합니다.

## 참고 자료

- [Rust Design Patterns 원문: `#![deny(warnings)]`](https://rust-unofficial.github.io/patterns/anti_patterns/deny-warnings.html)
- [Rust 컴파일러 문서: 린트 수준](https://doc.rust-lang.org/rustc/lints/levels.html)
- [Rust 컴파일러 문서: 기본적으로 허용되는 린트 목록](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html)
- [Clippy 문서](https://doc.rust-lang.org/clippy/)
- [Cargo 문서: 환경 변수](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- [Cargo 문서: 설정 파일](https://doc.rust-lang.org/cargo/reference/config.html)
