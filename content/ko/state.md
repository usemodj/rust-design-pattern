# State 패턴

## 의도

State(상태) 패턴은 객체의 내부 상태에 따라 허용되는 동작과 동작의 결과가 달라지도록 구성합니다. 상태와 전이 규칙을 명시하면 객체가 현재 어떤 작업을 할 수 있는지, 다음에 어떤 상태로 바뀔 수 있는지 이해하기 쉽습니다.

예를 들어 문서는 초안일 때 편집할 수 있고, 검토 중일 때 승인하거나 반려할 수 있으며, 승인된 뒤에야 독자에게 내용을 공개할 수 있습니다.

## 구성 요소

- **컨텍스트(Context):** 현재 상태와 공유 데이터를 보관하고 외부에 동작을 제공합니다.
- **상태(State):** 현재 단계에 따른 동작과 전이 규칙을 표현합니다.
- **클라이언트:** 컨텍스트에 작업을 요청하고 결과를 확인합니다.

전통적인 객체 지향 State 패턴은 상태별 객체에 동작을 위임하기도 합니다. 상태 종류가 고정된 Rust 프로그램에서는 열거형과 `match`로도 같은 목적을 간결하게 표현할 수 있습니다.

## Rust에서 표현하기

- **열거형과 `match`:** 알려진 상태와 전이를 한곳에서 확인하고, 상태가 추가되면 빠진 경우를 컴파일러가 확인할 수 있습니다.
- **트레이트와 상태 객체:** 상태별 구현을 별도 타입으로 분리하거나 실행 중 다른 구현을 다루어야 할 때 사용할 수 있습니다.
- **타입 상태(typestate):** `Draft`, `Published` 같은 타입으로 단계를 구분해 일부 잘못된 작업을 컴파일 시점에 막을 수 있습니다. 대신 실행 중 변하는 상태를 같은 타입의 컬렉션에 보관하는 등 사용 방식에 제약이 생길 수 있습니다.

아래 예제는 열거형 방식입니다. 외부 요청에 따라 상태가 실행 중 바뀌므로, 허용되지 않은 작업은 `Result`의 오류로 알립니다. 단순한 조건 분기로 충분한 경우라면 상태 객체를 따로 만드는 것은 불필요할 수 있습니다.

## Rust 예제: 게시글 검토와 공개

게시글은 `Draft`에서 시작합니다. 초안을 편집한 뒤 검토를 요청하고, 검토 중 승인되면 `Published`가 됩니다. 검토 중 반려되면 다시 초안으로 돌아갑니다.

| 현재 상태 | 작업 | 결과 |
| --- | --- | --- |
| Draft | 편집 | 내용 변경, Draft 유지 |
| Draft | 검토 요청 | PendingReview |
| PendingReview | 승인 | Published |
| PendingReview | 반려 | Draft |
| 그 외 조합 | 편집 또는 상태 전이 요청 | 오류, 기존 상태와 내용 유지 |

