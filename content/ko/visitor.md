# Visitor 패턴

## 설명

Visitor는 서로 다른 타입의 객체가 섞여 있는 자료 구조에 적용할 알고리즘을 별도의 객체로 캡슐화합니다. 같은 자료 구조에 여러 알고리즘을 적용할 수 있고, 알고리즘을 추가할 때 자료 구조의 데이터 타입이나 기본 동작을 매번 수정하지 않아도 됩니다.

또한 자료 구조를 **어떻게 순회하는지**와 각 요소에 **어떤 연산을 수행하는지**를 분리합니다. 예를 들어 추상 구문 트리(AST)를 그대로 두고 인터프리터, 타입 검사기, 코드 출력기 같은 서로 다른 방문자를 구현할 수 있습니다.

## 예제: AST 인터프리터

다음 예제의 AST는 문장(`Stmt`), 이름(`Name`), 식(`Expr`)으로 이루어집니다. 문장은 식 문장이거나 변수 선언이며, 식은 정수 리터럴 또는 두 식을 더하거나 빼는 연산입니다. `Box<Expr>`는 식이 재귀적으로 다른 식을 포함하는 트리 구조를 만듭니다.

```rust,ignore
use std::collections::HashMap;

// The data we will visit
mod ast {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Name {
        value: String,
    }

    impl Name {
        pub fn new(value: impl Into<String>) -> Self {
            Self {
                value: value.into(),
            }
        }

        pub fn as_str(&self) -> &str {
            &self.value
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Stmt {
        Expr(Expr),
        Let(Name, Expr),
        Block(Vec<Stmt>),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Expr {
        IntLit(i64),
        Var(Name),
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
    }
}

// The abstract visitor
mod visit {
    use super::ast::*;

    pub trait Visitor<T> {
        fn visit_name(&mut self, n: &Name) -> T;
        fn visit_stmt(&mut self, s: &Stmt) -> T;
        fn visit_expr(&mut self, e: &Expr) -> T;
    }
}

use ast::*;
use visit::*;

// An example concrete implementation - walks the AST interpreting it as code.
#[derive(Default)]
struct Interpreter {
    // 블록 스코프를 지원하기 위한 환경 스택 (가장 안쪽이 top)
    scopes: Vec<HashMap<String, i64>>,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn set_var(&mut self, name: &str, val: i64) {
        if let Some(current) = self.scopes.last_mut() {
            current.insert(name.to_string(), val);
        }
    }

    fn get_var(&self, name: &str) -> Option<i64> {
        for scope in self.scopes.iter().rev() {
            if let Some(&val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }
}

impl Visitor<i64> for Interpreter {
    fn visit_name(&mut self, n: &Name) -> i64 {
        self.get_var(n.as_str())
            .unwrap_or_else(|| panic!("undefined variable: {}", n.as_str()))
    }

    fn visit_stmt(&mut self, s: &Stmt) -> i64 {
        match s {
            Stmt::Expr(e) => self.visit_expr(e),
            Stmt::Let(name, expr) => {
                let val = self.visit_expr(expr);
                self.set_var(name.as_str(), val);
                val
            }
            Stmt::Block(stmts) => {
                self.scopes.push(HashMap::new());
                let mut last_val = 0;
                for stmt in stmts {
                    last_val = self.visit_stmt(stmt);
                }
                self.scopes.pop();
                last_val
            }
        }
    }

    fn visit_expr(&mut self, e: &Expr) -> i64 {
        match e {
            Expr::IntLit(n) => *n,
            Expr::Var(name) => self.visit_name(name),
            Expr::Add(lhs, rhs) => self.visit_expr(lhs) + self.visit_expr(rhs),
            Expr::Sub(lhs, rhs) => self.visit_expr(lhs) - self.visit_expr(rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(s: &str) -> Name {
        Name::new(s)
    }

    fn int(n: i64) -> Expr {
        Expr::IntLit(n)
    }

    fn var(s: &str) -> Expr {
        Expr::Var(Name::new(s))
    }

    fn add(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Add(Box::new(lhs), Box::new(rhs))
    }

    fn sub(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Sub(Box::new(lhs), Box::new(rhs))
    }

    #[test]
    fn eval_int_literal() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.visit_expr(&int(42)), 42);
        assert_eq!(interp.visit_expr(&int(-7)), -7);
    }

    #[test]
    fn eval_add_and_sub_expressions() {
        let mut interp = Interpreter::new();
        // (10 + 5) - 3 = 12
        let expr = sub(add(int(10), int(5)), int(3));
        assert_eq!(interp.visit_expr(&expr), 12);
    }

    #[test]
    fn eval_let_and_variable_lookup() {
        let mut interp = Interpreter::new();

        assert_eq!(interp.visit_stmt(&Stmt::Let(name("x"), int(10))), 10);
        assert_eq!(
            interp.visit_stmt(&Stmt::Let(name("y"), add(var("x"), int(5)))),
            15
        );
        assert_eq!(interp.visit_stmt(&Stmt::Expr(sub(var("y"), int(3)))), 12);
        assert_eq!(interp.visit_name(&name("x")), 10);
        assert_eq!(interp.visit_name(&name("y")), 15);
    }

    #[test]
    fn eval_variable_shadowing_in_inner_block() {
        let mut interp = Interpreter::new();

        let program = Stmt::Block(vec![
            Stmt::Let(name("x"), int(100)),
            Stmt::Block(vec![
                Stmt::Let(name("x"), int(20)),
                Stmt::Expr(add(var("x"), int(1))), // 내부 x(20) + 1 = 21
            ]),
            Stmt::Expr(var("x")), // 블록 종료 후 바깥 x(100) 유지
        ]);

        println!("program: {}", interp.visit_stmt(&program));
        assert_eq!(interp.visit_stmt(&program), 100);
    }

    #[test]
    #[should_panic(expected = "undefined variable: unknown")]
    fn visit_undefined_variable_panics() {
        let mut interp = Interpreter::new();
        interp.visit_name(&name("unknown"));
    }

    #[test]
    #[should_panic(expected = "undefined variable: inner_only")]
    fn variable_does_not_leak_outside_block() {
        let mut interp = Interpreter::new();

        let program = Stmt::Block(vec![
            Stmt::Block(vec![Stmt::Let(name("inner_only"), int(99))]),
            Stmt::Expr(var("inner_only")),
        ]);

        interp.visit_stmt(&program);
    }
}

```

