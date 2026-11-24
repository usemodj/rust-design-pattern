# Composite 패턴

## 의도

Composite(컴포지트) 패턴은 객체를 트리 형태로 구성해 단일 객체와 객체의 묶음을 같은 인터페이스로 다루게 합니다. 클라이언트는 요소가 하나의 항목인지, 여러 하위 항목을 가진 컨테이너인지 구분하지 않고 공통 연산을 호출할 수 있습니다.

파일 시스템을 예로 들면 파일은 단일 항목(Leaf)이고 디렉터리는 파일과 다른 디렉터리를 포함하는 복합 객체(Composite)입니다. 파일과 디렉터리가 `size` 같은 공통 인터페이스를 제공하면, 클라이언트는 같은 호출로 파일 크기나 디렉터리 전체 크기를 얻을 수 있습니다.

## 구성 요소

- **컴포넌트(Component):** 단일 객체와 복합 객체가 공유하는 인터페이스를 정의합니다.
- **리프(Leaf):** 하위 컴포넌트를 갖지 않는 단일 객체입니다.
- **컴포지트(Composite):** 하위 컴포넌트를 보관하고, 공통 연산을 하위 항목에 재귀적으로 적용합니다.
- **클라이언트(Client):** 구체 타입 대신 컴포넌트 인터페이스를 사용합니다.

## Rust에서 표현하기

Rust에서는 컴포넌트의 공통 동작을 트레이트로 정의하고, 컴포지트가 `Vec<Box<dyn Component>>`처럼 트레이트 객체 컬렉션을 보유하도록 만들 수 있습니다. 컴포지트의 연산은 각 자식에게 위임하거나 그 결과를 집계합니다.

이 방식은 동적 디스패치와 힙 할당을 사용합니다. 컴포넌트의 모든 변형을 미리 알고 있다면 `enum`과 재귀적인 `Vec<Node>`로도 트리를 표현할 수 있습니다. 트레이트 객체를 사용할지는 런타임 확장성과 타입 안전성, 할당 비용을 고려해 선택합니다.

## Rust 예제: 파일과 디렉터리 트리

아래 예제에서는 `File`이 리프이고 `Directory`가 하위 항목을 가진 컴포지트입니다. 두 타입 모두 `FileSystemEntry`를 구현하므로 크기 집계와 트리 렌더링을 같은 인터페이스로 처리합니다.

`src/lib.rs`:

```rust
pub trait FileSystemEntry {
    fn name(&self) -> &str;
    fn size(&self) -> u64;
    fn render_tree(&self, depth: usize) -> String;
}

pub struct File {
    name: String,
    bytes: u64,
}

impl File {
    pub fn new(name: impl Into<String>, bytes: u64) -> Self {
        Self {
            name: name.into(),
            bytes,
        }
    }
}

impl FileSystemEntry for File {
    fn name(&self) -> &str {
        &self.name
    }

    fn size(&self) -> u64 {
        self.bytes
    }

    fn render_tree(&self, depth: usize) -> String {
        format!("{}{} ({} bytes)\n", "  ".repeat(depth), self.name, self.bytes)
    }
}

pub struct Directory {
    name: String,
    children: Vec<Box<dyn FileSystemEntry>>,
}

impl Directory {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: impl FileSystemEntry + 'static) {
        self.children.push(Box::new(entry));
    }
}

impl FileSystemEntry for Directory {
    fn name(&self) -> &str {
        &self.name
    }

    fn size(&self) -> u64 {
        self.children.iter().map(|child| child.size()).sum()
    }

    fn render_tree(&self, depth: usize) -> String {
        let mut rendered = format!("{}{}/\n", "  ".repeat(depth), self.name);

        for child in &self.children {
            rendered.push_str(&child.render_tree(depth + 1));
        }

        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::{Directory, File, FileSystemEntry};

    fn sample_tree() -> Directory {
        let mut source = Directory::new("src");
        source.add(File::new("main.rs", 12));

        let mut project = Directory::new("project");
        project.add(source);
        project.add(File::new("README.md", 5));
        project
    }

    #[test]
    fn directory_recursively_sums_child_sizes() {
        let project = sample_tree();

        assert_eq!(project.size(), 17);
    }

    #[test]
    fn leaf_and_composite_use_the_same_interface() {
        let entries: Vec<Box<dyn FileSystemEntry>> = vec![
            Box::new(File::new("LICENSE", 3)),
            Box::new(sample_tree()),
        ];

        let sizes: Vec<u64> = entries.iter().map(|entry| entry.size()).collect();

        assert_eq!(sizes, vec![3, 17]);
    }

    #[test]
    fn directory_renders_a_nested_tree() {
        let project = sample_tree();

        assert_eq!(
            project.render_tree(0),
            "project/\n  src/\n    main.rs (12 bytes)\n  README.md (5 bytes)\n"
        );
    }
}
```

`Directory::size`와 `Directory::render_tree`는 각 자식에게 같은 트레이트 메서드를 호출합니다. 자식이 파일이면 파일 동작이 실행되고, 디렉터리면 다시 그 하위 항목을 순회합니다. 따라서 트리의 깊이나 내부 구성을 클라이언트가 직접 분기하지 않아도 됩니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 확인합니다.

1. 디렉터리 크기가 중첩된 하위 파일의 크기까지 재귀적으로 합산합니다.
2. 파일과 디렉터리를 같은 트레이트 객체 컬렉션에 넣고 같은 `size` 인터페이스로 다룹니다.
3. 디렉터리가 중첩된 파일 시스템 트리를 예상한 형태로 렌더링합니다.

프로젝트 루트에서 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 클라이언트가 단일 객체와 복합 객체를 같은 방식으로 사용할 수 있습니다.
- 트리 구조의 연산과 순회를 컴포지트 내부에 캡슐화할 수 있습니다.
- 새 리프나 복합 타입을 추가해도 공통 트레이트를 따르는 한 기존 클라이언트 코드를 단순하게 유지할 수 있습니다.

### 단점

- 자식 구성 규칙이 복잡한 경우, 공통 인터페이스만으로 모든 제약을 표현하기 어렵습니다.
- `Box<dyn Trait>` 컬렉션은 힙 할당과 동적 디스패치를 사용합니다.
- 재귀 구조의 일부 연산은 깊은 트리에서 비용이 커지거나 스택 사용량이 늘어날 수 있습니다.
- 트리 구조가 필요하지 않은 문제에 적용하면 설계가 불필요하게 복잡해집니다.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Composite 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/composite)
- [Composite 패턴 — 위키백과](https://en.wikipedia.org/wiki/Composite_pattern)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [Rust Reference: 트레이트 객체 타입](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Bridge 패턴](/docs/bridge): 추상화와 구현의 변화 축을 분리하는 구조 패턴
