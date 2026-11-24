# Chain of Responsibility 패턴

## 의도

Chain of Responsibility(책임 연쇄) 패턴은 요청을 여러 처리자에게 순서대로 전달하여, 요청을 처리할 수 있는 객체가 책임을 맡도록 합니다. 클라이언트는 어떤 처리자가 요청을 담당하는지 미리 알 필요가 없습니다. 처리자가 요청을 처리하지 못하면 다음 처리자에게 넘기며, 연쇄가 끝날 때까지 처리되지 않을 수도 있습니다.

구성 요소는 다음과 같습니다.

- **요청(Request):** 처리자가 판단하는 데 필요한 정보를 담습니다.
- **처리자(Handler):** 요청을 처리하거나 다음 처리자에게 넘길지 결정하는 공통 인터페이스입니다.
- **구체적인 처리자:** 자신이 담당할 수 있는 요청을 처리합니다.
- **클라이언트:** 연쇄에 요청을 보내고 결과를 받습니다.

처리 순서는 결과에 영향을 줍니다. 이 예제는 **처음 처리한 처리자에서 멈추는** 방식입니다. 로깅이나 검증처럼 모든 단계가 요청을 처리하는 파이프라인으로 변형할 수도 있지만, 그 경우에는 중단 조건과 오류 전달 규칙을 별도로 정의해야 합니다.

## Rust에서 표현하기

Rust에서는 공통 `Handler` 트레이트와 서로 다른 처리자를 담는 `Vec<Box<dyn Handler>>`로 연쇄를 만들 수 있습니다. 각 처리자는 `Option`을 반환하여 요청을 처리했는지 표현하고, 연쇄는 순서대로 호출하다가 첫 번째 결과를 반환합니다. 모든 처리자가 건너뛰면 `None`을 반환하므로 처리되지 않은 요청도 명시적으로 확인할 수 있습니다.

요청 종류나 처리 순서가 고정되어 단순하다면 `match`나 일반 함수가 더 간결할 수 있습니다. 처리 중 실패를 처리하지 못함과 구분해야 할 때는 `Result<Option<Response>, Error>`처럼 오류를 별도로 표현하는 것이 좋습니다.

## Rust 예제: 고객 지원 요청 분류

아래 예제는 고객 지원 요청을 자동 응답과 상담원에게 순서대로 전달합니다. 자동 응답은 배송 요청만 처리하고, 상담원은 다른 요청을 처리합니다. 처리자의 순서를 바꾸면 같은 요청에 대해 다른 결과가 나올 수 있습니다.

`src/lib.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Topic {
    Shipping,
    Billing,
}

pub struct Request {
    pub topic: Topic,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    pub handled_by: &'static str,
    pub message: &'static str,
}

pub trait Handler {
    fn handle(&self, request: &Request) -> Option<Response>;
}

pub struct ShippingBot;

impl Handler for ShippingBot {
    fn handle(&self, request: &Request) -> Option<Response> {
        if request.topic == Topic::Shipping {
            Some(Response {
                handled_by: "배송 봇",
                message: "주문 페이지에서 배송 상태를 확인하세요.",
            })
        } else {
            None
        }
    }
}

pub struct SupportAgent;

impl Handler for SupportAgent {
    fn handle(&self, _request: &Request) -> Option<Response> {
        Some(Response {
            handled_by: "상담원",
            message: "상담원이 요청을 확인합니다.",
        })
    }
}

#[derive(Default)]
pub struct SupportChain {
    handlers: Vec<Box<dyn Handler>>,
}

impl SupportChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, handler: impl Handler + 'static) {
        self.handlers.push(Box::new(handler));
    }

    pub fn handle(&self, request: &Request) -> Option<Response> {
        self.handlers.iter().find_map(|handler| handler.handle(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipping_request_is_handled_by_the_bot_first() {
        let mut chain = SupportChain::new();
        chain.add(ShippingBot);
        chain.add(SupportAgent);

        let response = chain.handle(&Request { topic: Topic::Shipping }).unwrap();
        assert_eq!(response.handled_by, "배송 봇");
    }

    #[test]
    fn billing_request_is_passed_to_the_agent() {
        let mut chain = SupportChain::new();
        chain.add(ShippingBot);
        chain.add(SupportAgent);

        let response = chain.handle(&Request { topic: Topic::Billing }).unwrap();
        assert_eq!(response.handled_by, "상담원");
    }

    #[test]
    fn unhandled_request_returns_none() {
        let mut chain = SupportChain::new();
        chain.add(ShippingBot);

        assert_eq!(chain.handle(&Request { topic: Topic::Billing }), None);
        assert_eq!(SupportChain::new().handle(&Request { topic: Topic::Shipping }), None);
    }

    #[test]
    fn handler_order_determines_who_handles_the_request() {
        let mut chain = SupportChain::new();
        chain.add(SupportAgent);
        chain.add(ShippingBot);

        let response = chain.handle(&Request { topic: Topic::Shipping }).unwrap();
        assert_eq!(response.handled_by, "상담원");
    }
}
```

`find_map`은 처리자가 `Some`을 반환하는 순간 순회를 중단합니다. 따라서 상담원이 모든 요청을 수락하는 이 예제에서는 상담원을 배송 봇보다 앞에 두면 배송 봇까지 요청이 전달되지 않습니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 첫 처리자에서 처리하는 경우, 다음 처리자로 넘기는 경우, 처리자가 없거나 모두 처리하지 못하는 경우, 처리 순서가 결과를 바꾸는 경우를 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

실제 상담 시스템에서는 요청 처리 중 외부 서비스 오류가 발생할 수 있습니다. 예제의 `None`은 처리 대상이 아니라는 뜻일 뿐 오류가 아닙니다. 오류가 발생해도 다음 처리자에게 넘길지, 즉시 중단할지 정책을 정하고 오류를 반환해야 합니다.

## 장점과 단점

### 장점

- 요청을 보내는 코드가 구체적인 처리자에 직접 의존하지 않습니다.
- 처리자를 추가하거나 조합하고 순서를 변경하기 쉽습니다.
- 각 처리자의 판단과 처리 책임을 분리할 수 있습니다.

### 단점

- 어떤 처리자가 요청을 맡을지, 요청이 처리될지 연쇄를 확인해야 알 수 있습니다.
- 순서가 잘못되면 뒤쪽 처리자에게 요청이 도달하지 않을 수 있습니다.
- 처리자가 많아지면 추적과 디버깅이 어려워지고 불필요한 검사 비용이 늘어날 수 있습니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Chain of Responsibility — Refactoring Guru](https://refactoring.guru/design-patterns/chain-of-responsibility)
- [Chain-of-responsibility pattern — 위키백과](https://en.wikipedia.org/wiki/Chain-of-responsibility_pattern)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [`Iterator::find_map` 문서](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.find_map)
