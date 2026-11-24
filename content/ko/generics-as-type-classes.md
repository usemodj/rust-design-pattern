# 타입 클래스로서의 제네릭

Rust의 타입 시스템은 제네릭 타입과 트레이트 바운드로 타입이 제공해야 할 동작을 컴파일 시점에 표현합니다. `Vec<i32>`와 `Vec<char>`는 서로 다른 구체 타입이며, 컴파일러는 제네릭 코드를 각 타입에 맞게 모노모피제이션할 수 있습니다.

이 방식은 객체 지향 언어의 상속과 다릅니다. 타입은 임의로 메서드를 추가하는 대신, 명시적으로 트레이트를 구현하고 해당 트레이트를 요구하는 API에 전달됩니다. 따라서 특정 상태에서만 유효한 메서드를 트레이트 객체에 억지로 넣지 않고, 제네릭 매개변수별 `impl` 블록으로 API를 나눌 수 있습니다.

```rust,ignore
struct Request<P: Protocol> {
    protocol: P,
}

impl<P: Protocol> Request<P> {
    fn common_operation(&self) {}
}

impl Request<Nfs> {
    fn mount_point(&self) -> &Path {
        self.protocol.mount_point()
    }
}
```

위 설계에서는 `mount_point`가 NFS 요청에만 존재합니다. BOOTP 요청에서 이 메서드를 호출하려 하면 컴파일 오류가 나므로, 실행 중 `Option`을 검사하는 API보다 잘못된 상태를 타입으로 방지할 수 있습니다.

## 장단점

공통 동작과 상태별 동작을 분리하면 중복된 필드와 조건문을 줄이고 API를 명료하게 만들 수 있습니다. 새 트레이트는 의미 있는 이름과 추가 동작의 확장 지점을 제공할 수도 있습니다. 반면 각 타입의 코드가 별도로 생성되어 바이너리가 커질 수 있고, 제네릭 API와 트레이트 경계가 복잡해질 수 있습니다.

설정 단계별 API에는 [Builder 패턴](/docs/builder), 동작만 바뀌는 경우에는 [Strategy 패턴](/docs/strategy)이 더 적합할 수 있습니다.

## 실행 가능한 예제: 프로토콜별 API

`Protocol`은 공통 동작을 정의하고, `Request<Nfs>`에만 마운트 경로 접근자를 제공합니다. 트레이트는 함수형 언어의 타입 클래스와 역할이 비슷하지만, 언어별 규칙과 표현력이 같다는 뜻은 아닙니다.

`src/lib.rs`:

```rust
use std::path::{Path, PathBuf};

pub trait Protocol {
    fn name(&self) -> &'static str;
}

pub struct Nfs {
    pub mount_point: PathBuf,
}

pub struct Bootp;

impl Protocol for Nfs {
    fn name(&self) -> &'static str { "NFS" }
}

impl Protocol for Bootp {
    fn name(&self) -> &'static str { "BOOTP" }
}

pub struct Request<P: Protocol> {
    protocol: P,
}

impl<P: Protocol> Request<P> {
    pub fn new(protocol: P) -> Self {
        Self { protocol }
    }

    pub fn protocol_name(&self) -> &'static str {
        self.protocol.name()
    }
}

impl Request<Nfs> {
    pub fn mount_point(&self) -> &Path {
        &self.protocol.mount_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_and_protocol_specific_methods() {
        let nfs = Request::new(Nfs { mount_point: PathBuf::from("data") });
        let bootp = Request::new(Bootp);
        assert_eq!(nfs.protocol_name(), "NFS");
        assert_eq!(nfs.mount_point(), Path::new("data"));
        assert_eq!(bootp.protocol_name(), "BOOTP");
    }
}
```

`Request<Bootp>`에는 `mount_point` 메서드가 없으므로 호출하려 하면 컴파일되지 않습니다. 구체 타입을 실행 중 선택해 같은 컬렉션에 넣어야 한다면 열거형이나 트레이트 객체를 고려할 수 있습니다. 예제를 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 `cargo test`로 테스트할 수 있습니다.

## 원문 소스 코드 예제

### 예제 1

```rust,ignore
enum AuthInfo {
    Nfs(crate::nfs::AuthInfo),
    Bootp(crate::bootp::AuthInfo),
}

struct FileDownloadRequest {
    file_name: PathBuf,
    authentication: AuthInfo,
}
```

### 예제 2

```rust,ignore
struct FileDownloadRequest {
    file_name: PathBuf,
    authentication: AuthInfo,
    mount_point: Option<PathBuf>,
}

impl FileDownloadRequest {
    // ... other methods ...

    /// Gets an NFS mount point if this is an NFS request. Otherwise,
    /// return None.
    pub fn mount_point(&self) -> Option<&Path> {
        self.mount_point.as_ref()
    }
}
```

### 예제 3

```rust
use std::path::{Path, PathBuf};

mod nfs {
    #[derive(Clone)]
    pub(crate) struct AuthInfo(String); // NFS session management omitted
}

mod bootp {
    pub(crate) struct AuthInfo(); // no authentication in bootp
}

// Keep the module private to prevent outside users from inventing their own protocols.
mod proto_trait {
    use super::{bootp, nfs};
    use std::path::{Path, PathBuf};

    pub(crate) trait ProtoKind {
        type AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo;
    }

    pub struct Nfs {
        auth: nfs::AuthInfo,
        mount_point: PathBuf,
    }

    impl Nfs {
        pub(crate) fn mount_point(&self) -> &Path {
            &self.mount_point
        }
    }

    impl ProtoKind for Nfs {
        type AuthInfo = nfs::AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo {
            self.auth.clone()
        }
    }

    pub struct Bootp(); // no additional metadata

    impl ProtoKind for Bootp {
        type AuthInfo = bootp::AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo {
            bootp::AuthInfo()
        }
    }
}

use proto_trait::ProtoKind; // keep internal to prevent impls
pub use proto_trait::{Bootp, Nfs}; // re-export so callers can see them

struct FileDownloadRequest<P: ProtoKind> {
    file_name: PathBuf,
    protocol: P,
}

// all common API parts go into a generic impl block
impl<P: ProtoKind> FileDownloadRequest<P> {
    fn file_path(&self) -> &Path {
        &self.file_name
    }

    fn auth_info(&self) -> P::AuthInfo {
        self.protocol.auth_info()
    }
}

// all protocol-specific impls go into their own block
impl FileDownloadRequest<Nfs> {
    fn mount_point(&self) -> &Path {
        self.protocol.mount_point()
    }
}

fn main() {
    // your code here
}
```

### 예제 4

```rust,ignore
fn main() {
    let mut socket = crate::bootp::listen()?;
    while let Some(request) = socket.next_request()? {
        match request.mount_point().as_ref() {
            "/secure" => socket.send("Access denied"),
            _ => {} // continue on...
        }
        // Rest of the code here
    }
}
```

## 참고 자료

- [Rust의 함수형 기능](/docs/functional-overview)
- [Rust Book: 제네릭 타입](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [Rust Book: 트레이트와 트레이트 바운드](https://doc.rust-lang.org/book/ch10-02-traits.html)
