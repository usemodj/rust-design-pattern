# Command 패턴

## 설명

Command 패턴의 기본 아이디어는 수행할 동작을 별도의 객체로 분리하고, 그 객체를 인자로 전달하는 것입니다. 동작 자체를 값으로 표현하면 나중에 실행하거나 다른 곳에 전달하고 저장할 수 있습니다.

## 동기

여러 작업이나 트랜잭션을 객체로 캡슐화했다고 가정해 봅시다. 이 작업들을 서로 다른 시점에 특정 순서로 실행하거나 호출하고 싶을 수 있습니다. 사용자 버튼 클릭이나 데이터 패킷 도착처럼 이벤트가 발생했을 때 명령을 실행할 수도 있습니다.

명령을 되돌릴 수 있게 만들면 편집기 작업처럼 실행 취소가 필요한 기능에 유용합니다. 실행한 명령을 로그에 저장해 두면 시스템이 충돌한 뒤에도 명령을 다시 적용할 수 있습니다.

## 예제

데이터베이스 작업으로 `create table`과 `add field`를 생각해 봅시다. 각 작업은 명령이며, 자신을 되돌리는 방법도 알고 있습니다. 예를 들어 `create table`은 `drop table`로, `add field`는 `remove field`로 되돌릴 수 있습니다. 데이터베이스 마이그레이션을 실행할 때는 명령을 정의된 순서대로 수행하고, 롤백할 때는 전체 명령을 역순으로 수행합니다.

## 접근법: 트레이트 객체 사용하기

`execute`와 `rollback` 두 연산을 제공하는 공통 트레이트를 정의합니다. 각 명령 구조체는 이 트레이트를 구현하고, `Box<dyn Migration>` 컬렉션에 저장됩니다.

```rust
pub trait Migration {
    fn execute(&self) -> &str;
    fn rollback(&self) -> &str;
}

pub struct CreateTable;
impl Migration for CreateTable {
    fn execute(&self) -> &str {
        "create table"
    }
    fn rollback(&self) -> &str {
        "drop table"
    }
}

pub struct AddField;
impl Migration for AddField {
    fn execute(&self) -> &str {
        "add field"
    }
    fn rollback(&self) -> &str {
        "remove field"
    }
}

struct Schema {
    commands: Vec<Box<dyn Migration>>,
}

impl Schema {
    fn new() -> Self {
        Self { commands: vec![] }
    }

    fn add_migration(&mut self, cmd: Box<dyn Migration>) {
        self.commands.push(cmd);
    }

    fn execute(&self) -> Vec<&str> {
        self.commands.iter().map(|cmd| cmd.execute()).collect()
    }
    fn rollback(&self) -> Vec<&str> {
        self.commands
            .iter()
            .rev() // reverse iterator's direction
            .map(|cmd| cmd.rollback())
            .collect()
    }
}

fn main() {
    let mut schema = Schema::new();

    let cmd = Box::new(CreateTable);
    schema.add_migration(cmd);
    let cmd = Box::new(AddField);
    schema.add_migration(cmd);

    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

`Schema`는 명령을 추가된 순서대로 실행하고, `rollback`에서는 반복자에 `rev()`를 적용해 역순으로 되돌립니다. 명령마다 서로 다른 구조체와 상태를 가질 수 있지만, 공통 트레이트를 통해 한 컬렉션에서 다룰 수 있습니다.

## 접근법: 함수 포인터 사용하기

각 명령을 별도의 함수로 만들고 나중에 호출할 함수 포인터를 저장할 수도 있습니다. 함수 포인터는 `Fn`, `FnMut`, `FnOnce` 세 트레이트를 모두 구현하므로, 함수 포인터 대신 클로저를 전달하고 저장하는 방식도 선택할 수 있습니다.

```rust
type FnPtr = fn() -> String;
struct Command {
    execute: FnPtr,
    rollback: FnPtr,
}

struct Schema {
    commands: Vec<Command>,
}

