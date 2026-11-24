# FFI로 문자열 전달하기

## 설명

문자열을 FFI 함수에 전달할 때는 다음 네 가지 원칙을 따르는 것이 좋습니다.

1. 소유한 문자열의 수명을 가능한 한 길게 유지합니다.
2. 문자열 변환 과정의 `unsafe` 코드를 최소화합니다.
3. C 코드가 문자열 데이터를 수정할 수 있다면 `CString` 대신 `Vec`를 사용합니다.
4. 외부 함수 API에서 요구하지 않는 한 문자열의 소유권을 호출된 함수로 넘기지 않습니다.

## 동기

Rust는 C 스타일 문자열을 다루기 위한 `CString`과 `CStr` 타입을 제공합니다. Rust 함수에서 외부 함수를 호출할 때 문자열을 전달하는 방법은 여러 가지가 있지만, 기본 원칙은 `CString`을 사용하면서 `unsafe` 코드를 최소화하는 것입니다.

또 하나 주의할 점은 문자열을 담은 객체가 충분히 오래 살아 있어야 한다는 것입니다. 포인터를 만들어도 원래 문자열의 수명은 연장되지 않습니다. 또한 문서에서 설명하듯이, C 코드가 수정한 `CString`을 다시 Rust 문자열로 변환하는(round-trip) 작업은 정의되지 않은 동작을 일으킬 수 있으므로, 수정이 필요한 경우에는 추가적인 처리가 필요합니다.

## 코드 예제

아래 예제는 오류를 C 함수에 전달하고, C 함수가 채운 버퍼에서 오류 문자열을 가져오는 방법을 보여줍니다.

```rust,ignore
pub mod unsafe_module {

    // other module content

    extern "C" {
        fn seterr(message: *const libc::c_char);
        fn geterr(buffer: *mut libc::c_char, size: libc::c_int) -> libc::c_int;
    }

    fn report_error_to_ffi<S: Into<String>>(err: S) -> Result<(), std::ffi::NulError> {
        let c_err = std::ffi::CString::new(err.into())?;

        unsafe {
            // SAFETY: calling an FFI whose documentation says the pointer is
            // const, so no modification should occur
            seterr(c_err.as_ptr());
        }

        Ok(())
        // The lifetime of c_err continues until here
    }

    fn get_error_from_ffi() -> Result<String, std::ffi::IntoStringError> {
        let mut buffer = vec![0u8; 1024];
        unsafe {
            // SAFETY: calling an FFI whose documentation implies
            // that the input need only live as long as the call
            let written: usize = geterr(buffer.as_mut_ptr(), 1023).into();

            buffer.truncate(written + 1);
        }

        std::ffi::CString::new(buffer).unwrap().into_string()
    }
}
```

`report_error_to_ffi`는 Rust 문자열을 `CString`으로 변환합니다. 변환은 문자열 내부에 NUL 바이트가 있으면 `NulError`를 반환할 수 있으므로 `?`로 오류를 호출자에게 전달합니다. 그다음 `as_ptr()`로 얻은 포인터를 C 함수에 전달합니다. `c_err`를 이름 있는 지역 변수에 저장했기 때문에 외부 호출이 끝날 때까지 문자열이 살아 있습니다. 예제의 C 함수 인자는 읽기 전용 포인터이므로 호출 중 문자열을 수정하지 않는다는 계약을 전제로 합니다.

`get_error_from_ffi`는 C 함수가 쓸 수 있도록 1024바이트 버퍼를 만들고, 버퍼 포인터와 길이를 전달합니다. C 함수가 보고한 길이에 맞춰 벡터를 자른 뒤 NUL 종료 문자열과 Rust `String`으로 변환합니다. 이때 실제 반환 길이와 NUL 종료 여부는 외부 함수의 계약에 따라 유효해야 합니다.

## 장점

이 예제는 다음 사항을 지키도록 작성되었습니다.

1. `unsafe` 블록을 실제 외부 함수 호출과 포인터 사용에 필요한 범위로 제한합니다.
2. `CString`이 포인터를 사용하는 동안 계속 살아 있도록 합니다.
3. 형 변환 과정에서 발생할 수 있는 오류를 가능하면 반환값으로 전달합니다.

자주 발생하는 실수는 첫 번째 블록에서 `CString`을 변수에 저장하지 않는 것입니다.

```rust,ignore
pub mod unsafe_module {

    // other module content

    fn report_error<S: Into<String>>(err: S) -> Result<(), std::ffi::NulError> {
        unsafe {
            // SAFETY: whoops, this contains a dangling pointer!
            seterr(std::ffi::CString::new(err.into())?.as_ptr());
        }
        Ok(())
    }
}
```

이 코드는 댕글링 포인터를 만듭니다. 임시 `CString`은 `as_ptr()`로 포인터를 만들었다고 해서 수명이 연장되지 않습니다. 따라서 `seterr`가 호출된 뒤 임시 값이 파괴되면 전달한 포인터는 더 이상 유효하지 않습니다. 앞선 예제처럼 `CString`을 변수에 저장하고 포인터 사용이 끝날 때까지 그 변수를 살아 있게 해야 합니다.

또한 1KiB 크기의 벡터를 0으로 초기화하는 작업이 느릴 수 있다는 우려가 종종 제기됩니다. 원문에 따르면 최근 Rust 버전은 이 초기화 매크로를 `zmalloc` 호출로 최적화하므로, 운영체제가 0으로 초기화된 메모리를 반환하는 속도에 가까운 성능을 낼 수 있습니다.

## 단점

원문에서는 별도의 단점을 제시하지 않습니다.
