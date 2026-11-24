# Template Method 패턴

## 의도

Template Method(템플릿 메서드) 패턴은 알고리즘의 전체 처리 순서를 정하고, 그중 일부 단계를 다른 구현으로 바꿀 수 있도록 합니다. 공통 흐름은 한곳에서 유지하면서 입력 해석이나 결과 표현처럼 달라지는 부분만 분리합니다.

예를 들어 보고서를 만들 때 입력 검사, 데이터 해석, 합계 계산, 결과 출력의 순서는 같지만 입력과 출력 형식은 다를 수 있습니다.

## 구성 요소

- **템플릿:** 알고리즘의 공통 단계와 실행 순서를 정의합니다.
- **변경 가능한 단계:** 구현에 따라 달라지는 작업을 공통 인터페이스로 제공합니다.
- **구체적인 구현:** 각 단계를 실제 형식이나 정책에 맞게 구현합니다.
- **클라이언트:** 사용할 구현을 선택해 템플릿에 전달합니다.

전통적인 객체 지향 구현은 상위 클래스의 메서드가 전체 흐름을 정하고 하위 클래스가 일부 메서드를 재정의합니다. Rust에는 클래스 상속이 없으므로 트레이트, 제네릭, 함수와 합성으로 같은 목적을 표현할 수 있습니다.

## Rust에서 표현하기

트레이트의 기본 메서드에 공통 흐름을 둘 수 있지만, 구현자가 그 기본 메서드 자체를 재정의할 수도 있습니다. 공통 순서를 강제하고 싶다면 아래 예제처럼 **별도 함수**를 템플릿으로 두고 변경 가능한 단계만 트레이트에 선언할 수 있습니다. 호출자가 단계 메서드를 직접 호출할 수는 있지만, `create_report`를 통한 실행 순서는 구현자가 덮어쓸 수 없습니다.

고정된 흐름의 일부 단계를 바꾸는 Template Method와 달리 [Strategy](/docs/strategy)는 같은 목적을 위한 알고리즘 자체를 교체하는 데 초점을 둡니다. 두 방식은 함께 사용할 수도 있습니다.

## Rust 예제: 숫자 보고서 만들기

`create_report`는 입력 검사 → 숫자 해석 → 합계 계산 → 결과 형식화 순서로 실행합니다. `ReportFormat` 구현이 해석과 형식화를 담당합니다. 아래 예제의 CSV는 쉼표로 구분된 정수만 지원하는 간단한 형식이며, 따옴표나 이스케이프를 지원하는 완전한 CSV 파서는 아닙니다.

`src/lib.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ReportError {
    EmptyInput,
    InvalidNumber(String),
    SumOverflow,
}

pub trait ReportFormat {
    fn parse(&self, input: &str) -> Result<Vec<i32>, ReportError>;
    fn format(&self, count: usize, total: i64) -> String;
}

pub fn create_report(input: &str, format: &impl ReportFormat) -> Result<String, ReportError> {
    if input.trim().is_empty() {
        return Err(ReportError::EmptyInput);
    }
    let numbers = format.parse(input)?;
    let total = numbers.iter().try_fold(0_i64, |total, &number| {
        total.checked_add(i64::from(number)).ok_or(ReportError::SumOverflow)
    })?;
    Ok(format.format(numbers.len(), total))
}

fn parse_numbers<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Result<Vec<i32>, ReportError> {
    values
        .map(|value| {
            let value = value.trim();
            value.parse::<i32>().map_err(|_| ReportError::InvalidNumber(value.to_owned()))
        })
        .collect()
}

pub struct CommaSeparated;

impl ReportFormat for CommaSeparated {
    fn parse(&self, input: &str) -> Result<Vec<i32>, ReportError> {
        parse_numbers(input.split(','))
    }

    fn format(&self, count: usize, total: i64) -> String {
        format!("개수: {count}, 합계: {total}")
    }
}

pub struct LineSeparated;

impl ReportFormat for LineSeparated {
    fn parse(&self, input: &str) -> Result<Vec<i32>, ReportError> {
        parse_numbers(input.lines())
    }

    fn format(&self, count: usize, total: i64) -> String {
        format!("개수={count}\n합계={total}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn comma_format_uses_the_common_workflow() {
        assert_eq!(
            create_report("10, -3, 5", &CommaSeparated),
            Ok("개수: 3, 합계: 12".to_owned())
        );
    }

    #[test]
    fn line_format_changes_the_steps_not_the_calculation() {
        assert_eq!(
            create_report("10\n-3\n5", &LineSeparated),
            Ok("개수=3\n합계=12".to_owned())
        );
    }

    #[test]
    fn invalid_numbers_are_reported() {
        assert_eq!(
            create_report("1,abc,3", &CommaSeparated),
            Err(ReportError::InvalidNumber("abc".to_owned()))
        );
        assert_eq!(
            create_report("1,,3", &CommaSeparated),
            Err(ReportError::InvalidNumber(String::new()))
        );
    }

    #[test]
    fn validation_and_parse_errors_stop_later_steps() {
        struct Probe {
            parsed: Cell<bool>,
            formatted: Cell<bool>,
        }
        impl ReportFormat for Probe {
            fn parse(&self, _: &str) -> Result<Vec<i32>, ReportError> {
                self.parsed.set(true);
                Err(ReportError::InvalidNumber("bad".to_owned()))
            }
            fn format(&self, _: usize, _: i64) -> String {
                self.formatted.set(true);
                String::new()
            }
        }

        let probe = Probe { parsed: Cell::new(false), formatted: Cell::new(false) };
        assert_eq!(create_report("  ", &probe), Err(ReportError::EmptyInput));
        assert!(!probe.parsed.get());
        assert!(!probe.formatted.get());

        assert_eq!(
            create_report("bad", &probe),
            Err(ReportError::InvalidNumber("bad".to_owned()))
        );
        assert!(probe.parsed.get());
        assert!(!probe.formatted.get());
    }
}
```

`create_report`가 공통 처리 순서를 소유하고, `CommaSeparated`와 `LineSeparated`가 달라지는 단계를 구현합니다. 입력이 비어 있거나 숫자 해석이 실패하면 오류를 반환하고 이후 단계를 실행하지 않습니다. 합계를 구할 때도 검사 덧셈으로 오버플로를 오류로 표현합니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 두 형식에서 동일한 계산 흐름을 사용하는 경우, 잘못된 숫자와 빈 필드, 빈 입력 및 해석 실패 시 이후 단계가 실행되지 않는 경우를 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 공통 처리 순서와 중복 로직을 한곳에서 관리할 수 있습니다.
- 변하는 단계를 분리해 다른 형식이나 정책을 추가하기 쉽습니다.
- 오류 발생 시 이후 단계를 중단하는 규칙을 공통 흐름에 둘 수 있습니다.

### 단점

- 변형마다 처리 순서가 달라야 한다면 하나의 템플릿이 오히려 제약이 될 수 있습니다.
- 단계 인터페이스가 너무 세분화되거나 구현별 예외가 많으면 구조가 복잡해집니다.
- 기본 트레이트 메서드를 템플릿으로 사용하면 구현자가 그 메서드를 재정의할 수 있습니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Template Method 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/template-method)
- [Template method 패턴 — 위키백과](https://en.wikipedia.org/wiki/Template_method_pattern)
- [Rust Book: 트레이트와 기본 구현](https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations)
- [Rust Book: `Result`를 이용한 오류 처리](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