impl Schema {
    fn new() -> Self {
        Self { commands: vec![] }
    }
    fn add_migration(&mut self, execute: FnPtr, rollback: FnPtr) {
        self.commands.push(Command { execute, rollback });
    }
    fn execute(&self) -> Vec<String> {
        self.commands.iter().map(|cmd| (cmd.execute)()).collect()
    }
    fn rollback(&self) -> Vec<String> {
        self.commands
            .iter()
            .rev()
            .map(|cmd| (cmd.rollback)())
            .collect()
    }
}

fn add_field() -> String {
    "add field".to_string()
}

fn remove_field() -> String {
    "remove field".to_string()
}

fn main() {
    let mut schema = Schema::new();
    schema.add_migration(|| "create table".to_string(), || "drop table".to_string());
    schema.add_migration(add_field, remove_field);
    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

각 `Command`는 실행 함수와 롤백 함수를 보관합니다. 이 예제에서는 캡처가 없는 클로저를 함수 포인터로 전달하고, 명명된 함수도 같은 방식으로 저장합니다.

## 접근법: `Fn` 트레이트 객체 사용하기

공통 명령 트레이트 대신, 각 명령을 `Fn` 트레이트 객체로 따로 저장할 수도 있습니다. 실행 클로저와 롤백 클로저를 각각 벡터에 보관합니다.

```rust
type Migration<'a> = Box<dyn Fn() -> &'a str>;

struct Schema<'a> {
    executes: Vec<Migration<'a>>,
    rollbacks: Vec<Migration<'a>>,
}

impl<'a> Schema<'a> {
    fn new() -> Self {
        Self {
            executes: vec![],
            rollbacks: vec![],
        }
    }
    fn add_migration<E, R>(&mut self, execute: E, rollback: R)
    where
        E: Fn() -> &'a str + 'static,
        R: Fn() -> &'a str + 'static,
    {
        self.executes.push(Box::new(execute));
        self.rollbacks.push(Box::new(rollback));
    }
    fn execute(&self) -> Vec<&str> {
        self.executes.iter().map(|cmd| cmd()).collect()
    }
    fn rollback(&self) -> Vec<&str> {
        self.rollbacks.iter().rev().map(|cmd| cmd()).collect()
    }
}

fn add_field() -> &'static str {
    "add field"
}

fn remove_field() -> &'static str {
    "remove field"
}

fn main() {
    let mut schema = Schema::new();
    schema.add_migration(|| "create table", || "drop table");
    schema.add_migration(add_field, remove_field);
    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

## 논의

명령이 작고 함수나 클로저로 표현할 수 있다면 함수 포인터가 더 적절할 수 있습니다. 동적 디스패치를 사용하지 않기 때문입니다. 반대로 명령이 여러 함수와 상태를 가진 구조체이고 관련 구현이 모듈로 분리되어 있다면 트레이트 객체가 더 잘 맞을 수 있습니다. 예를 들어 [`actix`](https://actix.rs/)는 경로에 핸들러 함수를 등록할 때 트레이트 객체를 사용합니다. `Fn` 트레이트 객체를 사용하는 방법도 함수 포인터 방식과 비슷하게 명령을 만들고 사용할 수 있습니다.

성능과 코드의 단순성·구조화 사이에는 항상 절충이 있습니다. 정적 디스패치는 더 빠른 성능을 제공하고, 동적 디스패치는 애플리케이션 구조를 유연하게 구성할 수 있게 합니다.

## 핵심 구성 요소

- **Command(명령):** 실행할 작업과 필요한 정보를 구조체, 열거형 또는 클로저로 나타냅니다. 실행 취소가 필요하다면 되돌리는 방법과 필요한 상태도 보관합니다.
- **Receiver(수신자):** 명령이 실제로 작업하는 대상입니다. 아래 예제에서는 텍스트를 보관하는 `Document`입니다.
- **Invoker(호출자):** 명령을 받아 실행하고 큐나 이력을 관리합니다. 아래 예제에서는 `Editor`가 실행과 Undo/Redo 스택을 관리합니다.

명령 객체는 실행 시점과 호출자를 분리하지만, 큐잉·로깅·재시도·Undo/Redo를 자동으로 제공하지는 않습니다. 각 기능의 저장 방식과 실패 시 동작은 별도로 구현해야 합니다. 특히 재시도는 작업이 중복 실행되어도 안전한지 확인해야 합니다.

## 예제: Undo/Redo가 가능한 텍스트 버퍼

`InsertCommand`와 `DeleteCommand`는 문서 수정과 복구 방법을 담습니다. `Editor`는 실행된 명령을 Undo 이력에 넣고, Undo한 명령을 Redo 이력으로 옮깁니다. Undo 후 새 명령이 성공하면 이전 Redo 이력은 삭제합니다.

위치는 문자 번호가 아니라 **UTF-8 바이트 오프셋**입니다. 범위 밖의 위치나 문자 중간 위치는 오류로 반환합니다. 예를 들어 `가`는 UTF-8에서 3바이트이므로 그 중간인 1이나 2는 유효한 경계가 아닙니다.

`src/main.rs`:

```rust
#[derive(Debug, Default)]
struct Document {
    content: String,
}

impl Document {
    fn insert_at(&mut self, pos: usize, text: &str) -> Result<(), EditError> {
        if !self.content.is_char_boundary(pos) {
            return Err(EditError::InvalidRange);
        }
        self.content.insert_str(pos, text);
        Ok(())
    }

    fn delete_range(&mut self, start: usize, end: usize) -> Result<String, EditError> {
        if start > end
            || !self.content.is_char_boundary(start)
            || !self.content.is_char_boundary(end)
        {
            return Err(EditError::InvalidRange);
        }
        Ok(self.content.drain(start..end).collect())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EditError {
    InvalidRange,
    NothingToRestore,
}

trait Command {
    fn execute(&mut self, doc: &mut Document) -> Result<(), EditError>;
    fn undo(&mut self, doc: &mut Document) -> Result<(), EditError>;
}

struct InsertCommand {
    pos: usize,
    text: String,
}

impl Command for InsertCommand {
    fn execute(&mut self, doc: &mut Document) -> Result<(), EditError> {
        doc.insert_at(self.pos, &self.text)
    }

    fn undo(&mut self, doc: &mut Document) -> Result<(), EditError> {
        let end = self.pos.checked_add(self.text.len()).ok_or(EditError::InvalidRange)?;
        doc.delete_range(self.pos, end)?;
        Ok(())
    }
}

struct DeleteCommand {
    start: usize,
    end: usize,
    deleted: Option<String>,
}

impl Command for DeleteCommand {
    fn execute(&mut self, doc: &mut Document) -> Result<(), EditError> {
        self.deleted = Some(doc.delete_range(self.start, self.end)?);
        Ok(())
    }

    fn undo(&mut self, doc: &mut Document) -> Result<(), EditError> {
        let text = self.deleted.as_deref().ok_or(EditError::NothingToRestore)?;
        doc.insert_at(self.start, text)
    }
}

#[derive(Default)]
struct Editor {
    doc: Document,
    history: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Editor {
    fn run(&mut self, mut cmd: Box<dyn Command>) -> Result<(), EditError> {
        cmd.execute(&mut self.doc)?;
        self.history.push(cmd);
        self.redo_stack.clear();
        Ok(())
    }

    fn undo(&mut self) -> Result<bool, EditError> {
        let Some(mut cmd) = self.history.pop() else {
            return Ok(false);
        };
        if let Err(error) = cmd.undo(&mut self.doc) {
            self.history.push(cmd);
            return Err(error);
        }
        self.redo_stack.push(cmd);
        Ok(true)
    }

    fn redo(&mut self) -> Result<bool, EditError> {
        let Some(mut cmd) = self.redo_stack.pop() else {
            return Ok(false);
        };
        if let Err(error) = cmd.execute(&mut self.doc) {
            self.redo_stack.push(cmd);
            return Err(error);
        }
        self.history.push(cmd);
        Ok(true)
    }
}

fn main() -> Result<(), EditError> {
    let mut editor = Editor::default();
    editor.run(Box::new(InsertCommand {
        pos: 0,
        text: "Hello, Rust!".to_string(),
    }))?;
    editor.run(Box::new(DeleteCommand {
        start: 7,
        end: 11,
        deleted: None,
    }))?;
    println!("삭제 후: {}", editor.doc.content); // Hello, !
    editor.undo()?;
    println!("Undo 후: {}", editor.doc.content); // Hello, Rust!
    editor.redo()?;
    println!("Redo 후: {}", editor.doc.content); // Hello, !
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert(pos: usize, text: &str) -> Box<dyn Command> {
        Box::new(InsertCommand { pos, text: text.into() })
    }

    fn delete(start: usize, end: usize) -> Box<dyn Command> {
        Box::new(DeleteCommand { start, end, deleted: None })
    }

    #[test]
    fn undo_and_redo_insert_and_delete_in_order() {
        let mut editor = Editor::default();
        editor.run(insert(0, "Hello, Rust!")).unwrap();
        editor.run(delete(7, 11)).unwrap();
        assert_eq!(editor.doc.content, "Hello, !");
        assert_eq!(editor.undo(), Ok(true));
        assert_eq!(editor.doc.content, "Hello, Rust!");
        assert_eq!(editor.undo(), Ok(true));
        assert_eq!(editor.doc.content, "");
        assert_eq!(editor.undo(), Ok(false));
        assert_eq!(editor.redo(), Ok(true));
        assert_eq!(editor.redo(), Ok(true));
        assert_eq!(editor.doc.content, "Hello, !");
        assert_eq!(editor.redo(), Ok(false));
    }

    #[test]
    fn new_successful_edit_clears_redo() {
        let mut editor = Editor::default();
        editor.run(insert(0, "abc")).unwrap();
        editor.undo().unwrap();
        editor.run(insert(0, "new")).unwrap();
        assert_eq!(editor.redo(), Ok(false));
        assert_eq!(editor.doc.content, "new");
    }

    #[test]
    fn invalid_utf8_boundaries_do_not_change_document_or_history() {
        let mut editor = Editor::default();
        editor.run(insert(0, "가나")).unwrap();
        assert_eq!(editor.run(delete(1, 3)), Err(EditError::InvalidRange));
        assert_eq!(editor.run(delete(3, 99)), Err(EditError::InvalidRange));
        assert_eq!(editor.run(delete(3, 0)), Err(EditError::InvalidRange));
        assert_eq!(editor.run(insert(2, "!")), Err(EditError::InvalidRange));
        assert_eq!(editor.doc.content, "가나");
        assert_eq!(editor.undo(), Ok(true));
        assert_eq!(editor.doc.content, "");
        assert_eq!(editor.undo(), Ok(false));
    }

    #[test]
    fn valid_multibyte_delete_can_be_undone_and_redone() {
        let mut editor = Editor::default();
        editor.run(insert(0, "가나")).unwrap();
        editor.run(delete(0, "가".len())).unwrap();
        assert_eq!(editor.doc.content, "나");
        editor.undo().unwrap();
        assert_eq!(editor.doc.content, "가나");
        editor.redo().unwrap();
        assert_eq!(editor.doc.content, "나");
    }

    #[test]
    fn failed_new_edit_preserves_redo() {
        let mut editor = Editor::default();
        editor.run(insert(0, "가")).unwrap();
        editor.undo().unwrap();
        assert_eq!(editor.run(insert(1, "!")), Err(EditError::InvalidRange));
        assert_eq!(editor.redo(), Ok(true));
        assert_eq!(editor.doc.content, "가");
    }
}
```

위 예제는 `cargo run`으로 실행하고, 같은 파일에 포함된 테스트는 `cargo test`로 실행합니다. 테스트는 실행·취소·재실행의 순서, 새 편집 후 Redo 이력 삭제, 잘못된 UTF-8 경계, 한글 편집 및 실패한 새 편집 후 Redo 유지 여부를 확인합니다.

여기서는 문서 변경을 모두 `Editor`의 명령으로만 수행하고, 각 명령이 오류를 반환할 때 문서와 명령 상태를 바꾸지 않는다고 가정합니다. 외부 구현을 허용한다면 이 계약을 지키도록 해야 합니다. 예를 들어 일부만 변경한 뒤 실패하는 명령은 이력만 복원해도 문서를 복구할 수 없으므로 별도의 트랜잭션이나 복구 정책이 필요합니다.

## 대안: 고정된 명령 집합은 열거형

명령 종류가 고정되어 있다면 `enum`과 `match`를 사용해 동적 디스패치 없이 모든 종류를 다룰 수 있습니다. `match`는 빠진 변형을 컴파일 시 확인합니다. 직렬화나 로깅에 활용하기도 좋지만, 실제 직렬화에는 `serde` 같은 별도 구현이나 라이브러리가 필요합니다.

```rust
enum EditorAction {
    Insert { pos: usize, text: String },
    Clear,
}

fn apply(doc: &mut String, action: EditorAction) -> Result<(), &'static str> {
    match action {
        EditorAction::Insert { pos, text } => {
            if !doc.is_char_boundary(pos) {
                return Err("유효하지 않은 UTF-8 위치");
            }
            doc.insert_str(pos, &text);
        }
        EditorAction::Clear => doc.clear(),
    }
    Ok(())
}
```

이 간단한 대안은 Undo/Redo나 직렬화를 구현하지 않습니다. 필요한 경우 각 동작에 복구 정보와 이력 관리를 추가해야 합니다.

## 대안: 일회성 클로저 작업 큐

캡처한 값을 소비하는 일회성 작업은 `Box<dyn FnOnce() + Send>`로 보관할 수 있습니다. `Send`는 작업을 다른 스레드로 이동해야 할 때 필요한 제약이며, 큐에 넣는 것만으로 작업이 비동기 실행되거나 다른 스레드에서 실행되는 것은 아닙니다.

```rust
fn enqueue_job(
    queue: &mut Vec<Box<dyn FnOnce() + Send>>,
    job: impl FnOnce() + Send + 'static,
) {
    queue.push(Box::new(job));
}

fn main() {
    let mut queue: Vec<Box<dyn FnOnce() + Send>> = Vec::new();
    let message = String::from("작업 실행");
    enqueue_job(&mut queue, move || println!("{message}"));

    for job in queue {
        job();
    }
}
```

`FnOnce` 작업은 호출 시 소비되므로 같은 작업을 그대로 재실행하거나 Undo할 수 없습니다. 반복 호출이 필요하면 상태와 수명에 맞춰 `FnMut` 등을 고려하고, 취소나 재시도가 필요하다면 그에 필요한 정보를 명시적으로 보관해야 합니다.

## 차용 검사기와 이력 관리

`Editor`가 문서와 이력을 서로 다른 필드로 보관하면 두 필드를 독립적으로 빌릴 수 있습니다. 따라서 두 필드를 동시에 사용한다고 해서 항상 차용 충돌이 발생하는 것은 아닙니다. 다만 전체 `self`를 빌리는 메서드를 호출하면서 이력 항목을 계속 빌려야 하는 등 빌림이 겹치는 설계에서는 충돌할 수 있습니다.

위 예제는 명령을 스택에서 `pop`해 소유한 뒤 `&mut self.doc`에 실행합니다. 성공하면 반대쪽 스택에 옮기고, 실패하면 원래 스택에 돌려놓습니다. 명령 실행의 성공 여부와 문서 변경을 함께 관리해야 이력이 실제 상태와 일치합니다.

## 참고 자료

- [Command 패턴](https://en.wikipedia.org/wiki/Command_pattern)
- [Command 패턴의 또 다른 예제](https://web.archive.org/web/20210223131236/https://chercher.tech/rust/command-design-pattern-rust)
- [Command 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/command)
- [Rust Book: 클로저](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [`String::is_char_boundary` 문서](https://doc.rust-lang.org/std/string/struct.String.html#method.is_char_boundary)
