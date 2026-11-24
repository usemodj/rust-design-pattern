# Fold 패턴

## 설명

Fold 패턴은 자료 구조의 각 항목이나 노드에 알고리즘을 적용해 변환된 항목을 만들고, 그 결과로 새로운 자료 구조를 구성하는 방법입니다. 기존 자료 구조를 단순히 방문해 값을 계산하거나 내부에서 수정하는 대신, 변환된 구조를 별도로 생성합니다.

이 패턴에서 *fold*라는 이름을 사용하는 이유는 명확하지 않습니다. Rust 컴파일러 코드에서도 `fold`, `Folder`라는 용어를 사용하지만, 원문의 저자는 반복자에서 흔히 말하는 fold보다는 각 요소를 변환하는 `map`에 더 가까워 보인다고 설명합니다. 아래의 논의에서 두 의미의 차이를 살펴봅니다.

## 예제: AST 이름 바꾸기

다음 예제는 간단한 추상 구문 트리(AST)를 정의하고, `Folder` 트레이트를 이용해 AST의 노드를 순회하면서 변환합니다.

```rust,ignore
// The data we will fold, a simple AST.
mod ast {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Stmt {
        Expr(Box<Expr>),
        Let(Box<Name>, Box<Expr>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Name {
        pub(crate) value: String,
    }

    impl Name {
        pub fn new(value: impl Into<String>) -> Self {
            Self {
                value: value.into(),
            }
        }

        pub fn value(&self) -> &str {
            &self.value
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Expr {
        IntLit(i64),
        Var(Box<Name>),
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
    }
}

// The abstract folder
mod fold {
    use super::ast::*;

    pub trait Folder {
        // A leaf node just returns the node itself. In some cases, we can do this
        // to inner nodes too.
        fn fold_name(&mut self, n: Box<Name>) -> Box<Name> {
            n
        }
        // Create a new inner node by folding its children.
        fn fold_stmt(&mut self, s: Box<Stmt>) -> Box<Stmt> {
            match *s {
                Stmt::Expr(e) => Box::new(Stmt::Expr(self.fold_expr(e))),
                Stmt::Let(n, e) => Box::new(Stmt::Let(self.fold_name(n), self.fold_expr(e))),
            }
        }
        fn fold_expr(&mut self, e: Box<Expr>) -> Box<Expr> {
            match *e {
                Expr::IntLit(n) => Box::new(Expr::IntLit(n)),
                Expr::Var(n) => Box::new(Expr::Var(self.fold_name(n))),
                Expr::Add(lhs, rhs) => {
                    Box::new(Expr::Add(self.fold_expr(lhs), self.fold_expr(rhs)))
                }
                Expr::Sub(lhs, rhs) => {
                    Box::new(Expr::Sub(self.fold_expr(lhs), self.fold_expr(rhs)))
                }
            }
        }
    }
}

use ast::*;
use fold::*;

// An example concrete implementation - renames every name to 'foo'.
struct Renamer;
impl Folder for Renamer {
    fn fold_name(&mut self, _n: Box<Name>) -> Box<Name> {
        Box::new(Name {
            value: "foo".to_owned(),
        })
    }
    // Use the default methods for the other nodes.
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(n: i64) -> Box<Expr> {
        Box::new(Expr::IntLit(n))
    }

    fn var(s: &str) -> Box<Expr> {
        Box::new(Expr::Var(Box::new(Name::new(s))))
    }

    fn add(lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Add(lhs, rhs))
    }

    fn sub(lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Sub(lhs, rhs))
    }

    #[test]
    fn renames_let_binding_and_expr_vars_to_foo() {
        let mut renamer = Renamer;
        // let total = (a + 10) - b;
        let stmt = Box::new(Stmt::Let(
            Box::new(Name::new("total")),
            sub(add(var("a"), int(10)), var("b")),
        ));

        let folded = renamer.fold_stmt(stmt);

        let expected = Box::new(Stmt::Let(
            Box::new(Name::new("foo")),
            sub(add(var("foo"), int(10)), var("foo")),
        ));
        assert_eq!(folded, expected);
    }

    #[test]
    fn renames_nested_vars_inside_expr() {
        let mut renamer = Renamer;
        let expr = add(var("x"), sub(var("y"), int(5)));

        let folded = renamer.fold_expr(expr);

        let expected = add(var("foo"), sub(var("foo"), int(5)));
        assert_eq!(folded, expected);
    }

    #[test]
    fn renames_let_binding_name_to_foo() {
        let mut renamer = Renamer;
        let stmt = Box::new(Stmt::Let(
            Box::new(Name::new("original_var")),
            add(int(10), int(5)),
        ));

        let folded = renamer.fold_stmt(stmt);

        let expected = Box::new(Stmt::Let(
            Box::new(Name::new("foo")),
            add(int(10), int(5)),
        ));
        assert_eq!(folded, expected);
    }

    #[test]
    fn folds_nested_expr_without_changing_literals() {
        let mut renamer = Renamer;
        let expr = sub(add(int(20), int(7)), int(3));

        let folded = renamer.fold_expr(expr.clone());

        assert_eq!(folded, expr);
    }

    #[test]
    fn folds_expr_stmt_directly() {
        let mut renamer = Renamer;
        let stmt = Box::new(Stmt::Expr(int(42)));

        let folded = renamer.fold_stmt(stmt);

        assert_eq!(folded, Box::new(Stmt::Expr(int(42))));
    }

    #[test]
    fn default_folder_preserves_name() {
        struct IdentityFolder;
        impl Folder for IdentityFolder {}

        let mut folder = IdentityFolder;
        let name = Box::new(Name::new("keep_me"));

        let folded = folder.fold_name(name);
        assert_eq!(folded.value(), "keep_me");
    }
}
```

