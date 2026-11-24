# FFI에서 오류 처리하기

## 설명

C 같은 외부 언어에서는 오류를 보통 함수의 반환 코드로 표현합니다. Rust의 타입 시스템은 더 풍부한 오류 정보를 하나의 타입에 담아 전달할 수 있으므로, 이 정보를 외부 언어에서도 사용하기 좋은 형태로 변환해야 합니다.

오류 타입의 구조에 따라 다음과 같은 표현 방식을 사용할 수 있습니다.

1. 단순 열거형(flat enum)은 정수로 변환해 오류 코드로 반환합니다.
2. 구조화된 열거형(structured enum)은 정수 오류 코드로 오류 종류를 나타내고, 문자열 메시지로 구체적인 내용을 전달합니다.
3. 사용자 정의 오류 타입(custom error type)은 C 표현을 가진 별도의 타입으로 변환해 외부에 노출합니다.

## 코드 예제

### 단순 열거형

오류 종류가 고정되어 있고 추가 데이터가 필요하지 않다면 각 변형에 정수 코드를 연결할 수 있습니다. C 호출자는 이 코드를 확인해 적절한 대응을 할 수 있습니다.

```rust,ignore
enum DatabaseError {
    IsReadOnly = 1,    // user attempted a write operation
    IOError = 2,       // user should read the C errno() for what it was
    FileCorrupted = 3, // user should run a repair tool to recover it
}

impl From<DatabaseError> for libc::c_int {
    fn from(e: DatabaseError) -> libc::c_int {
        (e as i8).into()
    }
}
```

### 구조화된 열거형

오류 변형이 추가 정보를 담는다면 오류 코드는 오류의 종류를 나타내고, 별도의 설명 함수가 자세한 메시지를 제공하도록 구성할 수 있습니다. 아래 예제에서는 읽기 전용 데이터베이스에 쓰려는 경우, I/O 오류, 파일 손상에 서로 다른 코드를 부여합니다. 설명 함수는 각 변형의 세부 정보를 문자열로 만들고, C 호출자가 사용할 수 있도록 널 종료된 문자열 버퍼로 복사합니다.

```rust,ignore
pub mod errors {
    enum DatabaseError {
        IsReadOnly,
        IOError(std::io::Error),
        FileCorrupted(String), // message describing the issue
    }

    impl From<DatabaseError> for libc::c_int {
        fn from(e: DatabaseError) -> libc::c_int {
            match e {
                DatabaseError::IsReadOnly => 1,
                DatabaseError::IOError(_) => 2,
                DatabaseError::FileCorrupted(_) => 3,
            }
        }
    }
}

pub mod c_api {
    use super::errors::DatabaseError;
    use core::ptr;

    #[no_mangle]
    pub extern "C" fn db_error_description(
        e: Option<ptr::NonNull<DatabaseError>>,
    ) -> Option<ptr::NonNull<libc::c_char>> {
        // SAFETY: we assume that the lifetime of `e` is greater than
        // the current stack frame.
        let error = unsafe { e?.as_ref() };

        let error_str: String = match error {
            DatabaseError::IsReadOnly => {
                format!("cannot write to read-only database")
            }
            DatabaseError::IOError(e) => {
                format!("I/O Error: {e}")
            }
            DatabaseError::FileCorrupted(s) => {
                format!("File corrupted, run repair: {}", &s)
            }
        };

        let error_bytes = error_str.as_bytes();

        let c_error = unsafe {
            // SAFETY: copying error_bytes to an allocated buffer with a '\0'
            // byte at the end.
            let buffer = ptr::NonNull::<u8>::new(libc::malloc(error_bytes.len() + 1).cast())?;

            buffer
                .as_ptr()
                .copy_from_nonoverlapping(error_bytes.as_ptr(), error_bytes.len());
            buffer.as_ptr().add(error_bytes.len()).write(0_u8);
            buffer
        };

        Some(c_error.cast())
    }
}
```

### 사용자 정의 오류 타입

Rust 내부 오류 타입의 필드 구성이 C와 동일하다고 가정하지 말고, C에 노출할 별도의 `#[repr(C)]` 구조체를 정의할 수 있습니다. 아래 예제는 내부 오류의 필드를 C 호환 구조체로 명시적으로 변환합니다.

```rust,ignore
struct ParseError {
    expected: char,
    line: u32,
    ch: u16,
}

impl ParseError {
    /* ... */
}

/* Create a second version which is exposed as a C structure */
#[repr(C)]
pub struct parse_error {
    pub expected: libc::c_char,
    pub line: u32,
    pub ch: u16,
}

impl From<ParseError> for parse_error {
    fn from(e: ParseError) -> parse_error {
        let ParseError { expected, line, ch } = e;
        parse_error { expected, line, ch }
    }
}
```

## 장점

외부 언어가 오류 정보를 명확하게 확인할 수 있으면서도 Rust 내부 코드의 API를 외부 ABI에 맞춰 바꿀 필요가 없습니다.

## 단점

변환 코드와 외부에 노출할 타입을 추가로 작성해야 합니다. 또한 일부 Rust 타입은 C 표현으로 쉽게 변환할 수 없습니다.
