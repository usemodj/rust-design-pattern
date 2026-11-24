# Mediator 패턴

## 의도

Mediator(중재자) 패턴은 여러 구성 요소가 서로 직접 통신하는 대신, 중재자를 통해 상호작용하도록 합니다. 구성 요소 사이의 협력 규칙을 중재자에 모아 각 구성 요소가 다른 구성 요소의 구현을 직접 알 필요가 없도록 합니다.

예를 들어 채팅 참가자가 다른 모든 참가자를 직접 보관하고 메시지를 전달하는 대신, 채팅방이 참가자를 관리하고 수신자에게 메시지를 전달할 수 있습니다.

## 구성 요소

- **중재자 인터페이스(Mediator):** 구성 요소가 협력을 요청할 때 사용하는 공통 API입니다.
- **구체적인 중재자(Concrete Mediator):** 구성 요소를 관리하고 전달이나 조정 규칙을 구현합니다.
- **구성 요소(Colleague):** 다른 구성 요소를 직접 참조하지 않고 중재자에게 작업을 요청합니다.
- **클라이언트:** 중재자와 구성 요소를 생성하고 연결합니다.

## Rust에서 표현하기

Rust에서는 중재자의 동작을 트레이트로 정의하고 구조체에 구현할 수 있습니다. 구성 요소가 중재자를 계속 소유하거나 상호 참조하도록 만들지 않고, 필요한 메서드 호출에 `&mut` 참조를 전달하면 소유권과 빌림 관계를 단순하게 유지할 수 있습니다. 트레이트 객체나 채널은 실행 중 구현을 교체하거나 비동기 메시지 전달이 필요할 때 고려할 수 있습니다.

Mediator는 구성 요소 간 **협력 방식**을 조정합니다. Observer는 상태 변화에 관심 있는 구독자에게 **변화를 알리는 방식**에 초점을 둡니다. 두 패턴은 함께 사용될 수 있지만 중재자가 반드시 구독·알림 시스템을 구현해야 하는 것은 아닙니다.

## Rust 예제: 채팅방을 통한 메시지 전달

`User`는 다른 참가자를 직접 참조하지 않습니다. `Mediator`에 발신자, 수신자, 메시지를 전달하고, `ChatRoom`은 등록 여부를 확인한 뒤 수신자의 메시지함에 저장합니다. 존재하지 않는 참가자와 중복 등록은 오류로 반환합니다.

`src/lib.rs`:

```rust
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub from: String,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChatError {
    DuplicateUser,
    UnknownSender,
    UnknownRecipient,
}

pub trait Mediator {
    fn send(&mut self, from: &str, to: &str, text: &str) -> Result<(), ChatError>;
}

pub struct User {
    name: String,
}

impl User {
    pub fn send(
        &self,
        mediator: &mut impl Mediator,
        to: &str,
        text: &str,
    ) -> Result<(), ChatError> {
        mediator.send(&self.name, to, text)
    }
}

#[derive(Default)]
pub struct ChatRoom {
    inboxes: HashMap<String, Vec<Message>>,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: &str) -> Result<User, ChatError> {
        if self.inboxes.contains_key(name) {
            return Err(ChatError::DuplicateUser);
        }
        self.inboxes.insert(name.to_owned(), Vec::new());
        Ok(User { name: name.to_owned() })
    }

    pub fn inbox(&self, name: &str) -> Option<&[Message]> {
        self.inboxes.get(name).map(Vec::as_slice)
    }
}

impl Mediator for ChatRoom {
    fn send(&mut self, from: &str, to: &str, text: &str) -> Result<(), ChatError> {
        if !self.inboxes.contains_key(from) {
            return Err(ChatError::UnknownSender);
        }
        let inbox = self.inboxes.get_mut(to).ok_or(ChatError::UnknownRecipient)?;
        inbox.push(Message {
            from: from.to_owned(),
            text: text.to_owned(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mediator_delivers_only_to_the_recipient() {
        let mut room = ChatRoom::new();
        let alice = room.register("Alice").unwrap();
        room.register("Bob").unwrap();
        room.register("Carol").unwrap();

        alice.send(&mut room, "Bob", "안녕하세요!").unwrap();

        assert_eq!(
            room.inbox("Bob").unwrap(),
            &[Message { from: "Alice".into(), text: "안녕하세요!".into() }]
        );
        assert!(room.inbox("Alice").unwrap().is_empty());
        assert!(room.inbox("Carol").unwrap().is_empty());
    }

    #[test]
    fn registered_users_can_communicate_in_both_directions() {
        let mut room = ChatRoom::new();
        let alice = room.register("Alice").unwrap();
        let bob = room.register("Bob").unwrap();

        alice.send(&mut room, "Bob", "안녕").unwrap();
        bob.send(&mut room, "Alice", "반가워").unwrap();

        assert_eq!(room.inbox("Alice").unwrap()[0].from, "Bob");
        assert_eq!(room.inbox("Bob").unwrap()[0].from, "Alice");
    }

    #[test]
    fn duplicate_registration_preserves_existing_inbox() {
        let mut room = ChatRoom::new();
        let alice = room.register("Alice").unwrap();
        room.register("Bob").unwrap();
        alice.send(&mut room, "Bob", "첫 메시지").unwrap();

        assert!(matches!(room.register("Bob"), Err(ChatError::DuplicateUser)));
        assert_eq!(room.inbox("Bob").unwrap().len(), 1);
    }

    #[test]
    fn unknown_participants_are_rejected_without_delivery() {
        let mut room = ChatRoom::new();
        let alice = room.register("Alice").unwrap();
        room.register("Bob").unwrap();

        assert_eq!(alice.send(&mut room, "Nobody", "안녕"), Err(ChatError::UnknownRecipient));
        assert_eq!(room.send("Nobody", "Bob", "안녕"), Err(ChatError::UnknownSender));
        assert!(room.inbox("Bob").unwrap().is_empty());
        assert!(room.inbox("Nobody").is_none());
    }
}
```

`User::send`는 `Mediator` 트레이트만 요구하므로 `ChatRoom`의 메시지함 구현을 알 필요가 없습니다. `ChatRoom::send`는 발신자와 수신자를 확인한 뒤에만 메시지를 저장합니다. 이 예제는 프로세스 내부에서 동기적으로 동작하며 실제 네트워크 전송이나 발신자 인증을 구현하지 않습니다. 실제 서비스에서는 인증, 전달 실패, 동시성과 저장 정책을 별도로 설계해야 합니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 지정한 수신자에게만 전달되는지, 양방향 통신이 가능한지, 중복 등록이 기존 메시지함을 유지하는지, 미등록 참가자의 요청이 오류로 처리되는지 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 구성 요소가 다른 구성 요소를 직접 참조하지 않아 서로 간의 결합을 줄일 수 있습니다.
- 통신과 협력 규칙을 한 곳에서 이해하고 변경하기 쉽습니다.
- 구성 요소가 중재자 인터페이스에 의존하면 다른 구현을 전달할 수 있습니다.

### 단점

- 중재자에 규칙이 지나치게 집중되면 복잡하고 변경하기 어려운 객체가 될 수 있습니다.
- 간단한 두 객체의 통신에도 중재자를 도입하면 불필요한 간접 계층이 생길 수 있습니다.
- 공유 상태, 오류 처리, 메시지 순서와 동시성은 별도로 설계해야 합니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Mediator 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/mediator)
- [Mediator 패턴 — 위키백과](https://en.wikipedia.org/wiki/Mediator_pattern)
- [Rust Book: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust Book: 메시지 전달로 스레드 간 데이터 전송하기](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [`HashMap` 표준 라이브러리 문서](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
