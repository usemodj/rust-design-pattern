# 구조 패턴

## 정의와 목적

Rust Design Patterns 원문은 [위키백과의 정의](https://en.wikipedia.org/wiki/Structural_pattern)를 인용해 구조 패턴을 “요소들 사이의 관계를 간단히 구현하는 방법을 찾아 설계를 돕는 패턴”으로 설명합니다. 여기서 요소는 객체에 한정되지 않습니다. 타입, 모듈, 크레이트, 또는 이들이 제공하는 API처럼 시스템을 구성하는 여러 부분을 포함할 수 있습니다.

구조 패턴은 구성 요소들이 서로 어떻게 연결되고, 기능을 어떻게 나누며, 구현 세부 사항을 어떻게 감출지를 다룹니다. 목표는 관계를 복잡한 직접 의존성으로 얽히게 하는 대신, 조합·래퍼·모듈 경계와 같은 분명한 구조로 표현해 변경과 확장을 쉽게 하는 것입니다.

생성 패턴이 “무엇을 어떻게 만들 것인가”에 초점을 둔다면, 구조 패턴은 “만들어진 구성 요소를 어떻게 연결하고 조직할 것인가”에 초점을 둡니다. 동작 패턴이 구성 요소 사이의 알고리즘이나 책임 전달을 다룬다면, 구조 패턴은 그 동작이 놓이는 관계와 경계를 설계합니다.

## 위키백과에 소개된 대표 유형

위키백과는 구조 패턴을 구성 요소 사이의 관계를 캡슐화하는 소프트웨어 설계 패턴으로 정의하고, 다음과 같은 유형을 예로 듭니다.

- **[Adapter(어댑터)](/docs/adapter):** 한 인터페이스를 클라이언트가 기대하는 다른 인터페이스에 맞춥니다. 기존 구현을 수정하지 않고 연결할 수 있습니다. 위키백과는 디버깅을 위해 여러 어댑터를 연결하는 Adapter pipeline과, 여러 클래스에 공통 인터페이스를 제공하는 Retrofit Interface 변형도 소개합니다.
- **Aggregate(애그리게이트):** 자식 요소를 모으고 집계하는 동작을 제공하는 Composite의 변형입니다.
- **[Bridge(브리지)](/docs/bridge):** 추상화와 실제 구현을 분리해 양쪽을 독립적으로 변경할 수 있게 합니다. 관련 예인 Tombstone은 중간 조회 객체에 대상 객체의 실제 위치를 보관해 직접적인 위치 의존을 감춥니다.
- **[Composite(컴포지트)](/docs/composite):** 객체를 트리 형태로 구성하고, 단일 객체와 하위 객체들의 묶음을 같은 인터페이스로 다룹니다.
- **[Decorator(데코레이터)](/docs/decorator):** 객체를 감싸 런타임에 기능을 추가합니다. 기능 조합마다 하위 클래스를 계속 만드는 방식의 복잡성을 줄입니다.
- **Extensibility(확장성) 패턴:** 복잡한 구현을 단순한 인터페이스 뒤에 숨겨 확장 가능한 프레임워크나 구조를 제공합니다.
- **[Facade(퍼사드)](/docs/facade):** 기존의 복잡한 인터페이스 위에 자주 쓰는 작업을 위한 간단한 인터페이스를 제공합니다.
- **[Flyweight(플라이웨이트)](/docs/flyweight):** 여러 객체가 공통 데이터를 공유해 메모리 사용량을 줄입니다.
- **Marker interface(마커 인터페이스):** 동작 메서드가 없는 인터페이스로 타입에 메타데이터나 분류를 표시합니다.
- **Pipes and filters(파이프와 필터):** 한 처리 단계의 출력이 다음 단계의 입력이 되는 처리 흐름을 구성합니다.
- **Opaque pointer(불투명 포인터):** 비공개 또는 외부에 정의가 드러나지 않는 타입을 가리켜 구현 세부 사항을 감춥니다.
- **[Proxy(프록시)](/docs/proxy):** 다른 객체나 자원에 대한 대리 인터페이스를 제공해 접근을 제어하거나 중개합니다.

이 목록은 구조 패턴을 이해하는 데 도움이 되는 분류이지, 모든 언어나 프로젝트가 반드시 같은 이름과 구현 형태를 사용해야 한다는 규칙은 아닙니다. Rust에서는 클래스 상속을 전제로 한 전통적인 객체 지향 구조보다 구조체 합성, 열거형, 트레이트, 모듈 경계, 래퍼 타입, 소유권을 중심으로 비슷한 목적을 달성할 수 있습니다.

## 이 안내서의 구조 설계 기법

이 장에서는 [Adapter 패턴](/docs/adapter), [Bridge 패턴](/docs/bridge), [Composite 패턴](/docs/composite), [Decorator 패턴](/docs/decorator), [Facade 패턴](/docs/facade), [Flyweight 패턴](/docs/flyweight), [Proxy 패턴](/docs/proxy)과 함께 Rust의 타입 시스템과 모듈 시스템에 맞춘 구조 설계 관용구를 다룹니다.

- [Adapter 패턴](/docs/adapter): 기존 API를 감싸 클라이언트가 기대하는 트레이트에 맞춰 연결합니다.
- [Bridge 패턴](/docs/bridge): 추상화와 구현을 별도 트레이트로 나누어 각 변화 축을 독립적으로 확장합니다.
- [Composite 패턴](/docs/composite): 리프와 복합 객체를 공통 트레이트로 다루어 트리 구조를 재귀적으로 처리합니다.
- [Decorator 패턴](/docs/decorator): 기존 인터페이스를 유지하는 래퍼를 쌓아 책임을 조합합니다.
- [Facade 패턴](/docs/facade): 여러 하위 시스템의 작업을 단순한 고수준 API로 조정합니다.
- [Flyweight 패턴](/docs/flyweight): 내재 상태를 공유하고 개별 객체의 외재 상태를 분리해 중복 메모리를 줄입니다.
- [Proxy 패턴](/docs/proxy): 실제 객체와 같은 인터페이스로 객체 생성이나 자원 접근을 중개합니다.
- [독립적으로 빌리기 위한 구조체 분해](/docs/compose-structs): 큰 구조체의 책임과 데이터를 작은 구성 요소로 나누어 필요한 부분을 독립적으로 빌리고 변경하기 쉽게 합니다.
- [작은 크레이트를 선호하기](/docs/small-crates): 기능 경계를 응집도 높은 크레이트로 나누어 의존성, 재사용성, 컴파일 범위를 관리합니다.
- [`unsafe`를 작은 모듈에 가두기](/docs/unsafe-modules): 안전하지 않은 연산을 좁고 검토 가능한 경계에 격리하고, 바깥에는 안전한 인터페이스를 제공합니다.
- [복잡한 타입 바운드를 줄이는 사용자 정의 트레이트](/docs/trait-bounds): 반복적으로 나타나는 제네릭 제약을 의미 있는 트레이트로 묶어 타입 시그니처를 더 읽기 쉽게 합니다.

각 기법은 구조나 경계를 추가하므로 그 자체가 목적이 되어서는 안 됩니다. 타입과 모듈을 나눈 결과 기능 응집도가 높아지고, 의존 관계가 단순해지거나, 안전성과 유지보수성이 실제로 개선되는지 확인해야 합니다. 반대로 작은 구현을 불필요하게 여러 크레이트로 나누거나 래퍼를 겹겹이 두면 구조가 오히려 이해하기 어려워질 수 있습니다.

## 생성·동작 패턴과의 관계

세 패턴 분류는 서로 배타적이지 않습니다. 예를 들어 Decorator는 객체의 구조를 감싸면서 요청 처리 동작을 추가하고, Factory는 특정한 구조를 가진 구성 요소를 생성할 수 있습니다. 패턴을 분류하는 목적은 이름을 붙이는 것이 아니라, 해결하려는 문제의 중심이 생성 책임인지, 동작의 분리인지, 구성 요소 사이의 관계인지 파악하는 데 있습니다.

## 참고 자료

- [구조 패턴 — 위키백과](https://en.wikipedia.org/wiki/Structural_pattern)
- [Rust Design Patterns: Structural Patterns 원문](https://rust-unofficial.github.io/patterns/patterns/structural/)
- [Adapter 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/adapter)
- [Adapter 패턴 — 위키백과](https://en.wikipedia.org/wiki/Adapter_pattern)
- [Bridge 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/bridge)
- [Bridge 패턴 — 위키백과](https://en.wikipedia.org/wiki/Bridge_pattern)
- [Composite 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/composite)
- [Composite 패턴 — 위키백과](https://en.wikipedia.org/wiki/Composite_pattern)
- [Decorator 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/decorator)
- [Decorator 패턴 — 위키백과](https://en.wikipedia.org/wiki/Decorator_pattern)
- [Facade 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/facade)
- [Facade 패턴 — 위키백과](https://en.wikipedia.org/wiki/Facade_pattern)
- [Flyweight 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/flyweight)
- [Flyweight 패턴 — 위키백과](https://en.wikipedia.org/wiki/Flyweight_pattern)
- [Proxy 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/proxy)
- [Proxy 패턴 — 위키백과](https://en.wikipedia.org/wiki/Proxy_pattern)
- [독립적으로 빌리기 위한 구조체 분해](https://rust-unofficial.github.io/patterns/patterns/structural/compose-structs.html)
- [작은 크레이트를 선호하기](https://rust-unofficial.github.io/patterns/patterns/structural/small-crates.html)
- [`unsafe`를 작은 모듈에 가두기](https://rust-unofficial.github.io/patterns/patterns/structural/unsafe-mods.html)
- [복잡한 타입 바운드를 줄이는 사용자 정의 트레이트](https://rust-unofficial.github.io/patterns/patterns/structural/trait-for-bounds.html)
- [생성 패턴](/docs/creational-patterns)
- [동작 패턴](/docs/behavioral-patterns)
