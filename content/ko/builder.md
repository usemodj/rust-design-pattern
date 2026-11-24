# Builder 패턴

## 설명

Builder 패턴은 빌더 도우미에 설정 메서드를 차례로 호출해 최종 객체를 구성하는 방법입니다. 객체를 직접 생성하는 코드와 복잡한 초기화 절차를 분리해, 호출부에서 어떤 값이 설정되는지 읽기 쉽게 표현할 수 있습니다.

## 예제

다음 예제의 `Foo`는 여러 복잡한 필드를 가질 수 있는 대상 타입이고, `FooBuilder`는 그 값을 모아 `Foo`를 생성하는 빌더입니다.

```rust
#[derive(Debug, PartialEq)]
pub struct Foo {
    // Lots of complicated fields.
    bar: String,
}

impl Foo {
    // This method will help users to discover the builder
    pub fn builder() -> FooBuilder {
        FooBuilder::default()
    }
}

#[derive(Default)]
pub struct FooBuilder {
    // Probably lots of optional fields.
    bar: String,
}

impl FooBuilder {
    pub fn new(/* ... */) -> FooBuilder {
        // Set the minimally required fields of Foo.
        FooBuilder {
            bar: String::from("X"),
        }
    }

    pub fn name(mut self, bar: String) -> FooBuilder {
        // Set the name on the builder itself, and return the builder by value.
        self.bar = bar;
        self
    }

    // If we can get away with not consuming the Builder here, that is an
    // advantage. It means we can use the FooBuilder as a template for constructing
    // many Foos.
    pub fn build(self) -> Foo {
        // Create a Foo from the FooBuilder, applying all settings in FooBuilder
        // to Foo.
        Foo { bar: self.bar }
    }
}

#[test]
fn builder_test() {
    let foo = Foo {
        bar: String::from("Y"),
    };
    let foo_from_builder: Foo = FooBuilder::new().name(String::from("Y")).build();
    assert_eq!(foo, foo_from_builder);
}
```

`Foo::builder()`는 사용자가 빌더를 쉽게 발견할 수 있도록 제공하는 진입점입니다. `FooBuilder::new()`는 최소한으로 필요한 값을 설정하고, `name(...)`은 빌더의 설정을 바꾼 뒤 빌더를 반환합니다. 마지막의 `build()`는 설정을 대상 객체에 적용해 `Foo`를 만듭니다.

이 예제의 설정 메서드는 빌더를 값으로 받아 수정한 뒤 다시 값으로 반환합니다. 따라서 `FooBuilder::new().name(...).build()`처럼 메서드를 연결할 수 있습니다. `build(self)`도 빌더를 값으로 소비하므로, 이 구현에서는 빌드를 마친 빌더를 다시 사용할 수 없습니다. 빌더를 여러 객체의 템플릿으로 재사용하려면 빌더를 소비하지 않는 `build` API를 설계할 수 있습니다.

## 동기

생성 시 필요한 선택지가 많아 생성자 함수가 여러 개 필요해지거나, 객체를 만드는 과정에 부수 효과가 포함될 때 Builder 패턴이 유용합니다. 빌더는 설정을 단계적으로 모으고, 최종 생성 시점에 객체를 구성하게 합니다.

예를 들어 필수 설정과 선택 설정을 구분하거나, 여러 설정을 조합한 뒤 최종 값의 유효성을 검사할 수 있습니다. 빌더 자체가 중간 상태를 표현하므로 구성 과정의 책임과 완성된 객체의 동작도 분리됩니다.

## 장점

- **구성 로직 분리:** 객체를 만드는 메서드와 완성된 객체가 제공하는 동작을 나눌 수 있습니다.
- **생성자 수 증가 방지:** 옵션 조합마다 생성자를 추가하는 대신 빌더 메서드로 필요한 설정을 선택할 수 있습니다.
- **단순한 초기화와 복잡한 초기화 모두 지원:** 한 줄짜리 메서드 체인으로 간결하게 만들거나, 단계별로 설정을 구성할 수 있습니다.
- **호출부의 하위 호환성 유지:** 대상 구조체에 필드를 추가하더라도 빌더가 기본값이나 내부 설정을 처리하면 기존 클라이언트 코드를 그대로 유지할 수 있습니다.

