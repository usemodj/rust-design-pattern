# 오류 시 소비된 인자 돌려주기

## 설명

실패할 수 있는 함수가 인자를 소비해 소유권을 가져간다면, 실패했을 때 그 인자를 오류 값 안에 담아 호출자에게 돌려줄 수 있습니다. 그러면 성공 시에는 인자를 함수에 넘기고, 실패 시에는 값을 회수해 다른 방법을 시도하거나 다시 호출할 수 있습니다.

## 예제

아래 `send` 함수는 `String`의 소유권을 인자로 받습니다. 작업에 성공하면 값은 소비되고, 실패하면 `SendError` 안에 원래 `String`을 담아 반환합니다. `main`은 실패한 경우 오류에서 문자열을 꺼내 `value`에 다시 넣고 최대 두 번까지 전송을 시도합니다.

```rust
pub fn send(value: String) -> Result<(), SendError> {
    println!("using {value} in a meaningful way");
    // Simulate non-deterministic fallible action.
    use std::time::SystemTime;
    let period = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    if period.subsec_nanos() % 2 == 1 {
        Ok(())
    } else {
        Err(SendError(value))
    }
}

pub struct SendError(String);

fn main() {
    let mut value = "imagine this is very long string".to_string();

    let success = 's: {
        // Try to send value two times.
        for _ in 0..2 {
            value = match send(value) {
                Ok(()) => break 's true,
                Err(SendError(value)) => value,
            }
        }
        false
    };

    println!("success: {success}");
}
```

`send(value)`를 호출하면 `value`의 소유권이 함수로 이동합니다. 성공하면 더 이상 원래 값이 필요하지 않으므로 `Ok(())`를 반환합니다. 실패하면 `Err(SendError(value))`로 감싸 돌려줍니다. 호출자는 오류에서 값을 꺼내 다시 `value`에 저장하므로 다음 시도에서도 복제본을 만들 필요가 없습니다.

## 동기

오류가 발생하면 다른 방법으로 처리하거나, 결과가 비결정적인 작업을 다시 시도하고 싶을 수 있습니다. 하지만 함수가 매번 인자를 소비하고 실패 시에도 돌려주지 않는다면 재시도하기 위해 호출 전에 인자를 복제해야 합니다. 특히 큰 값을 매번 복제하면 비효율적입니다.

표준 라이브러리의 `String::from_utf8`도 이 방식을 사용합니다. 입력한 바이트 벡터가 유효한 UTF-8이 아니면 `FromUtf8Error`를 반환하며, [`FromUtf8Error::into_bytes`](https://doc.rust-lang.org/std/string/struct.FromUtf8Error.html#method.into_bytes)를 호출해 원래 벡터를 되찾을 수 있습니다.

## 장점

가능한 경우 인자를 복제하지 않고 소유권을 이동해 처리하므로 성능을 높일 수 있습니다.

## 단점

실패한 인자를 보관해야 하므로 오류 타입이 조금 더 복잡해집니다.
