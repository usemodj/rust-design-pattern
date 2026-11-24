# Rust의 함수형 기능

Rust는 함수형 프로그래밍 언어 그 자체는 아니지만, 함수형 언어에서 발전한 아이디어를 타입 시스템과 여러 표현 방식에 활용합니다. 이 장에서는 프로그래밍 패러다임, 제네릭과 트레이트, 함수형 옵틱스를 다룹니다.

Rust의 제네릭은 컴파일 시점에 타입별 코드로 구체화되는 모노모피제이션을 사용합니다. 트레이트 바운드는 해당 타입이 제공해야 하는 동작을 표현하며, 런타임에 임의로 타입의 메서드를 추가하는 방식이 아닙니다.

불변 데이터를 변환하는 반복자와 `Option`·`Result` 같은 열거형도 함수형 스타일을 지원합니다. 그렇다고 가변 상태나 반복문을 피해야 한다는 뜻은 아닙니다. 문제에 맞는 표현을 고르고, 코드의 흐름과 오류 처리를 쉽게 이해할 수 있도록 하는 것이 우선입니다.

## 주요 구성 요소

- **함수와 클로저:** 동작을 값으로 전달하고, 클로저는 필요한 주변 값을 캡처할 수 있습니다.
- **반복자:** `map`, `filter`, `fold` 등으로 데이터 변환을 조합합니다. 대부분의 어댑터는 소비되기 전까지 지연 평가됩니다.
- **열거형과 패턴 매칭:** `Option`, `Result`, `match`로 값의 부재와 실패를 명시적으로 처리합니다.
- **제네릭과 트레이트:** 타입에 필요한 동작을 제약으로 표현하고 여러 타입에 같은 알고리즘을 적용합니다.
- **소유권과 빌림:** 데이터의 수명과 변경 가능 범위를 명시합니다. 소유권을 사용한다고 해서 함수가 자동으로 순수해지는 것은 아닙니다.

## 예제: 입력 변환과 오류 전달

다음 예제는 문자열을 정수로 변환하고 양수만 모읍니다. 유효하지 않은 입력을 조용히 건너뛰지 않고 첫 파싱 오류를 호출자에게 돌려줍니다.

`src/lib.rs`:

```rust
use std::num::ParseIntError;

pub fn positive_numbers(inputs: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    let numbers: Vec<i32> = inputs.iter().map(|text| text.parse()).collect::<Result<_, _>>()?;
    Ok(numbers.into_iter().filter(|number| *number > 0).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_positive_values_in_order() {
        assert_eq!(positive_numbers(&["3", "-1", "2"]).unwrap(), vec![3, 2]);
        assert!(positive_numbers(&[]).unwrap().is_empty());
    }

    #[test]
    fn invalid_input_is_an_error() {
        assert!(positive_numbers(&["1", "bad", "2"]).is_err());
    }
}
```

`collect::<Result<_, _>>()`는 첫 오류에서 수집을 중단합니다. 위 코드는 전체 파싱이 성공한 뒤 필터링하므로 중간 벡터를 만듭니다. 대용량 입력에서 중간 할당이 문제라면 처리 방식을 측정하고 조정할 수 있습니다. 예제를 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 `cargo test`로 테스트할 수 있습니다.

## 적용할 때 살펴볼 점

변환의 입출력과 부수 효과를 분리하면 테스트와 재사용이 쉬워집니다. 다만 긴 반복자 체인이나 지나치게 복잡한 타입 제약은 코드를 읽기 어렵게 할 수 있습니다. 성능도 사용한 추상화, 데이터와 컴파일러 최적화에 따라 달라지므로 필요한 경우 측정하세요.

## 하위 문서

- [프로그래밍 패러다임](/docs/functional-programming)
- [타입 클래스로서의 제네릭](/docs/generics-as-type-classes)
- [함수형 옵틱스](/docs/functional-optics)

## 참고 자료

- [Rust Book: 클로저](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Rust Book: 이터레이터](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Rust 표준 라이브러리: `Result`](https://doc.rust-lang.org/std/result/)
