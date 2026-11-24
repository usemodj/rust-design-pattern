# 프로그래밍 패러다임

명령형 방식에서 함수형·선언형 방식으로 옮겨 갈 때 가장 큰 변화는 사고의 관점입니다. 명령형 프로그램은 **어떻게** 수행하는지 절차를 설명하고, 선언형 프로그램은 **무엇을** 원하는지 표현합니다. 1부터 10까지 더하는 예를 살펴봅시다.

## 명령형

```rust
let mut sum = 0;
for i in 1..11 {
    sum += i;
}
println!("{sum}");
```

이 코드는 `sum`을 0으로 시작하고, 1부터 10까지 반복하면서 현재 값을 누적하는 단계들을 지정합니다. 프로그램을 읽을 때 각 반복에서 상태가 어떻게 변하는지 따라가야 합니다.

## 선언형

```rust
println!("{}", (1..11).fold(0, |a, b| a + b));
```

여기서는 범위의 모든 값에 덧셈을 누적해 하나의 결과를 만든다는 점을 `fold`로 표현합니다. 시작값은 `0`이고, 각 단계에서 누적값 `a`와 다음 원소 `b`를 더해 새 누적값을 얻습니다. 마지막 값은 `55`입니다.

선언형 표현이 언제나 더 낫다는 뜻은 아닙니다. 데이터 변환을 체인으로 표현하면 의도가 선명해질 수 있지만, 익숙하지 않은 조합은 오히려 이해하기 어려울 수 있습니다. 문제를 가장 명확히 표현하는 방식을 선택하세요.

## 순수 함수와 부수 효과

순수 함수는 같은 입력에 대해 같은 결과를 반환하며 외부에서 관찰할 수 있는 상태를 변경하지 않습니다. 파일 쓰기, 네트워크 요청과 출력은 부수 효과의 예입니다. 프로그램에서 부수 효과를 없애야 한다는 뜻은 아닙니다. 계산과 입출력을 구분하면 계산 부분을 독립적으로 테스트하기 쉽습니다.

지역 변수의 가변성 자체는 외부 부수 효과와 같지 않습니다. 함수 안에서 누적 변수를 바꾸더라도 외부 상태에 의존하거나 외부 상태를 바꾸지 않는다면 순수한 계산일 수 있습니다.

## 예제: 양수 합계

같은 계산을 반복자와 `fold`로 표현합니다. 입력을 빌리므로 호출자는 계산 후에도 원본을 사용할 수 있습니다.

`src/lib.rs`:

```rust
pub fn positive_total(values: &[i32]) -> i64 {
    values.iter()
        .filter(|&&value| value > 0)
        .fold(0_i64, |total, &value| total + i64::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_only_positive_values() {
        let values = [5, -2, 0, 3];
        assert_eq!(positive_total(&values), 8);
        assert_eq!(values, [5, -2, 0, 3]);
    }

    #[test]
    fn empty_or_nonpositive_input_has_zero_total() {
        assert_eq!(positive_total(&[]), 0);
        assert_eq!(positive_total(&[-4, 0]), 0);
    }
}
```

`filter`와 `fold`는 계산을 구성하고 소비하지만 화면에 출력하지 않습니다. 결과를 `println!`으로 보여 주는 것은 호출자의 책임입니다. 실제 대량 입력에서 합계가 `i64` 범위를 넘을 수 있다면 `checked_add`와 `try_fold` 등으로 오버플로를 처리해야 합니다. 예제를 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 `cargo test`로 테스트할 수 있습니다.

## 참고 자료

- [Rust의 함수형 기능](/docs/functional-overview)
- [Rust Book: 이터레이터로 일련의 항목 처리하기](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [`Iterator::fold` 표준 라이브러리 문서](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)
