# 행동 패턴

## 정의와 목적

행동 패턴(behavioral pattern)은 프로그램의 구성 요소가 **어떻게 협력하고, 책임과 알고리즘을 어떻게 나눌지** 다룹니다. 요청을 누가 처리할지, 실행할 작업을 어떻게 표현할지, 동작을 언제 교체할지와 같은 문제를 공통된 설계 방식으로 정리합니다.

생성 패턴이 구성 요소를 **어떻게 만드는지**, 구조 패턴이 구성 요소를 **어떻게 연결하는지**에 중점을 둔다면, 행동 패턴은 연결된 구성 요소가 **무슨 순서로 어떻게 협력하는지**에 중점을 둡니다. 하나의 설계가 여러 분류의 패턴을 함께 사용할 수도 있습니다.

## 적용하기 좋은 상황

- 요청을 보내는 쪽이 실제 처리자를 미리 알지 못하거나 처리 순서를 바꿔야 할 때
- 작업을 값으로 보관해 나중에 실행하거나, 기록·취소 같은 기능을 추가할 때
- 같은 문제에 적용할 알고리즘을 상황에 따라 바꿔야 할 때
- 문법이나 자료 구조에 대해 평가, 출력, 검사 등 여러 작업을 수행할 때
- 자원 사용 범위와 정리 시점을 명확히 표현해야 할 때

패턴을 도입하면 처리 흐름을 분리할 수 있지만 타입과 간접 호출이 늘어날 수도 있습니다. 간단한 분기나 함수 하나로 충분한 문제라면 먼저 그 방법을 고려하세요.

## 이 안내서에서 다루는 항목

| 항목 | 해결하려는 문제 | 주로 사용하는 상황 |
| --- | --- | --- |
| [Chain of Responsibility](/docs/chain-of-responsibility) | 요청을 처리자들에게 순서대로 전달합니다. | 조건에 맞는 첫 처리자가 요청을 처리하거나 다음 처리자에게 넘길 때 |
| [Command](/docs/command) | 실행할 동작과 필요한 정보를 값으로 나타냅니다. | 작업을 큐에 넣거나 나중에 실행하고, 필요한 경우 실행 취소를 지원할 때 |
| [Interpreter](/docs/interpreter) | 언어의 문법을 표현하고 각 표현식의 의미를 해석합니다. | 간단한 표현식이나 도메인 전용 언어를 평가할 때 |
| [Iterator](/docs/iterator) | 컬렉션의 내부 구조를 드러내지 않고 요소를 순회합니다. | 순회 방법과 저장 구조를 분리하고 필터·변환을 조합할 때 |
| [Mediator](/docs/mediator) | 구성 요소 사이의 협력과 통신을 중재자에게 맡깁니다. | 여러 구성 요소가 서로 직접 참조하지 않고 협력할 때 |
| [Memento](/docs/memento) | 객체의 내부 상태를 캡슐화해 저장하고 복원합니다. | 편집 기록이나 체크포인트를 보관해 이전 상태로 되돌릴 때 |
| [Newtype](/docs/newtype) | 기존 타입을 새 타입으로 감싸 의미와 허용 동작을 구분합니다. | 같은 내부 타입을 서로 다른 개념으로 구분하거나 공개 API를 제한할 때 |
| [Observer](/docs/observer) | 한 대상의 상태 변화를 등록된 관찰자에게 알립니다. | 상태 변경에 따라 여러 구성 요소를 갱신할 때 |
| [RAII 가드](/docs/raii) | 자원을 소유한 값의 생명주기로 자원 정리를 관리합니다. | 잠금 해제처럼 범위를 벗어날 때 반드시 필요한 정리가 있을 때 |
| [State](/docs/state) | 현재 상태에 따라 동작과 허용되는 상태 전이를 달리합니다. | 문서 승인이나 주문 처리처럼 단계별 규칙이 다를 때 |
| [Strategy](/docs/strategy) | 같은 목적을 수행하는 알고리즘을 교체할 수 있도록 분리합니다. | 정렬·계산·검증 등의 정책을 상황에 따라 선택할 때 |
| [Template Method](/docs/template-method) | 공통 알고리즘의 순서를 유지하며 일부 단계를 교체합니다. | 처리 흐름은 같지만 입력 해석이나 출력 형식이 다를 때 |
| [Visitor](/docs/visitor) | 자료 구조를 방문하는 작업을 구조 자체와 분리합니다. | 트리나 구문 구조에 여러 종류의 분석·출력 작업을 추가할 때 |

