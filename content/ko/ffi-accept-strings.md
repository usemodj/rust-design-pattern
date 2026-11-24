# FFI에서 문자열 받기

## 설명

FFI를 통해 포인터로 문자열을 받을 때는 다음 두 원칙을 따르는 것이 좋습니다.

1. 외부에서 받은 문자열을 바로 복사하지 말고 빌린 상태로 사용합니다.
2. C 문자열을 Rust 문자열로 바꾸는 과정의 복잡성과 `unsafe` 코드 양을 최소화합니다.

## 동기

C 문자열과 Rust 문자열은 표현과 사용 방식이 다릅니다.

- C 문자열은 널 바이트(`NUL`)로 끝나지만, Rust 문자열은 길이를 별도로 저장합니다.
- C 문자열은 0이 아닌 임의의 바이트를 담을 수 있지만, Rust 문자열은 유효한 UTF-8이어야 합니다.
- C 문자열은 `unsafe` 포인터 연산으로 접근하고 조작하지만, Rust 문자열은 안전한 메서드를 통해 다룹니다.

Rust 표준 라이브러리는 Rust의 `String`과 `&str`에 대응하는 C 문자열 타입으로 `CString`과 `&CStr`을 제공합니다. 이 타입들을 사용하면 두 문자열 표현 사이를 변환할 때 필요한 복잡성과 `unsafe` 코드를 줄일 수 있습니다.

`&CStr`은 빌린 데이터를 다루므로 문자열을 Rust와 C 사이에서 전달할 때 복사나 할당 없이 사용할 수 있습니다. 다만 외부 포인터를 `CStr`로 해석할 때는 포인터의 유효성과 문자열의 수명을 호출자 계약으로 보장해야 합니다.

## 코드 예제

아래 함수는 로그 메시지를 C 문자열 포인터로 받습니다. 호출자는 `msg`가 널 포인터가 아니고, 유효하게 초기화된 데이터를 가리키며, 널 바이트로 끝나고, 함수 실행 중 변경되지 않음을 보장해야 합니다. 이 전제 아래 `CStr::from_ptr`로 빌린 C 문자열을 얻고, `to_str`로 UTF-8 변환을 시도합니다. 메시지가 유효한 UTF-8이 아니면 오류를 기록하고 반환합니다.

```rust,ignore
pub mod unsafe_module {

    // other module content

    /// Log a message at the specified level.
    ///
    /// # Safety
    ///
    /// It is the caller's guarantee to ensure `msg`:
    ///
    /// - is not a null pointer
    /// - points to valid, initialized data
    /// - points to memory ending in a null byte
    /// - won't be mutated for the duration of this function call
    #[no_mangle]
    pub unsafe extern "C" fn mylib_log(msg: *const libc::c_char, level: libc::c_int) {
        let level: crate::LogLevel = match level { /* ... */ };

        // SAFETY: The caller has already guaranteed this is okay (see the
        // `# Safety` section of the doc-comment).
        let msg_str: &str = match std::ffi::CStr::from_ptr(msg).to_str() {
            Ok(s) => s,
            Err(e) => {
                crate::log_error("FFI string conversion failed");
                return;
            }
        };

        crate::log(msg_str, level);
    }
}
```

## 장점

이 예제는 다음 두 가지를 보장하도록 작성되었습니다.

1. `unsafe` 블록을 가능한 한 작게 유지합니다.
2. 수명이 추적되지 않는 원시 포인터를 수명이 추적되는 공유 참조로 바꿉니다.

문자열을 실제로 복사하는 대안을 살펴보겠습니다.

```rust,ignore
pub mod unsafe_module {

    // other module content

    pub extern "C" fn mylib_log(msg: *const libc::c_char, level: libc::c_int) {
        // DO NOT USE THIS CODE.
        // IT IS UGLY, VERBOSE, AND CONTAINS A SUBTLE BUG.

        let level: crate::LogLevel = match level { /* ... */ };

        let msg_len = unsafe { /* SAFETY: strlen is what it is, I guess? */
            libc::strlen(msg)
        };

        let mut msg_data = Vec::with_capacity(msg_len + 1);

        let msg_cstr: std::ffi::CString = unsafe {
            // SAFETY: copying from a foreign pointer expected to live
            // for the entire stack frame into owned memory
            std::ptr::copy_nonoverlapping(msg, msg_data.as_mut(), msg_len);

            msg_data.set_len(msg_len + 1);

            std::ffi::CString::from_vec_with_nul(msg_data).unwrap()
        }

        let msg_str: String = unsafe {
            match msg_cstr.into_string() {
                Ok(s) => s,
                Err(e) => {
                    crate::log_error("FFI string conversion failed");
                    return;
                }
            }
        };

        crate::log(&msg_str, level);
    }
}
```

이 코드는 권장 예제보다 두 가지 점에서 열등합니다.

1. `unsafe` 코드가 훨씬 많고, 그 코드가 지켜야 하는 불변 조건도 더 많습니다.
2. 복잡한 길이 계산과 포인터 연산 때문에 Rust에서 정의되지 않은 동작(undefined behavior)을 일으키는 버그가 있습니다.

구체적으로 `strlen`은 문자열의 길이를 널 종료 바이트를 제외하고 반환합니다. 위 코드는 `msg_len` 바이트만 복사하므로 마지막 `NUL` 바이트는 복사되지 않습니다. 그런데 `Vec`의 길이를 `msg_len + 1`로 설정해 마지막 바이트까지 초기화된 것처럼 표시합니다. 용량만 `msg_len + 1`로 확보했을 뿐 그 마지막 바이트에 값을 기록하지 않았으므로, 벡터의 마지막 바이트는 초기화되지 않은 메모리입니다. 이어서 `CString::from_vec_with_nul`이 그 벡터를 읽으면 정의되지 않은 동작이 발생합니다.

길이와 포인터를 다루는 이런 문제는 찾아내기 어렵습니다. 실행 환경과 입력에 따라 UTF-8 변환 오류처럼 보이거나, 문자열 끝에 이상한 문자가 붙거나, 프로그램이 갑자기 충돌하는 등 증상이 달라질 수 있습니다. 이는 정의되지 않은 동작의 결과가 예측 가능하지 않기 때문입니다.

## 단점

원문에서는 이 방식의 별도 단점을 제시하지 않습니다.