## 단점

구조체 리터럴이나 간단한 생성자 함수로 객체를 만들 수 있는 경우와 비교하면 코드가 더 복잡해집니다. 빌더 타입과 설정 메서드를 추가로 유지해야 하므로, 설정이 적고 기본값이 분명한 객체에는 불필요한 추상화일 수 있습니다.

## 논의

Builder 패턴은 다른 많은 언어보다 Rust에서, 그리고 비교적 단순한 객체에도 자주 사용됩니다. Rust에는 함수 인자 오버로딩과 기본 인자값이 없기 때문입니다. 같은 이름의 메서드는 하나만 정의할 수 있으므로, C++이나 Java처럼 인자 조합별로 같은 이름의 생성자를 여러 개 두기보다 빌더를 사용하는 편이 자연스러울 수 있습니다.

빌더는 단순히 최종 객체를 만들기 위한 임시 도우미가 아니라 그 자체로 유용한 API가 될 수도 있습니다. 예를 들어 표준 라이브러리의 [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)는 프로세스를 설정하고 [`Child`](https://doc.rust-lang.org/std/process/struct.Child.html)를 실행하는 빌더입니다. 이러한 경우에는 항상 `T`와 `TBuilder`처럼 대상 타입과 빌더 타입의 이름을 짝지어 사용하지 않습니다.

### 값으로 받는 방식과 가변 참조로 받는 방식

위의 `FooBuilder` 예제는 빌더를 값으로 받아 수정하고 다시 반환합니다. 메서드 체이닝에 잘 맞는 방식입니다. 하지만 빌더를 `&mut self`로 받아 가변 참조를 반환하면 기존 빌더를 단계별로 수정할 수 있습니다. 빌림 검사기는 이 방식의 소유권과 빌림을 자연스럽게 관리합니다.

```rust,ignore
let mut fb = FooBuilder::new();
fb.a();
fb.b();
let f = fb.build();
```

가변 참조 방식은 다음처럼 단계별로 쓰는 코드뿐 아니라 `FooBuilder::new().a().b().build()`와 같은 체이닝 형식도 지원할 수 있습니다. 어떤 반환 방식을 선택할지는 빌더를 재사용할 필요가 있는지, 호출부에서 단계별 구성이 읽기 쉬운지, 빌드 과정에서 값을 복제해야 하는지 등을 고려해 결정합니다.

## Rust 스타일 가이드의 추가 설계 기준

아카이브된 Rust 스타일 가이드는 다음과 같은 상황에서 별도의 빌더를 고려하도록 권합니다. 생성에 많은 입력값이 필요하거나, 슬라이스 같은 복합 데이터를 모아야 하거나, 선택적 설정이 많거나, 여러 구현 방식 중 하나를 골라야 할 때입니다. 이런 요구를 모두 생성자 인자에 담으면 인자가 많은 생성자와 조합별 생성자가 계속 늘어날 수 있습니다.

가이드는 빌더 API를 설계할 때 다음 기준을 제시합니다.

1. 대상 타입 `T`의 설정을 단계적으로 표현할 별도 타입을 둡니다. 일반적인 이름은 `TBuilder`이지만, 더 자연스러운 역할 이름을 쓸 수도 있습니다. 예를 들어 프로세스 설정 객체는 `ProcessBuilder` 대신 `Command`라는 이름을 사용할 수 있습니다.
2. 빌더의 생성자는 `T`를 만드는 데 반드시 필요한 값만 받습니다. 나머지 선택 설정은 설정 메서드로 지정하게 합니다.
3. 설정 메서드는 편리한 API를 제공하고, 가능한 경우 슬라이스 같은 복합 입력도 단계별로 추가할 수 있게 합니다. 메서드 체이닝을 원하면 빌더 자신 또는 빌더에 대한 가변 참조를 반환합니다.
4. `build`, `spawn` 같은 하나 이상의 최종 메서드(terminal method)를 제공해 설정된 빌더를 실제 결과로 변환하거나 동작을 수행합니다.

객체 생성 과정에서 스레드를 시작하거나 프로세스를 실행하는 것처럼 부수 효과가 발생한다면 Builder 패턴은 특히 유용합니다. 설정과 실행을 구분하면 호출자가 필요한 옵션을 먼저 구성하고, 명시적인 마지막 호출에서 작업을 시작할 수 있습니다.

### 빌더가 소비되지 않는 경우

최종 동작에 빌더가 보유한 설정의 소유권이 필요하지 않다면, 빌더 자체를 소비하지 않는 방식을 우선 고려할 수 있습니다. 설정 메서드는 `&mut self`를 받아 `&mut Self`를 반환하고, 최종 메서드는 빌더를 빌려 설정을 사용할 수 있습니다. 이 방식은 현재 설정을 유지한 채 조건에 따라 설정을 더하거나, 설정을 마친 뒤 동작을 호출하기 쉽습니다.

표준 라이브러리의 `std::process::Command`는 프로세스 실행에 필요한 프로그램 이름, 인자, 작업 디렉터리 등을 모으는 설정 객체입니다. 각 `.arg(...)`나 `.current_dir(...)` 호출로 설정을 추가한 다음 `.spawn()`으로 자식 프로세스를 시작합니다. `Command`는 최종 `Child`와 별도로 존재하는 객체일 뿐 아니라, 프로세스 설정을 누적하는 빌더 역할도 합니다.

가이드의 핵심을 보여 주기 위해 아래에 현재 Rust API로 다시 구성한 예제를 추가했습니다. `ToolCommand`는 프로그램 이름과 인자, 작업 디렉터리를 보관합니다. 설정 메서드는 `&mut self`를 받아 `&mut Self`를 반환하고, `spawn(&self)`는 설정을 빌려 실제 프로세스를 시작합니다. 최종 동작이 빌더의 설정을 소유해야 하지 않으므로 빌더를 소비하지 않습니다.

```rust
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command as SystemCommand};

struct ToolCommand {
    program: String,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
}

impl ToolCommand {
    fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
        }
    }

    fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    fn args(&mut self, args: &[String]) -> &mut Self {
        self.args.extend_from_slice(args);
        self
    }

    fn current_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(dir.into());
        self
    }

    fn spawn(&self) -> io::Result<Child> {
        let mut command = SystemCommand::new(&self.program);
        command.args(&self.args);
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }
        command.spawn()
    }
}

fn spawn_tool(verbose: bool) -> io::Result<Child> {
    let mut command = ToolCommand::new("tool");
    command.args(&["--config".to_string(), "settings.toml".to_string()]);
    command.current_dir(".");
    if verbose {
        command.arg("--verbose");
    }

    command.spawn()
}

fn spawn_version() -> io::Result<Child> {
    ToolCommand::new("tool").arg("--version").spawn()
}
```

`spawn_tool`은 여러 설정을 단계적으로 추가하는 방식이고, `spawn_version`은 메서드를 연결한 한 줄 구성입니다. 두 경우 모두 `spawn`은 `ToolCommand`의 소유권을 가져가지 않습니다. `SystemCommand`는 실제 프로세스를 시작하기 위해 메서드 안에서 새로 만들어 사용합니다. `"tool"`은 예시 이름이므로 실행하려면 환경에 존재하는 프로그램 이름으로 바꿔야 합니다.

표준 라이브러리의 `std::process::Command`도 같은 형태의 비소비형 빌더 API를 제공합니다. 다만 앞선 `ToolCommand` 구현은 API 가이드의 개념을 설명하기 위해 현재 Rust의 `std::process::Command` 위에 작성한 예제이며, 아카이브 원문의 코드를 그대로 옮긴 것은 아닙니다.

### 빌드가 소유권을 가져가는 경우

최종 객체를 만들 때 설정의 소유권을 넘겨야 한다면 최종 메서드는 `&self`가 아니라 `self`를 받아야 합니다. 예를 들어 빌더가 복제할 수 없는 writer나 다른 소유 자원을 보관하고, 최종 실행 대상에 그 자원을 넘겨야 하는 경우입니다. 이 방식에서는 중간 설정 메서드도 빌더를 값으로 받아 수정한 뒤 다시 반환해야 소유권이 메서드 체인을 따라 이동합니다.

값을 반환하는 설정 메서드는 다음과 같이 바로 연결할 수 있습니다.

```rust,ignore
let thread = ThreadBuilder::new()
    .name("worker")
    .stdout(writer)
    .spawn(task);
```

대신 조건부 설정처럼 여러 단계로 구성할 때는 반환된 빌더를 다시 변수에 대입해야 합니다.

```rust,ignore
let mut thread = ThreadBuilder::new();
thread = thread.name("worker");

if reroute {
    thread = thread.stdout(writer);
}

thread.spawn(task);
```

요약하면, 빌더를 빌려서 최종 작업을 수행할 수 있으면 비소비형 API가 조건부 설정과 재사용에 편리합니다. 최종 작업에 설정의 소유권을 넘겨야 하면 소비형 API를 사용해야 하며, 이때 모든 설정 메서드가 소유된 `self`를 반환해야 한 줄 구성과 단계별 구성을 모두 지원할 수 있습니다.

### 선택 필드를 검증하는 빌더 예제

다음은 기존 문서의 추가 예제입니다. `RequestBuilder`는 URL을 선택 설정으로 보관하고, 기본 타임아웃을 제공합니다. `build()`에서 필수 URL이 누락되었는지 확인해 유효한 `Request`만 반환합니다.

```rust
#[derive(Default)]
struct RequestBuilder {
    url: Option<String>,
    timeout_ms: u64,
}

impl RequestBuilder {
    fn new() -> Self {
        Self { timeout_ms: 1_000, ..Self::default() }
    }

    fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    fn build(self) -> Result<Request, &'static str> {
        let url = self.url.ok_or("URL이 필요합니다")?;
        Ok(Request { url, timeout_ms: self.timeout_ms })
    }
}

struct Request {
    url: String,
    timeout_ms: u64,
}
```

호출부는 필요한 설정을 선택해 구성할 수 있습니다.

```rust
let request = RequestBuilder::new()
    .url("https://example.com")
    .timeout_ms(2_000)
    .build()?;
```

필수 값이 빠진 경우 `build()`가 오류를 반환하므로, 생성에 실패할 수 있는 조건을 호출부에서 명시적으로 처리할 수 있습니다.

> 참고: 아래 스타일 가이드는 Rust 1.12 시기의 문서입니다. 아카이브된 예제에는 현재 Rust에서 그대로 컴파일되지 않는 오래된 문법이나 API가 포함될 수 있으므로, 구체적인 호출 방식보다 빌더의 설계 원칙을 참고하고 실제 API는 최신 표준 라이브러리 문서를 확인하세요.

## 참고 자료

- [Rust 스타일 가이드의 Builder 설명](https://web.archive.org/web/20210104103100/https://doc.rust-lang.org/1.12.0/style/ownership/builders.html)
- [Rust API 가이드라인: 복잡한 값의 생성을 위한 Builder — Non-consuming builders](https://web.archive.org/web/20210104103000/https://rust-lang.github.io/api-guidelines/type-safety.html#non-consuming-builders-preferred)
- [`derive_builder`](https://crates.io/crates/derive_builder): 반복적인 빌더 코드를 줄이도록 구현을 자동 생성하는 크레이트
- [생성자 패턴](/docs/constructor): 생성 과정이 더 단순한 경우에 적합한 관용구
- [Builder 패턴 — 위키백과](https://en.wikipedia.org/wiki/Builder_pattern)
- [복잡한 값의 생성 — Rust API 가이드라인](https://web.archive.org/web/20210104103000/https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder)
