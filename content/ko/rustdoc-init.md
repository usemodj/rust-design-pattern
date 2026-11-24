# 문서 예제 초기화 간소화

## 설명

문서 예제에서 구조체를 초기화하는 과정이 번거롭다면, 해당 구조체를 인자로 받는 보조 함수로 예제를 감싸는 방법을 사용할 수 있습니다. 그러면 예제마다 복잡한 초기화 코드를 반복해서 작성하지 않아도 됩니다.

## 동기

여러 메서드가 있는 구조체에 복잡한 매개변수가 여럿 필요하다면, 각 메서드의 문서 예제에서 같은 준비 코드를 반복하게 될 수 있습니다.

아래 예제에서는 `Connection`과 `Request`를 만들기 위한 보일러플레이트가 `send_request` 문서 예제에 들어 있습니다. 같은 구조체의 다른 메서드에도 예제를 추가하면 초기화 코드까지 다시 작성해야 합니다.

````rust,ignore
struct Connection {
    name: String,
    stream: TcpStream,
}

impl Connection {
    /// Sends a request over the connection.
    ///
    /// # Example
    /// ```no_run
    /// # // Boilerplate are required to get an example working.
    /// # let stream = TcpStream::connect("127.0.0.1:34254");
    /// # let connection = Connection { name: "foo".to_owned(), stream };
    /// # let request = Request::new("RequestId", RequestType::Get, "payload");
    /// let response = connection.send_request(request);
    /// assert!(response.is_ok());
    /// ```
    fn send_request(&self, request: Request) -> Result<Status, SendErr> {
        // ...
    }

    /// Oh no, all that boilerplate needs to be repeated here!
    fn check_status(&self) -> Status {
        // ...
    }
}
````

## 예제

`Connection`과 `Request`를 만드는 코드를 매번 쓰는 대신, 두 값을 인자로 받는 보조 함수를 둘 수 있습니다. Rustdoc 예제에서 함수 정의 줄은 `#`으로 시작하므로 생성된 문서에서는 숨겨지지만, 테스트 코드는 해당 정의를 포함해 컴파일합니다.

````rust,ignore
struct Connection {
    name: String,
    stream: TcpStream,
}

impl Connection {
    /// Sends a request over the connection.
    ///
    /// # Example
    /// ```
    /// # fn call_send(connection: Connection, request: Request) {
    /// let response = connection.send_request(request);
    /// assert!(response.is_ok());
    /// # }
    /// ```
    fn send_request(&self, request: Request) -> Result<Status, SendErr> {
        // ...
    }
}
````

위 예제의 `assert!(response.is_ok())`는 테스트 중 실제로 실행되지 않습니다. 이 문장은 호출되지 않는 `call_send` 함수 안에 있기 때문입니다.

## 장점

예제가 훨씬 간결해지고, 반복되는 초기화 코드를 줄일 수 있습니다.

## 단점

예제를 함수 안에 넣으면 그 함수가 호출되지 않는 한 본문은 실행되지 않습니다. 따라서 예제 안의 단언문도 실행 검증을 받지 않습니다. 다만 `cargo test`를 실행하면 함수 본문이 컴파일되는지는 검사합니다.

이 패턴은 예제를 실행하지 않고 컴파일만 확인해야 하는 경우에 특히 유용합니다. 보조 함수로 감싸면 `no_run`을 지정하지 않아도 실행 대신 컴파일 검사를 받을 수 있습니다.

## 논의

실행 시 단언문을 확인할 필요가 없다면 이 패턴이 잘 맞습니다. 반대로 단언문 실행이 중요하다면, `#[doc(hidden)]`가 붙은 공개 보조 메서드를 만들어 문서에서 사용할 수 있습니다. 메서드가 크레이트의 공개 API에 속하므로 Rustdoc 예제에서 호출해 실행 검증할 수 있고, `#[doc(hidden)]` 덕분에 일반 생성 문서에는 노출되지 않습니다.