`Folder`의 기본 `fold_name`은 잎 노드인 이름을 변경하지 않고 그대로 반환합니다. 기본 `fold_stmt`는 문장 종류를 확인한 뒤, 하위 식과 이름을 해당 `fold_*` 메서드에 넘겨 변환하고 그 결과로 새 문장 노드를 만듭니다. 실제 구현에서 `fold_expr`는 정수 리터럴은 그대로 반환하고 덧셈·뺄셈 노드의 자식 식을 재귀적으로 접도록 정의할 수 있습니다. 예제에서는 핵심 구조만 보이도록 구현을 생략했습니다.

`Renamer`는 `fold_name`만 재정의해 이름을 모두 `"foo"`로 바꿉니다. 나머지 노드는 `Folder`의 기본 동작을 사용합니다. 이 방문자를 AST에 적용한 결과는 원래 구조와 같지만 모든 이름이 바뀐 새 AST입니다. 실제 Folder는 방문자 객체 안에 상태를 보관할 수도 있습니다. 예를 들어 앞서 처리한 노드의 정보를 사용해 뒤에 나오는 노드를 변환할 수 있습니다.

Fold는 같은 구조를 변환하는 데만 국한되지 않습니다. 한 자료 구조를 다른 종류의 자료 구조로 바꿀 수도 있습니다. 예를 들어 AST를 고수준 중간 표현(HIR, High-level Intermediate Representation) 트리로 변환하는 Folder를 만들 수 있습니다.

## 동기

복합 자료 구조의 각 노드에 연산을 적용해 새 구조를 만드는 일은 흔합니다. 자료 구조가 단순하고 각 요소에 독립적으로 적용할 연산이라면 `Iterator::map`으로 충분할 수 있습니다.

반면 다음과 같은 경우에는 Fold가 더 적합합니다.

- 이전 노드에서 얻은 정보가 이후 노드의 변환에 영향을 주는 경우
- 자료 구조를 순회하는 과정 자체가 단순한 반복으로 표현되지 않는 경우
- 트리처럼 노드마다 하위 노드의 형태가 달라 재귀적인 순회가 필요한 경우

Visitor 패턴처럼 Fold도 자료 구조를 **순회하는 방법**과 각 노드에 **수행할 작업**을 분리합니다. 차이는 Fold가 일반적으로 변환된 새 자료 구조를 만들어 반환한다는 점입니다.

## 논의

