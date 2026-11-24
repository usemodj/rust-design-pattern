# Iterator 패턴

## 의도

Iterator(이터레이터) 패턴은 컬렉션의 내부 저장 구조를 직접 다루지 않고 요소에 순차적으로 접근하는 방법을 제공합니다. 순회 상태를 컬렉션과 분리하므로 클라이언트는 인덱스나 저장 방식 대신 다음 요소를 요청하는 공통 인터페이스를 사용할 수 있습니다.

구성 요소는 다음과 같습니다.

- **컬렉션:** 순회할 데이터를 보관하고 이터레이터를 제공합니다.
- **이터레이터:** 순회 위치를 보관하고 다음 요소를 반환합니다.
- **클라이언트:** 컬렉션의 내부 구조 대신 이터레이터를 통해 요소를 사용합니다.

컬렉션은 용도에 따라 서로 다른 순회 방법을 제공할 수 있습니다. 예를 들어 모든 항목을 순회하거나, 조건에 맞는 항목만 반환할 수 있습니다. 별도로 생성한 이터레이터는 각자의 순회 위치를 가집니다.

## Rust에서 표현하기

Rust 표준 라이브러리의 `Iterator` 트레이트는 연관 타입 `Item`과 `next(&mut self) -> Option<Self::Item>`을 정의합니다. `next`는 다음 값이 있으면 `Some`을, 현재 더 이상 반환할 값이 없으면 `None`을 반환합니다. 일반적으로 순회가 끝나면 이후에도 `None`을 반환하도록 구현하지만, `Iterator` 자체는 이를 보장하지 않습니다. 이 보장이 필요하면 `FusedIterator`를 사용하거나 `.fuse()` 어댑터를 적용할 수 있습니다.

`for` 문은 `IntoIterator`를 통해 이터레이터를 얻어 사용합니다. 이미 `Iterator`를 구현한 값은 `for`에서 바로 사용할 수 있으며, `Vec`처럼 컬렉션에서 소유한 값이나 빌린 값을 순회할 수도 있습니다. `iter()`는 보통 공유 참조, `iter_mut()`는 가변 참조, `into_iter()`는 대상에 따라 소유한 값 또는 참조를 반환합니다. 빌린 이터레이터는 원본보다 오래 살 수 없습니다.

`map`과 `filter` 같은 어댑터는 새로운 이터레이터를 만듭니다. 이터레이터는 지연 평가되므로 일반적으로 `collect`, `sum`, `for`처럼 실제로 소비할 때 각 요소를 처리합니다. 사용자 정의 순회가 필요하지 않다면 표준 이터레이터와 어댑터를 조합하는 편이 간결합니다.

## Rust 예제: 대출 가능한 책 순회하기

도서 목록을 빌려 대출 가능한 책만 순서대로 반환하는 이터레이터입니다. 이터레이터는 책을 복사하거나 소유하지 않고 컬렉션의 책을 참조합니다.

`src/lib.rs`:

```rust
pub struct Book {
    pub title: String,
    pub available: bool,
}

pub struct Library {
    books: Vec<Book>,
}

impl Library {
    pub fn new(books: Vec<Book>) -> Self {
        Self { books }
    }

    pub fn available_books(&self) -> AvailableBooks<'_> {
        AvailableBooks {
            books: self.books.iter(),
        }
    }
}

pub struct AvailableBooks<'a> {
    books: std::slice::Iter<'a, Book>,
}

impl<'a> Iterator for AvailableBooks<'a> {
    type Item = &'a Book;

    fn next(&mut self) -> Option<Self::Item> {
        self.books.find(|book| book.available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> Library {
        Library::new(vec![
            Book { title: "Rust 입문".into(), available: true },
            Book { title: "알고리즘".into(), available: false },
            Book { title: "디자인 패턴".into(), available: true },
        ])
    }

    #[test]
    fn returns_available_books_in_order() {
        let library = library();
        let titles: Vec<&str> = library
            .available_books()
            .map(|book| book.title.as_str())
            .collect();

        assert_eq!(titles, vec!["Rust 입문", "디자인 패턴"]);
    }

    #[test]
    fn independent_iterators_have_independent_positions() {
        let library = library();
        let mut first = library.available_books();
        let mut second = library.available_books();

        assert_eq!(first.next().map(|book| book.title.as_str()), Some("Rust 입문"));
        assert_eq!(first.next().map(|book| book.title.as_str()), Some("디자인 패턴"));
        assert_eq!(second.next().map(|book| book.title.as_str()), Some("Rust 입문"));
    }

    #[test]
    fn returns_none_after_the_last_available_book() {
        let library = library();
        let mut books = library.available_books();

        assert!(books.next().is_some());
        assert!(books.next().is_some());
        assert!(books.next().is_none());
        assert!(books.next().is_none());
    }

    #[test]
    fn empty_or_unavailable_books_produce_no_items() {
        let empty = Library::new(vec![]);
        let unavailable = Library::new(vec![
            Book { title: "대출 중".into(), available: false },
        ]);

        assert_eq!(empty.available_books().count(), 0);
        assert_eq!(unavailable.available_books().count(), 0);
    }
}
```

`AvailableBooks`는 내부 슬라이스 이터레이터의 `find`로 대출 가능한 다음 책을 찾습니다. 조건에 맞지 않는 책은 건너뛰고, 더 이상 책이 없으면 `None`을 반환합니다. 이 예제는 내부 슬라이스가 끝난 후 계속 `None`을 반환하지만 `FusedIterator` 트레이트를 별도로 구현하지는 않습니다.

## 테스트 코드

위 `src/lib.rs`의 테스트는 대출 가능한 책의 순서와 필터링, 독립적인 순회 위치, 순회 종료 후 반복 호출, 빈 목록과 모두 대출 중인 목록을 확인합니다. 예제 코드를 Rust 라이브러리 프로젝트의 `src/lib.rs`에 저장한 뒤 프로젝트 루트에서 실행하세요.

```console
cargo test
```

## 장점과 단점

### 장점

- 순회하는 코드가 컬렉션의 저장 방식과 직접 결합되지 않습니다.
- 독립적인 이터레이터를 만들어 같은 컬렉션을 서로 다른 위치에서 탐색할 수 있습니다.
- 표준 어댑터를 조합해 필터링과 변환을 명확하게 표현할 수 있습니다.

### 단점

- 단순한 순회에 별도 타입을 만들면 코드가 불필요하게 복잡해질 수 있습니다.
- 지연 평가로 인해 실제 연산 시점이나 부수 효과를 놓치기 쉽습니다.
- 이터레이터가 원본을 빌리는 동안에는 Rust의 빌림 규칙에 따라 원본을 변경할 수 없는 경우가 있습니다.

## 참고 자료

- [행동 패턴 개요](/docs/behavioral-patterns)
- [Iterator 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/iterator)
- [Iterator 패턴 — 위키백과](https://en.wikipedia.org/wiki/Iterator_pattern)
- [Rust Book: 이터레이터로 일련의 항목 처리하기](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [`Iterator` 표준 라이브러리 문서](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [`IntoIterator` 표준 라이브러리 문서](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
- [`FusedIterator` 표준 라이브러리 문서](https://doc.rust-lang.org/std/iter/trait.FusedIterator.html)
