# Memento 패턴

## 의도

Memento(메멘토) 패턴은 객체의 내부 상태를 외부에 직접 노출하지 않고 저장해 두었다가 나중에 복원하는 방법입니다. 편집기의 실행 취소, 작업 단계의 체크포인트, 설정 복구처럼 이전 상태가 필요한 경우에 사용할 수 있습니다.

상태를 소유한 객체가 스냅숏을 만들고 복원하며, 이력을 관리하는 쪽은 스냅숏의 내부 내용을 몰라도 됩니다.

## 구성 요소

- **Originator(원본 객체):** 현재 상태를 관리하고, 자신의 상태에서 스냅숏을 만들거나 복원합니다.
- **Memento(메멘토):** 복원에 필요한 상태를 보관하는 값입니다. 내부 내용은 원본 객체의 모듈 밖에 공개하지 않습니다.
- **Caretaker(관리자):** 메멘토를 보관하고 필요한 시점에 원본 객체에 전달합니다. 상태를 직접 수정하지 않습니다.

## Rust에서 표현하기

Rust에서는 비공개 필드를 가진 구조체로 스냅숏을 표현하고 모듈의 가시성으로 내부 상태를 보호할 수 있습니다. 상태를 `Clone`하거나 필요한 필드를 복사하면 원본이 이후 변경되어도 저장된 스냅숏은 독립적으로 유지됩니다. `Vec`에 스냅숏을 쌓으면 가장 최근 상태부터 복원할 수 있습니다.

복사 비용은 상태의 크기와 저장 빈도에 따라 증가합니다. 큰 상태나 긴 이력이 필요하다면 이력 개수 제한, 공유 가능한 불변 데이터 또는 변경분 저장 등을 검토하세요. 어느 방식을 택하든 복원에 필요한 상태가 빠지지 않아야 합니다.

## Rust 예제: 문서 상태 저장과 복원

`Document`가 원본 객체이고 `DocumentSnapshot`이 메멘토입니다. `History`는 편집 **전** 상태를 저장한 뒤 가장 최근 스냅숏부터 복원합니다. 이 예제는 Undo만 제공하며 Redo는 구현하지 않습니다.

`src/lib.rs`:

```rust
pub mod editor {
    #[derive(Default)]
    pub struct Document {
        text: String,
    }

    pub struct DocumentSnapshot {
        text: String,
    }

    impl Document {
        pub fn text(&self) -> &str {
            &self.text
        }

        pub fn set_text(&mut self, text: &str) {
            self.text = text.to_owned();
        }

        pub fn snapshot(&self) -> DocumentSnapshot {
            DocumentSnapshot {
                text: self.text.clone(),
            }
        }

        pub fn restore(&mut self, snapshot: DocumentSnapshot) {
            self.text = snapshot.text;
        }
    }

    #[derive(Default)]
    pub struct History {
        snapshots: Vec<DocumentSnapshot>,
    }

    impl History {
        pub fn save(&mut self, document: &Document) {
            self.snapshots.push(document.snapshot());
        }

        pub fn undo(&mut self, document: &mut Document) -> bool {
            match self.snapshots.pop() {
                Some(snapshot) => {
                    document.restore(snapshot);
                    true
                }
                None => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::editor::{Document, History};

    #[test]
    fn restores_previous_states_in_reverse_order() {
        let mut document = Document::default();
        let mut history = History::default();
        history.save(&document);
        document.set_text("첫 번째");
        history.save(&document);
        document.set_text("두 번째");

        assert!(history.undo(&mut document));
        assert_eq!(document.text(), "첫 번째");
        assert!(history.undo(&mut document));
        assert_eq!(document.text(), "");
        assert!(!history.undo(&mut document));
    }

    #[test]
    fn snapshot_is_independent_of_later_changes() {
        let mut document = Document::default();
        document.set_text("원래 내용");
        let snapshot = document.snapshot();
        document.set_text("변경된 내용");

        document.restore(snapshot);
        assert_eq!(document.text(), "원래 내용");
    }

    #[test]
    fn empty_history_leaves_document_unchanged() {
        let mut document = Document::default();
        let mut history = History::default();
        document.set_text("유지할 내용");

        assert!(!history.undo(&mut document));
        assert_eq!(document.text(), "유지할 내용");
    }
}
```

`DocumentSnapshot`과 `History`의 필드는 비공개이므로 모듈 밖에서는 스냅숏의 내용을 직접 바꿀 수 없습니다. `History::save`는 문서의 현재 상태를 복사하고, `undo`는 그 복사본의 소유권을 복원할 문서에 넘깁니다. 빈 이력에서 `false`를 반환하는 것은 복원할 상태가 없다는 정상적인 결과입니다.

이 단순한 예제는 자동으로 편집을 감지하지 않으므로 호출자가 변경 전에 `save`해야 합니다. 또한 스냅숏이 어느 문서에서 만들어졌는지 식별하지 않아 다른 `Document`에도 복원할 수 있습니다. 문서별 이력을 엄격히 구분해야 한다면 소유 관계나 식별자를 검증하는 정책을 추가해야 합니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 여러 스냅숏을 역순으로 복원하는 경우, 이후 변경이 저장된 상태에 영향을 주지 않는 경우, 빈 이력에서 문서가 유지되는 경우를 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장하고 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 내부 상태를 이력 관리자에게 공개하지 않고 이전 상태를 복원할 수 있습니다.
- 저장과 복원 규칙을 상태를 소유한 객체에 모을 수 있습니다.
- 여러 체크포인트를 보관해 순서대로 되돌릴 수 있습니다.

### 단점

- 전체 상태를 자주 복사하면 메모리와 처리 비용이 커질 수 있습니다.
- 스냅숏 생성 시점을 빠뜨리거나 복원에 필요한 필드를 누락하면 이력이 올바르지 않습니다.
- 파일, 네트워크 요청처럼 객체 밖에서 발생한 부수 효과는 메모리 상태 복원만으로 되돌릴 수 없습니다.

## Command 패턴과의 차이

[Command 패턴](/docs/command)은 실행할 **작업**을 값으로 캡슐화하며 작업을 되돌리는 동작을 제공할 수 있습니다. Memento는 복원할 **상태**를 저장하는 데 초점을 둡니다. 두 패턴을 함께 사용해 명령이 실행 전 스냅숏을 저장하고 Undo에 활용할 수도 있습니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Memento 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/memento)
- [Memento 패턴 — 위키백과](https://en.wikipedia.org/wiki/Memento_pattern)
- [Rust Book: 모듈과 가시성](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [`Clone` 표준 라이브러리 문서](https://doc.rust-lang.org/std/clone/trait.Clone.html)