이런 방식의 자료 구조 변환은 함수형 프로그래밍 언어에서 흔합니다. 객체 지향 언어에서는 기존 자료 구조를 제자리에서 변경하는 경우가 더 많습니다. Rust에서는 불변성을 선호하는 경향 때문에 기존 값을 직접 바꾸기보다 새로운 자료 구조를 만드는 함수형 접근을 자주 사용합니다. 새 값을 만들면 많은 상황에서 이전 값을 언제, 어디서 변경했는지 추적할 필요가 줄어 코드를 이해하기 쉬워집니다.

다만 새 자료 구조를 만드는 방식은 효율성과 재사용성 사이에 절충이 있습니다. 각 `fold_*` 메서드가 노드를 소유해서 받는지, 빌린 참조로 받는지, 참조 카운트 포인터로 받는지에 따라 비용과 재사용 방식이 달라집니다.

### 소유한 노드(`Box<T>`)를 받는 경우

예제처럼 `Box<T>`를 받으면 Folder가 노드의 소유권을 가져옵니다. 따라서 원래 자료 구조를 그대로 유지한 채 재사용할 수는 없습니다. 대신 노드가 바뀌지 않았다면 소유권을 그대로 반환해 해당 노드를 복제하지 않고 재사용할 수 있어 효율적입니다. 원본 구조를 소비해도 되는 변환에 적합합니다.

### 빌린 참조를 받는 경우

`&T` 같은 빌린 참조를 받으면 원본 자료 구조를 유지할 수 있습니다. 하지만 결과 자료 구조도 새로 만들어야 하므로 변경되지 않은 노드까지 복제해야 할 수 있습니다. 노드가 크거나 복제가 비싸다면 이 비용이 문제가 될 수 있습니다.

### 참조 카운트 포인터를 사용하는 경우

`Rc<T>`나 `Arc<T>` 같은 참조 카운트 포인터를 사용하면 원본 구조를 보존하면서 변경되지 않은 노드를 공유할 수 있습니다. 공유 포인터를 복제하면 실제 노드 전체를 복사하는 대신 참조 카운트가 증가하므로, 변경되지 않은 노드를 복제하지 않고 재사용할 수 있습니다.

대신 참조 카운트 포인터는 소유권을 직접 다루는 방식보다 사용이 덜 간결할 수 있고, 공유되는 자료 구조를 제자리에서 변경할 수 없게 합니다. 따라서 원본을 보존하면서 노드 공유가 중요한 경우에 이 방식을 고려하고, 필요하지 않다면 더 단순한 소유권 모델을 선택하는 것이 좋습니다.

## 다른 `fold` 연산과의 차이

Rust 반복자의 `fold` 메서드는 컬렉션의 요소를 누적해 **하나의 누적값**을 만듭니다. 예를 들어 숫자 반복자에서 합계를 계산하면 여러 숫자가 하나의 숫자로 접힙니다. 이 문서의 Fold 패턴은 각 노드를 변환해 **새 자료 구조**를 만드는 방식입니다.

반복자의 `map`은 각 요소를 변환해 새로운 반복 결과를 만든다는 점에서 이 패턴의 의미에 더 가깝습니다. 다른 언어에서도 `fold`는 보통 Rust 반복자의 누산 연산과 비슷한 의미로 사용되지만, 일부 함수형 언어는 자료 구조를 유연하게 변환하는 강력한 연산도 제공합니다.

## Visitor 패턴과의 관계

Visitor와 Fold는 모두 자료 구조를 순회하면서 각 노드에 작업을 수행한다는 개념을 공유합니다. 하지만 Visitor는 보통 기존 자료 구조를 소비하지도, 새 자료 구조를 만들지도 않습니다. 기존 구조를 유지하면서 검사나 해석 같은 작업을 수행하는 데 적합합니다. Fold는 노드를 변환하고 그 결과로 새 자료 구조를 만듭니다.

## 참고 자료

- [Fold 패턴 — Rust Design Patterns 원문](https://rust-unofficial.github.io/patterns/patterns/creational/fold.html)
- [Visitor 패턴](/docs/visitor)
- [`Iterator::map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map)
- [`Iterator::fold`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)
- [고수준 중간 표현(HIR) — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/hir.html)
