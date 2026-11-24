# Interpreter 패턴

## 설명

자주 반복해서 마주치는 문제가 있고, 해결에 길고 반복적인 절차가 필요하다면 문제의 각 사례를 간단한 언어로 표현하고 그 언어로 작성된 문장을 해석하는 인터프리터를 만들 수 있습니다. 그러면 문제마다 절차를 처음부터 다시 기술하는 대신, 문제를 해당 언어의 문장으로 나타내고 인터프리터가 이를 처리하도록 할 수 있습니다.

일반적으로 다음을 정의합니다.

- 해결하려는 문제 영역에 맞춘 [도메인 특화 언어(DSL)](https://en.wikipedia.org/wiki/Domain-specific_language)
- DSL이 허용하는 문장과 구조를 설명하는 문법
- DSL로 표현된 문제를 해석하고 해결하는 인터프리터

## 동기

간단한 수학식을 후위 표기법(Postfix notation), 즉 [역폴란드 표기법(Reverse Polish notation)](https://en.wikipedia.org/wiki/Reverse_Polish_notation)으로 바꾸는 문제를 살펴보겠습니다.

이 예제에서는 입력에 숫자 `0`부터 `9`까지와 두 연산자 `+`, `-`만 허용합니다. 중위 표기법에서는 연산자가 피연산자 사이에 오므로 `2 + 4`처럼 쓰지만, 후위 표기법에서는 연산자를 피연산자 뒤에 놓아 `2 4 +`로 씁니다. 공백을 제외하면 `2+4`는 `24+`가 됩니다.

## 문제의 문맥 자유 문법

중위 수식을 후위 수식으로 변환하려면 우선 올바른 입력이 어떤 모양인지 정의해야 합니다. 이를 위해 `0`부터 `9`, `+`, `-`로 이루어진 수식의 문맥 자유 문법을 정의합니다.

- **단말 기호(terminal symbols)**: 입력에 실제로 나타나는 `0`부터 `9`, `+`, `-`
- **비단말 기호(non-terminal symbols)**: 문법 규칙으로 확장되는 `exp`, `term`
- **시작 기호(start symbol)**: 전체 수식을 나타내는 `exp`
- **생성 규칙(production rules)**: 각 비단말 기호가 어떤 형태로 구성될 수 있는지를 정하는 규칙

```ignore
exp -> exp + term
exp -> exp - term
exp -> term
term -> 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
```

`term`은 한 자리 숫자 하나이고, `exp`는 항 하나이거나 기존 수식 뒤에 `+ term` 또는 `- term`을 붙인 형태입니다. 따라서 `2+3-4`와 같은 식은 문법에 맞지만, 피연산자가 빠진 `2+`나 항 사이에 연산자가 없는 `2-34`는 이 문법에 맞지 않습니다.

이 문법에는 `exp -> exp + term`처럼 `exp`가 자기 자신으로 시작하는 **좌재귀(left recursion)**가 있습니다. 문법을 어떤 방식으로 처리하느냐에 따라 이 규칙을 그대로 사용할 수 없고, 재귀 하강 파서를 작성하기 전에 좌재귀를 제거하는 등 문법을 변환해야 할 수 있습니다. 문법 변환과 파서 구성에 대한 자세한 내용은 컴파일러 분야의 표준 교재인 [Compilers: Principles, Techniques, and Tools](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)(일명 *Dragon Book*)를 참고하세요.

## 해결 방법

여기서는 간단한 재귀 하강 파서(recursive descent parser)를 구현합니다. 코드는 먼저 `term` 하나를 읽고, 그 뒤에 연산자가 있으면 다음 `term`을 읽은 다음 연산자를 출력 문자열에 추가합니다. 따라서 `2+3`을 읽으면 숫자 `2`, 숫자 `3`, 연산자 `+` 순서로 출력하여 `23+`를 만듭니다.

예제를 간결하게 보이기 위해 문법에 맞지 않는 입력은 오류를 반환하는 대신 패닉을 발생시킵니다. 예를 들어 `2-34`에서는 `2-3`을 처리한 다음 연산자가 와야 할 자리에 `4`가 나타나므로 패닉이 발생합니다. `2+5-`에서는 `-` 뒤에 필요한 항을 읽으려 하지만 입력이 끝나므로 패닉이 발생합니다.

```rust
pub struct Interpreter<'a> {
    it: std::str::Chars<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(infix: &'a str) -> Self {
        Self { it: infix.chars() }
    }

    fn next_char(&mut self) -> Option<char> {
        self.it.next()
    }

    pub fn interpret(&mut self, out: &mut String) {
        self.term(out);

        while let Some(op) = self.next_char() {
            if op == '+' || op == '-' {
                self.term(out);
                out.push(op);
            } else {
                panic!("Unexpected symbol '{op}'");
            }
        }
    }

    fn term(&mut self, out: &mut String) {
        match self.next_char() {
            Some(ch) if ch.is_ascii_digit() => out.push(ch),
            Some(ch) => panic!("Unexpected symbol '{ch}'"),
            None => panic!("Unexpected end of string"),
        }
    }
}

pub fn main() {
    let mut intr = Interpreter::new("2+3");
    let mut postfix = String::new();
    intr.interpret(&mut postfix);
    assert_eq!(postfix, "23+");

    intr = Interpreter::new("1-2+3-4");
    postfix.clear();
    intr.interpret(&mut postfix);
    assert_eq!(postfix, "12-3+4-");
}
```

`Interpreter`는 입력 문자열의 문자 반복자인 `Chars`를 보관합니다. `next_char`는 다음 문자를 하나씩 소비하고, `term`은 문자가 ASCII 숫자인지 확인해 결과에 추가합니다. `interpret`은 첫 항을 처리한 뒤 입력이 끝날 때까지 연산자와 다음 항을 차례로 읽습니다. 연산자는 오른쪽 피연산자를 출력한 다음 결과에 추가하므로 후위 표기법의 순서가 만들어집니다.

여기서 `interpret`에 전달하는 `out`은 호출자가 준비한 가변 문자열입니다. 두 번째 예제에서는 같은 인터프리터 변수를 새 입력으로 다시 만들고, 기존 결과 문자열을 `clear()`한 뒤 재사용합니다.

## 논의

Interpreter 패턴을 형식 언어의 문법을 설계하고 그 파서를 구현하는 일로만 이해할 수 있지만, 이 패턴의 핵심은 문제 사례를 그 문제에 더 구체적인 방식으로 표현하고, 그 사례를 처리하는 함수·클래스·구조체를 만드는 데 있습니다. 목적에 따라 완전한 파서 대신 작은 규칙이나 도메인 전용 표현만으로도 문제를 더 분명하게 나타낼 수 있습니다.

Rust의 `macro_rules!`를 사용하면 특수한 구문과 그 구문을 소스 코드로 확장하는 규칙을 정의할 수 있습니다. 다음 예제의 `norm!` 매크로는 전달된 각 요소의 제곱을 더한 뒤 제곱근을 구해 n차원 벡터의 [유클리드 길이(Euclidean length)](https://en.wikipedia.org/wiki/Euclidean_distance)를 계산합니다.

```rust
macro_rules! norm {
    ($($element:expr),*) => {
        {
            let mut n = 0.0;
            $(
                n += ($element as f64)*($element as f64);
            )*
            n.sqrt()
        }
    };
}

fn main() {
    let x = -3f64;
    let y = 4f64;

    assert_eq!(3f64, norm!(x));
    assert_eq!(5f64, norm!(x, y));
    assert_eq!(0f64, norm!(0, 0, 0));
    assert_eq!(1f64, norm!(0.5, -0.5, 0.5, -0.5));
}
```

`norm!(x, y)`는 `x`, `y`를 `Vec`에 담아 길이 계산 함수에 전달하는 대신 계산할 요소를 바로 나열하는 표현입니다. 이 예제에서는 매크로가 각 표현식의 제곱을 합산하는 코드를 생성합니다. 이런 호출 방식은 이 문제에 대해 벡터를 만들고 함수를 호출하는 것보다 표현하기 쉽고 더 효율적일 수 있습니다.

다만 DSL이나 매크로가 언제나 더 나은 것은 아닙니다. 별도의 문법을 도입하는 것이 일반 함수나 자료구조를 사용하는 것보다 문제를 실제로 더 명확하게 표현하는지 따져야 합니다.

## 참고 자료

- [Interpreter 패턴](https://en.wikipedia.org/wiki/Interpreter_pattern)
- [문맥 자유 문법](https://en.wikipedia.org/wiki/Context-free_grammar)
- [`macro_rules!`](https://doc.rust-lang.org/rust-by-example/macros.html)