`src/lib.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostState {
    Draft,
    PendingReview,
    Published,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Edit,
    RequestReview,
    Approve,
    Reject,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidAction {
    pub state: PostState,
    pub action: Action,
}

pub struct Post {
    content: String,
    state: PostState,
}

impl Post {
    pub fn new() -> Self {
        Self { content: String::new(), state: PostState::Draft }
    }

    pub fn state(&self) -> PostState {
        self.state
    }

    pub fn content(&self) -> &str {
        match self.state {
            PostState::Published => &self.content,
            PostState::Draft | PostState::PendingReview => "",
        }
    }

    pub fn edit(&mut self, text: &str) -> Result<(), InvalidAction> {
        if self.state != PostState::Draft {
            return Err(InvalidAction { state: self.state, action: Action::Edit });
        }
        self.content = text.to_owned();
        Ok(())
    }

    pub fn request_review(&mut self) -> Result<(), InvalidAction> {
        self.transition(Action::RequestReview)
    }

    pub fn approve(&mut self) -> Result<(), InvalidAction> {
        self.transition(Action::Approve)
    }

    pub fn reject(&mut self) -> Result<(), InvalidAction> {
        self.transition(Action::Reject)
    }

    fn transition(&mut self, action: Action) -> Result<(), InvalidAction> {
        let next = match (self.state, action) {
            (PostState::Draft, Action::RequestReview) => PostState::PendingReview,
            (PostState::PendingReview, Action::Approve) => PostState::Published,
            (PostState::PendingReview, Action::Reject) => PostState::Draft,
            (state, action) => return Err(InvalidAction { state, action }),
        };
        self.state = next;
        Ok(())
    }
}

impl Default for Post {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_visible_only_after_approval() {
        let mut post = Post::new();
        post.edit("Rust 패턴").unwrap();
        assert_eq!(post.state(), PostState::Draft);
        assert_eq!(post.content(), "");

        post.request_review().unwrap();
        assert_eq!(post.state(), PostState::PendingReview);
        assert_eq!(post.content(), "");

        post.approve().unwrap();
        assert_eq!(post.state(), PostState::Published);
        assert_eq!(post.content(), "Rust 패턴");
    }

    #[test]
    fn rejection_returns_to_editable_draft() {
        let mut post = Post::new();
        post.edit("첫 내용").unwrap();
        post.request_review().unwrap();
        post.reject().unwrap();
        assert_eq!(post.state(), PostState::Draft);
        post.edit("수정된 내용").unwrap();
        post.request_review().unwrap();
        post.approve().unwrap();
        assert_eq!(post.content(), "수정된 내용");
    }

    #[test]
    fn invalid_actions_return_error_without_changing_state_or_content() {
        let mut post = Post::new();
        post.edit("원래 내용").unwrap();
        assert_eq!(
            post.approve(),
            Err(InvalidAction { state: PostState::Draft, action: Action::Approve })
        );
        assert_eq!(post.state(), PostState::Draft);
        post.request_review().unwrap();
        assert_eq!(
            post.edit("잘못된 수정"),
            Err(InvalidAction { state: PostState::PendingReview, action: Action::Edit })
        );
        assert_eq!(
            post.request_review(),
            Err(InvalidAction { state: PostState::PendingReview, action: Action::RequestReview })
        );
        assert_eq!(post.state(), PostState::PendingReview);
        post.approve().unwrap();
        assert_eq!(post.content(), "원래 내용");
        assert_eq!(
            post.reject(),
            Err(InvalidAction { state: PostState::Published, action: Action::Reject })
        );
        assert_eq!(post.state(), PostState::Published);
        assert_eq!(post.content(), "원래 내용");
    }
}
```

`Post`가 컨텍스트이고 `PostState`가 현재 상태입니다. `transition`은 유효한 상태·작업 조합에만 다음 상태를 지정하므로, 실패한 요청은 기존 상태를 바꾸지 않습니다. `content()`는 공개 API의 예시로 미승인 내용을 빈 문자열로 나타냅니다. 빈 게시글이 승인된 경우와 아직 공개되지 않은 경우를 구분해야 한다면 `Option<&str>` 같은 반환 타입을 사용할 수 있습니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 승인 전 내용 숨김과 승인 후 공개, 반려 후 재편집·승인, 허용되지 않은 작업에서 오류 반환과 기존 상태·내용 유지를 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 상태에 따른 동작과 허용되는 전이를 명확하게 표현할 수 있습니다.
- 잘못된 요청을 오류로 나타내고 기존 상태를 보존할 수 있습니다.
- 상태별 규칙이 늘어날 때 관련 로직을 모아 검토할 수 있습니다.

### 단점

- 간단한 상태 분기에도 별도 타입이나 구조를 도입하면 코드가 복잡해질 수 있습니다.
- 상태와 작업이 늘어나면 가능한 전이 조합과 테스트가 함께 증가합니다.
- 열거형 구현에서 새 상태를 추가하면 관련 `match`를 수정해야 합니다. 유효한 전이의 정책은 별도로 설계해야 합니다.

## Strategy 패턴과의 차이

[Strategy 패턴](/docs/strategy)은 같은 작업을 수행하는 알고리즘을 선택·교체하는 데 초점을 둡니다. State는 객체의 현재 상태에 따라 가능한 동작이 달라지고 작업의 결과로 상태가 전이되는 데 초점을 둡니다. 두 패턴은 유사한 트레이트 구조를 사용할 수 있지만 해결하려는 문제는 다릅니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [State 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/state)
- [State 패턴 — 위키백과](https://en.wikipedia.org/wiki/State_pattern)
- [Rust Book: 객체 지향 디자인 패턴 구현하기](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html)
- [Rust Book: 열거형과 `match`](https://doc.rust-lang.org/book/ch06-02-match.html)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