Newtype과 RAII 가드는 전통적인 객체 지향 행동 패턴 목록이라기보다 Rust의 타입 및 자원 관리에 맞춘 설계 기법입니다. 이 안내서에서는 동작과 책임을 안전하게 표현하는 데 도움이 되어 함께 다룹니다.

Rust에서는 전통적인 클래스 상속을 그대로 옮기기보다 트레이트, 열거형, 함수, 합성 등으로 같은 목적을 표현할 수 있습니다. 예를 들어 표준 `Iterator` 트레이트는 요소 순회에 이미 사용할 수 있는 인터페이스를 제공합니다.

## 비슷한 패턴 구분하기

- **Chain of Responsibility와 Strategy:** 책임 연쇄는 여러 처리자를 순서대로 살펴 요청을 처리할 대상을 찾습니다. Strategy는 같은 목적을 수행하는 알고리즘 중 하나를 선택해 사용합니다.
- **Command와 Strategy:** Command는 실행할 작업 자체를 값으로 표현해 저장하거나 전달하는 데 초점을 둡니다. Strategy는 작업에 사용할 알고리즘을 교체하는 데 초점을 둡니다.
- **Interpreter와 Visitor:** Interpreter는 표현식의 의미를 해석하는 데 초점을 둡니다. Visitor는 자료 구조를 순회하며 수행할 작업을 분리합니다. 예를 들어 Visitor로 구문 트리를 순회하면서 표현식을 평가할 수 있습니다.
- **RAII 가드와 Command:** RAII 가드는 값이 정리될 때 자원 해제 같은 동작을 연결합니다. Command는 명시적으로 실행할 작업을 값으로 다룹니다.

## Rust에서 행동 표현하기

- **열거형과 `match`:** 가능한 종류가 정해져 있을 때 각 경우를 명시적으로 처리합니다.
- **함수와 클로저:** 간단한 동작을 인자로 전달하거나, 필요한 상태를 캡처해 보관합니다.
- **트레이트와 제네릭:** 공통 동작을 정의하고 구체 타입에 따라 정적 디스패치를 사용합니다.
- **트레이트 객체:** `Box<dyn Trait>`처럼 서로 다른 구현을 같은 컬렉션에 넣거나 실행 중 선택합니다.
- **소유권과 `Drop`:** 자원의 소유 범위와 정리 시점을 연결합니다. 다만 `Drop`이 실행되지 않을 수 있는 경우도 있으므로 프로그램의 메모리 안전성을 `Drop` 실행에만 의존해서는 안 됩니다.

선택 기준은 추상화의 이름보다 요구사항입니다. 구현을 추가하거나 교체할 가능성, 오류와 미처리 요청을 표현하는 방법, 실행 순서, 소유권과 수명, 테스트 용이성을 함께 살펴보세요.

## 참고 자료

- [행동 패턴 — Rust Design Patterns](https://rust-unofficial.github.io/patterns/patterns/behavioural/)
- [행동 패턴 — 위키백과](https://en.wikipedia.org/wiki/Behavioral_pattern)
- [Chain of Responsibility — Refactoring Guru](https://refactoring.guru/design-patterns/chain-of-responsibility)
- [Strategy — Refactoring Guru](https://refactoring.guru/design-patterns/strategy)
- [Visitor — Refactoring Guru](https://refactoring.guru/design-patterns/visitor)
- [Rust Book: 트레이트로 공통 동작 정의하기](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust Book: 트레이트 객체](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [Rust Book: `Drop` 트레이트](https://doc.rust-lang.org/book/ch15-03-drop.html)
- [Rust 표준 라이브러리: `Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [생성 패턴](/docs/creational-patterns) · [구조 패턴](/docs/structural-patterns)
