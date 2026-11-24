# 생성 패턴

## 정의와 목적

[위키백과의 정의](https://en.wikipedia.org/wiki/Creational_pattern)에 따르면 생성 패턴(creational pattern)은 특정 상황에 알맞은 방식으로 객체를 만드는 소프트웨어 설계 패턴입니다. 프로그래밍 언어가 기본 제공하는 생성 방식만 사용하면 설계상의 제약이 생기거나 설계가 불필요하게 복잡해질 수 있습니다. 생성 패턴은 객체 생성을 제어하는 별도의 방법을 두어 이런 문제를 해결합니다.

생성 패턴의 핵심은 시스템이 객체를 **어떻게 생성하고, 조합하고, 표현하는지**와 생성된 객체를 사용하는 부분을 분리하는 것입니다. 그 결과 시스템은 무엇을 만들지, 누가 만들지, 어떻게 만들지, 언제 만들지를 유연하게 결정할 수 있습니다.

생성 패턴은 주로 두 가지를 캡슐화합니다.

- 시스템이 어떤 구체 타입을 사용하는지에 관한 지식
- 그 구체 타입의 인스턴스를 어떻게 생성하고 서로 조합하는지에 관한 세부 사항

따라서 호출하는 쪽은 구체 구현을 직접 생성하는 코드에 덜 의존하고, 구현을 바꾸거나 새로운 생성 방식을 추가하기 쉬워집니다.

## 생성 패턴의 분류

전통적으로 생성 패턴은 다음 두 부류로 나눕니다.

- **객체 생성 패턴(object-creational patterns):** 객체 생성을 다른 객체에 위임합니다. 예를 들어 팩토리 객체가 어떤 구체 객체를 만들지 결정할 수 있습니다.
- **클래스 생성 패턴(class-creational patterns):** 객체 생성을 하위 클래스에 위임합니다. 이 분류는 클래스 상속을 사용하는 객체 지향 언어의 설계 방식에 기반합니다.

Rust에는 전통적인 클래스 상속이 없으므로, 상속을 전제로 한 분류를 Rust에 그대로 대응시키기는 어렵습니다. Rust에서는 트레이트, 제네릭, 열거형, 함수와 클로저 등을 조합해 생성 책임과 구현 선택을 분리할 수 있습니다.

## 적용하기 좋은 상황

위키백과는 다음과 같은 경우 생성 패턴을 고려할 수 있다고 설명합니다.

- 시스템이 구체 객체나 제품이 만들어지는 방식과 독립적이어야 할 때
- 함께 사용하도록 설계된 관련 객체들의 집합을 만들어야 할 때
- 라이브러리나 제품의 내부 구현을 숨기고 인터페이스만 공개하고 싶을 때
- 복잡한 객체를 서로 다른 표현 형태로 구성해야 할 때
- 어떤 클래스가 생성할 객체를 하위 클래스에서 결정해야 할 때
- 실행 중에 생성할 구체 타입이 정해질 때
- 인스턴스를 하나로 제한하고 클라이언트가 항상 그 인스턴스에 접근해야 할 때
- 기존 코드를 수정하지 않고 생성 가능한 인스턴스나 구현을 확장해야 할 때

이 목록은 생성 패턴을 무조건 도입하라는 뜻이 아닙니다. 기본 생성 코드가 충분히 단순하다면 그 코드를 직접 사용하는 편이 더 분명합니다. 생성 과정이 복잡하거나 호출 코드가 구체 구현에 강하게 결합되는 등 실제 문제가 있을 때 패턴을 적용하는 것이 좋습니다.

## 대표적인 생성 패턴

생성 패턴에는 객체를 선택하거나 구성하는 여러 접근 방식이 있습니다.

- **[Abstract Factory(추상 팩토리)](/docs/abstract-factory):** 서로 관련된 객체들의 제품군을 구체 타입에 직접 의존하지 않고 생성합니다.
- **[Factory Method(팩토리 메서드)](/docs/factory-method):** 생성 메서드의 구현을 구체 Creator에 맡겨, 제품을 사용하는 핵심 로직과 구체 제품의 생성을 분리합니다.
- **Builder(빌더):** 복잡한 객체를 구성하는 과정과 최종 표현을 분리하여 같은 구성 절차로 서로 다른 결과를 만들 수 있게 합니다.
- **Dependency Injection(의존성 주입):** 클래스나 컴포넌트가 필요한 객체를 직접 생성하는 대신 외부에서 전달받습니다.
- **Lazy Initialization(지연 초기화):** 객체나 값, 비용이 큰 연산의 실행을 실제로 처음 필요해질 때까지 미룹니다.
- **Object Pool(객체 풀):** 더 이상 사용하지 않는 객체를 재사용해 객체나 자원을 매번 획득하고 해제하는 비용을 줄입니다.
- **[Prototype(프로토타입)](/docs/prototype):** 원형 객체를 복제하여 새 객체를 만들며, 생성할 객체의 구체 타입이 원형 인스턴스에 의해 정해질 때 사용합니다.
- **Singleton(싱글턴):** 생성되는 인스턴스를 하나로 제한합니다. Rust에서는 전역 공유 상태가 동시성·테스트 가능성을 복잡하게 만들 수 있으므로, 싱글턴이 실제 요구사항인지 먼저 검토해야 합니다.

## Rust에서 생성 책임 표현하기

Rust에서는 복잡한 패턴을 도입하기 전에 기본 관용구로 문제가 해결되는지 확인하세요.

- `new` 연관 함수는 필요한 값을 인자로 받아 유효한 상태의 인스턴스를 만듭니다.
- `Default`는 합리적인 기본값을 제공할 수 있을 때 적합합니다.
- 구조체나 열거형 생성자는 데이터 형태를 타입으로 드러냅니다.
- 선택적 설정이 많거나 구성 단계가 여러 개라면 Builder가 호출부를 명확하게 만들 수 있습니다.
- 실행 중에 구현을 선택해야 한다면 트레이트 객체나 팩토리 역할을 하는 함수를 사용할 수 있습니다.
- 필드나 생성자를 비공개로 두고 검증을 거친 생성 함수만 공개하면, 잘못된 상태의 값을 외부에서 만들지 못하게 할 수 있습니다.

패턴의 목적은 추상화 자체를 늘리는 것이 아니라, 생성 방법과 사용 코드를 적절히 분리해 실제 변경 가능성이나 복잡성을 관리하는 것입니다.

## 이 안내서에서 다루는 항목

이 장에서는 [Abstract Factory 패턴](/docs/abstract-factory), [Factory Method 패턴](/docs/factory-method), [Builder 패턴](/docs/builder), [Fold 패턴](/docs/fold), [Prototype 패턴](/docs/prototype), [Singleton 패턴](/docs/singleton)을 다룹니다. Abstract Factory는 관련 제품군을 함께 생성해 서로 호환되는 변형을 유지합니다. Factory Method는 생성 메서드를 구체 Creator에 맡겨 제품 생성과 공통 동작을 분리합니다. Builder는 복잡한 값의 생성을 단계적으로 구성하는 데 사용됩니다. Fold는 전통적인 객체 생성 패턴과는 다르며, 기존 자료 구조를 순회하면서 변환된 새 자료 구조를 만드는 함수형 패턴에 가깝습니다. Prototype은 등록된 원형을 복제해 런타임에 선택된 타입의 객체를 생성합니다. Singleton은 인스턴스를 하나로 제한하고 공통 접근 경로를 제공하는 패턴입니다. Rust에서는 `OnceLock` 같은 표준 타입으로 초기화를 안전하게 처리할 수 있지만, 전역 상태가 테스트와 의존성 설계를 복잡하게 만들 수 있으므로 필요한 경우에만 사용해야 합니다.

## 참고 자료

- [생성 패턴 — 위키백과](https://en.wikipedia.org/wiki/Creational_pattern)
- [생성 패턴 — Rust Design Patterns](https://rust-unofficial.github.io/patterns/patterns/creational/)
- [Abstract Factory 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/abstract-factory)
- [Abstract Factory 패턴 — 위키백과](https://en.wikipedia.org/wiki/Abstract_factory_pattern)
- [Factory Method 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/factory-method)
- [Factory Method 패턴 — 위키백과](https://en.wikipedia.org/wiki/Factory_method_pattern)
- [Prototype 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/prototype)
- [Prototype 패턴 — 위키백과](https://en.wikipedia.org/wiki/Prototype_pattern)
- [Rust 표준 라이브러리: `Clone` 트레이트](https://doc.rust-lang.org/std/clone/trait.Clone.html)
- [Builder 패턴](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
- [Singleton 패턴 — 위키백과](https://en.wikipedia.org/wiki/Singleton_pattern)
- [`std::sync::OnceLock` 문서](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)