`Visitor<T>`는 각 종류의 노드를 방문하는 메서드를 정의합니다. 방문 메서드가 반환하는 `T`는 방문자가 각 노드를 처리한 결과 타입입니다. 여기서는 인터프리터가 식의 계산 결과를 `i64`로 반환하므로 `Interpreter`는 `Visitor<i64>`를 구현합니다.

`visit_stmt`는 식 문장을 만나면 해당 식의 평가를 위임하고, 변수 선언은 이 간단한 인터프리터에서 아직 구현하지 않아 `unimplemented!()`로 남겨 둡니다. `visit_expr`는 정수 리터럴을 그대로 반환하며, 덧셈과 뺄셈 노드에서는 왼쪽과 오른쪽 하위 식을 재귀적으로 방문한 뒤 결과를 계산합니다. 이름 처리 역시 변수 환경이 없으므로 `panic!()`으로 표시되어 있습니다. 따라서 이 코드는 Visitor 패턴의 구조를 보여 주는 예제이지, 변수 선언과 이름 평가까지 지원하는 완성된 언어 구현은 아닙니다.

AST는 그대로 둔 채 추가 방문자를 구현할 수 있습니다. 예를 들어 별도의 타입 검사 방문자는 각 노드의 타입을 검사하고, 출력 방문자는 AST를 텍스트로 변환할 수 있습니다.

## 동기

서로 다른 타입의 데이터가 섞인 구조 전체에 알고리즘을 적용하고 싶을 때 Visitor가 유용합니다. 데이터가 모두 같은 타입이라면 반복자와 비슷한 순회 방식으로 충분할 수 있습니다.

함수형 접근 대신 방문자 객체를 사용하는 중요한 이유 중 하나는 방문자가 상태를 가질 수 있다는 점입니다. 한 노드에서 얻은 정보를 다음 노드의 처리에 전달할 수 있으므로, 예를 들어 타입 검사 과정의 문맥이나 변수 정보를 방문자 내부에 저장할 수 있습니다.

## 논의

`visit_*` 메서드는 예제처럼 값을 반환하기보다 `()`(void)를 반환하는 경우가 흔합니다. 메서드가 별도의 결과를 돌려주지 않는다면 트리 순회 코드를 방문자마다 반복하지 않고 공통 함수로 분리할 수 있습니다. 이 공통 함수는 방문자가 기본 동작을 재사용하도록 하거나, 특정 노드에 아무 작업도 하지 않는 기본 메서드를 제공하는 데에도 활용할 수 있습니다.

Rust에서는 일반적으로 각 데이터 종류에 대응하는 `walk_*` 함수를 둡니다. 다음 `walk_expr`는 정수 리터럴에서는 더 내려갈 노드가 없으므로 아무것도 하지 않고, 덧셈·뺄셈 노드에서는 두 자식 식을 차례로 방문합니다.

```rust,ignore
pub fn walk_expr(visitor: &mut Visitor, e: &Expr) {
    match *e {
        Expr::IntLit(_) => {}
        Expr::Add(ref lhs, ref rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        Expr::Sub(ref lhs, ref rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
    }
}
```

방문자는 `visit_expr`에서 이 함수를 호출해 기본 하위 노드 순회를 재사용하면서, 필요하면 노드별 추가 작업을 수행할 수 있습니다. 다른 언어에서는 같은 역할을 데이터 타입의 `accept` 메서드가 맡는 경우가 흔합니다. `accept`가 방문자를 받아 적절한 방문 메서드를 호출해 순회를 위임하는 방식입니다.

## 참고 자료

- [Visitor 패턴](https://en.wikipedia.org/wiki/Visitor_pattern)
- [Fold 패턴](/docs/fold): Visitor와 비슷하지만 방문한 자료 구조의 새 버전을 만들어 반환합니다.
