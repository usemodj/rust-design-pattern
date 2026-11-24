# `format!`으로 문자열 연결하기

가변 `String`의 `push`와 `push_str`, 또는 `+` 연산자로 문자열을 조합할 수 있습니다. 리터럴과 변수 값이 섞인 문자열을 만들 때는 `format!`이 간결하고 읽기 쉽습니다.

```rust
fn say_hello(name: &str) -> String {
    format!("Hello {name}!")
}
```

`format!`은 새 `String`을 만들어 반환하므로 일반적으로 여러 번 `push`하는 방식보다 효율적이지 않을 수 있습니다. 예상 길이를 알고 있고 성능이 중요하다면 미리 용량을 확보한 뒤 가변 문자열에 이어 붙이는 방법을 고려하세요.

성능 차이가 실제로 중요한지는 측정으로 확인하고, 기본적으로는 의도가 잘 드러나는 표현을 선택하세요.

## 원문 소스 코드 예제

### 예제 1

```rust
fn say_hello(name: &str) -> String {
    // We could construct the result string manually.
    // let mut result = "Hello ".to_owned();
    // result.push_str(name);
    // result.push('!');
    // result

    // But using format! is better.
    format!("Hello {name}!")
}
```
