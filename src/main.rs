use axum::{
    Router,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use pulldown_cmark::{Options, Parser, html as markdown_html};
use std::collections::HashMap;

fn server_address(host: String, port: &str) -> Result<(String, u16), Box<dyn std::error::Error>> {
    if host.trim().is_empty() {
        return Err("HOST는 비어 있을 수 없습니다".into());
    }
    let port = port
        .parse::<u16>()
        .map_err(|error| format!("PORT가 올바르지 않습니다: {error}"))?;
    Ok((host, port))
}

fn environment_or_default(name: &str, default: &str) -> Result<String, std::env::VarError> {
    match std::env::var(name) {
        Ok(value) => Ok(value),
        Err(std::env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Copy)]
struct Page {
    slug: &'static str,
    title: &'static str,
    section: &'static str,
    file: &'static str,
    summary: &'static str,
}

#[derive(Clone, Copy)]
struct TocItem {
    title: &'static str,
    slug: &'static str,
    children: &'static [TocItem],
}

const fn toc_page(title: &'static str, slug: &'static str) -> TocItem {
    TocItem {
        title,
        slug,
        children: &[],
    }
}

const fn toc_section(
    title: &'static str,
    slug: &'static str,
    children: &'static [TocItem],
) -> TocItem {
    TocItem {
        title,
        slug,
        children,
    }
}

const TOC: &[TocItem] = &[
    toc_section(
        "소개",
        "introduction",
        &[toc_page("번역 안내", "translations")],
    ),
    toc_section(
        "관용구",
        "idioms",
        &[
            toc_page("인자에는 빌린 타입 사용하기", "borrowed-types"),
            toc_page("format!으로 문자열 연결하기", "format-concat"),
            toc_page("생성자", "constructor"),
            toc_page("Default 트레이트", "default-trait"),
            toc_page("컬렉션은 스마트 포인터", "collections-deref"),
            toc_page("소멸자에서의 마무리 작업", "destructor-finalization"),
            toc_page("mem::take와 mem::replace", "mem-take-replace"),
            toc_page("스택에서의 동적 디스패치", "stack-dynamic-dispatch"),
            toc_section(
                "FFI 관용구",
                "ffi-idioms",
                &[
                    toc_page("FFI에서 오류 처리하기", "ffi-errors"),
                    toc_page("FFI에서 문자열 받기", "ffi-accept-strings"),
                    toc_page("FFI로 문자열 전달하기", "ffi-pass-strings"),
                ],
            ),
            toc_page("Option 반복하기", "option-iterator"),
            toc_page("클로저에 변수 전달하기", "closure-capture"),
            toc_page("확장 가능한 타입 만들기", "extensible-types"),
            toc_page("문서 예제 초기화 간소화", "rustdoc-init"),
            toc_page("임시 가변성", "temporary-mutability"),
            toc_page("오류 시 소비된 인자 돌려주기", "consumed-argument-error"),
        ],
    ),
    toc_section(
        "디자인 패턴 개요",
        "design-patterns",
        &[
            toc_section(
                "행동 패턴",
                "behavioral-patterns",
                &[
                    toc_page("Chain of Responsibility 패턴", "chain-of-responsibility"),
                    toc_page("Command 패턴", "command"),
                    toc_page("Interpreter 패턴", "interpreter"),
                    toc_page("Iterator 패턴", "iterator"),
                    toc_page("Mediator 패턴", "mediator"),
                    toc_page("Memento 패턴", "memento"),
                    toc_page("Newtype 패턴", "newtype"),
                    toc_page("Observer 패턴", "observer"),
                    toc_page("RAII 가드", "raii"),
                    toc_page("State 패턴", "state"),
                    toc_page("Strategy 패턴", "strategy"),
                    toc_page("Template Method 패턴", "template-method"),
                    toc_page("Visitor 패턴", "visitor"),
                ],
            ),
            toc_section(
                "생성 패턴",
                "creational-patterns",
                &[
                    toc_page("Abstract Factory 패턴", "abstract-factory"),
                    toc_page("Factory Method 패턴", "factory-method"),
                    toc_page("Builder 패턴", "builder"),
                    toc_page("Fold 패턴", "fold"),
                    toc_page("Prototype 패턴", "prototype"),
                    toc_page("Singleton 패턴", "singleton"),
                ],
            ),
            toc_section(
                "구조 패턴",
                "structural-patterns",
                &[
                    toc_page("Adapter 패턴", "adapter"),
                    toc_page("Bridge 패턴", "bridge"),
                    toc_page("Composite 패턴", "composite"),
                    toc_page("Decorator 패턴", "decorator"),
                    toc_page("Facade 패턴", "facade"),
                    toc_page("Flyweight 패턴", "flyweight"),
                    toc_page("Proxy 패턴", "proxy"),
                    toc_page("독립적으로 빌리기 위한 구조체 분해", "compose-structs"),
                    toc_page("작은 크레이트를 선호하기", "small-crates"),
                    toc_page("unsafe를 작은 모듈에 가두기", "unsafe-modules"),
                    toc_page(
                        "사용자 정의 트레이트로 복잡한 바운드 줄이기",
                        "trait-bounds",
                    ),
                ],
            ),
            toc_section(
                "FFI 디자인 패턴",
                "ffi-patterns",
                &[
                    toc_page("FFI를 위한 객체 기반 API", "ffi-object-apis"),
                    toc_page("래퍼로 타입 통합하기", "ffi-wrapper-types"),
                ],
            ),
        ],
    ),
    toc_section(
        "안티 패턴",
        "anti-patterns",
        &[
            toc_page(
                "빌림 검사기를 만족시키기 위한 무분별한 Clone",
                "clone-borrow-checker",
            ),
            toc_page("#![deny(warnings)]", "deny-warnings"),
            toc_page("Deref 다형성", "deref-polymorphism"),
        ],
    ),
    toc_section(
        "Rust의 함수형 기능",
        "functional-overview",
        &[
            toc_page("프로그래밍 패러다임", "functional-programming"),
            toc_page("타입 클래스로서의 제네릭", "generics-as-type-classes"),
            toc_page("함수형 옵틱스", "functional-optics"),
        ],
    ),
    toc_section(
        "추가 자료",
        "design-resources",
        &[
            toc_page("디자인 원칙", "design-principles"),
            toc_page("리팩터링", "refactoring"),
        ],
    ),
];

const PAGES: &[Page] = &[
    Page {
        slug: "introduction",
        title: "소개",
        section: "시작하기",
        file: "introduction.md",
        summary: "Rust에서 패턴을 이해하고 선택하는 방법을 살펴봅니다.",
    },
    Page {
        slug: "idioms",
        title: "관용구",
        section: "관용구",
        file: "idioms.md",
        summary: "Rust 커뮤니티가 널리 사용하는 표현과 코딩 습관입니다.",
    },
    Page {
        slug: "borrowed-types",
        title: "인자에는 빌린 타입 사용하기",
        section: "관용구",
        file: "borrowed-types.md",
        summary: "소유권 이전이 필요하지 않은 함수 인자에는 빌린 타입을 사용합니다.",
    },
    Page {
        slug: "constructor",
        title: "생성자",
        section: "관용구",
        file: "constructor.md",
        summary: "관례적인 new 함수와 타입별 초기화 방식을 알아봅니다.",
    },
    Page {
        slug: "default-trait",
        title: "Default 트레이트",
        section: "관용구",
        file: "default-trait.md",
        summary: "기본값을 제공해 타입의 초기화를 간결하게 만듭니다.",
    },
    Page {
        slug: "iterator",
        title: "Iterator 패턴",
        section: "디자인 패턴",
        file: "iterator.md",
        summary: "컬렉션의 내부 구조와 순회 방법을 분리해 요소를 차례로 탐색합니다.",
    },
    Page {
        slug: "newtype",
        title: "Newtype 패턴",
        section: "디자인 패턴",
        file: "newtype.md",
        summary: "튜플 구조체로 타입 안전성과 의미를 더합니다.",
    },
    Page {
        slug: "builder",
        title: "Builder 패턴",
        section: "디자인 패턴",
        file: "builder.md",
        summary: "단계적으로 구성하는 복잡한 값을 명확하게 만듭니다.",
    },
    Page {
        slug: "strategy",
        title: "Strategy 패턴",
        section: "디자인 패턴",
        file: "strategy.md",
        summary: "행동을 바꿔 끼울 수 있도록 알고리즘을 분리합니다.",
    },
    Page {
        slug: "raii",
        title: "RAII 가드",
        section: "디자인 패턴",
        file: "raii.md",
        summary: "값의 수명과 자원 해제를 연결해 정리를 안전하게 처리합니다.",
    },
    Page {
        slug: "anti-patterns",
        title: "안티 패턴",
        section: "안티 패턴",
        file: "anti-patterns.md",
        summary: "흔하지만 유지보수성과 안정성을 해칠 수 있는 선택을 살펴봅니다.",
    },
    Page {
        slug: "clone-borrow-checker",
        title: "빌림 검사기를 만족시키기 위한 무분별한 Clone",
        section: "안티 패턴",
        file: "clone-borrow-checker.md",
        summary: "복제가 진짜 문제 해결인지, 소유권 구조의 신호인지 구분합니다.",
    },
    Page {
        slug: "functional-programming",
        title: "프로그래밍 패러다임",
        section: "함수형 프로그래밍",
        file: "functional-programming.md",
        summary: "명령형과 선언형 스타일의 차이, Rust에서의 활용을 살펴봅니다.",
    },
    Page {
        slug: "translations",
        title: "번역 안내",
        section: "시작하기",
        file: "translations.md",
        summary: "원문에 기여하거나 다른 언어로 번역할 때 참고할 사항입니다.",
    },
    Page {
        slug: "design-resources",
        title: "추가 자료",
        section: "추가 자료",
        file: "design-resources.md",
        summary: "Rust 디자인 패턴을 더 공부할 수 있는 강연과 온라인 책입니다.",
    },
    Page {
        slug: "design-principles",
        title: "디자인 원칙",
        section: "추가 자료",
        file: "design-principles.md",
        summary: "SOLID, DRY, KISS 등 자주 인용되는 설계 원칙을 정리합니다.",
    },
    Page {
        slug: "deny-warnings",
        title: "#![deny(warnings)]",
        section: "안티 패턴",
        file: "deny-warnings.md",
        summary: "라이브러리 코드에서 모든 경고를 오류로 만드는 위험을 살펴봅니다.",
    },
    Page {
        slug: "deref-polymorphism",
        title: "Deref 다형성",
        section: "안티 패턴",
        file: "deref-polymorphism.md",
        summary: "Deref를 상속이나 일반적인 다형성 대용으로 사용하는 문제입니다.",
    },
    Page {
        slug: "functional-overview",
        title: "Rust의 함수형 기능",
        section: "함수형 프로그래밍",
        file: "functional-overview.md",
        summary: "Rust의 함수형 구성 요소와 이 안내서의 범위를 소개합니다.",
    },
    Page {
        slug: "generics-as-type-classes",
        title: "타입 클래스로서의 제네릭",
        section: "함수형 프로그래밍",
        file: "generics-as-type-classes.md",
        summary: "제네릭 타입과 트레이트 제약으로 상태별 API를 표현합니다.",
    },
    Page {
        slug: "functional-optics",
        title: "함수형 옵틱스",
        section: "함수형 프로그래밍",
        file: "functional-optics.md",
        summary: "중첩된 데이터의 일부를 조합해 읽고 갱신하는 추상화를 소개합니다.",
    },
    Page {
        slug: "format-concat",
        title: "format!으로 문자열 연결하기",
        section: "관용구",
        file: "format-concat.md",
        summary: "문자열 조각을 연결할 때 format! 매크로를 활용합니다.",
    },
    Page {
        slug: "collections-deref",
        title: "컬렉션은 스마트 포인터",
        section: "관용구",
        file: "collections-deref.md",
        summary: "컬렉션을 역참조해 요소를 가리키는 관용적 인터페이스를 설명합니다.",
    },
    Page {
        slug: "destructor-finalization",
        title: "소멸자에서의 마무리 작업",
        section: "관용구",
        file: "destructor-finalization.md",
        summary: "Drop을 이용한 정리와 명시적인 종료 처리의 차이를 살펴봅니다.",
    },
    Page {
        slug: "ffi-idioms",
        title: "FFI 관용구",
        section: "관용구",
        file: "ffi-idioms.md",
        summary: "Rust와 다른 언어 사이의 외부 함수 인터페이스 원칙입니다.",
    },
    Page {
        slug: "ffi-errors",
        title: "FFI에서 오류 처리하기",
        section: "관용구",
        file: "ffi-errors.md",
        summary: "언어 경계를 넘어 오류를 안전하게 전달하는 방식을 알아봅니다.",
    },
    Page {
        slug: "ffi-accept-strings",
        title: "FFI에서 문자열 받기",
        section: "관용구",
        file: "ffi-accept-strings.md",
        summary: "외부 문자열의 포인터, 길이, 인코딩과 수명 계약을 다룹니다.",
    },
    Page {
        slug: "ffi-pass-strings",
        title: "FFI로 문자열 전달하기",
        section: "관용구",
        file: "ffi-pass-strings.md",
        summary: "Rust 문자열을 C 호환 표현으로 전달할 때의 소유권을 살펴봅니다.",
    },
    Page {
        slug: "mem-take-replace",
        title: "mem::take와 mem::replace",
        section: "관용구",
        file: "mem-take-replace.md",
        summary: "열거형 값을 교체하면서 내부의 소유 값을 이동합니다.",
    },
    Page {
        slug: "stack-dynamic-dispatch",
        title: "스택에서의 동적 디스패치",
        section: "관용구",
        file: "stack-dynamic-dispatch.md",
        summary: "힙 할당을 피하면서 트레이트 객체를 다루는 선택지를 봅니다.",
    },
    Page {
        slug: "option-iterator",
        title: "Option 반복하기",
        section: "관용구",
        file: "option-iterator.md",
        summary: "Option을 0개 또는 1개 항목의 반복자로 다루는 방법입니다.",
    },
    Page {
        slug: "closure-capture",
        title: "클로저에 변수 전달하기",
        section: "관용구",
        file: "closure-capture.md",
        summary: "클로저의 캡처와 move를 이용해 값의 소유권을 전달합니다.",
    },
    Page {
        slug: "extensible-types",
        title: "확장 가능한 타입 만들기",
        section: "관용구",
        file: "extensible-types.md",
        summary: "비공개 필드와 non_exhaustive로 라이브러리 API를 확장합니다.",
    },
    Page {
        slug: "consumed-argument-error",
        title: "오류 시 소비된 인자 돌려주기",
        section: "관용구",
        file: "consumed-argument-error.md",
        summary: "실패한 연산에서 호출자가 소유권을 되찾을 수 있게 합니다.",
    },
    Page {
        slug: "rustdoc-init",
        title: "문서 예제 초기화 간소화",
        section: "관용구",
        file: "rustdoc-init.md",
        summary: "실제 프로그램 진입점과 문서 테스트 코드를 분리합니다.",
    },
    Page {
        slug: "temporary-mutability",
        title: "임시 가변성",
        section: "관용구",
        file: "temporary-mutability.md",
        summary: "초기화 중에만 가변으로 두고 이후에는 불변으로 사용합니다.",
    },
    Page {
        slug: "design-patterns",
        title: "디자인 패턴 개요",
        section: "디자인 패턴",
        file: "design-patterns.md",
        summary: "Rust에서 패턴을 선택할 때의 기준과 YAGNI 원칙입니다.",
    },
    Page {
        slug: "behavioral-patterns",
        title: "행동 패턴",
        section: "디자인 패턴",
        file: "behavioral-patterns.md",
        summary: "객체와 구성 요소 사이의 상호작용을 조직하는 패턴입니다.",
    },
    Page {
        slug: "chain-of-responsibility",
        title: "Chain of Responsibility 패턴",
        section: "디자인 패턴",
        file: "chain-of-responsibility.md",
        summary: "요청을 처리할 수 있는 객체를 순서대로 찾아 책임을 위임합니다.",
    },
    Page {
        slug: "command",
        title: "Command 패턴",
        section: "디자인 패턴",
        file: "command.md",
        summary: "실행할 동작을 값으로 캡슐화하고 지연하거나 되돌립니다.",
    },
    Page {
        slug: "interpreter",
        title: "Interpreter 패턴",
        section: "디자인 패턴",
        file: "interpreter.md",
        summary: "작은 문법을 데이터 타입으로 표현하고 재귀적으로 평가합니다.",
    },
    Page {
        slug: "mediator",
        title: "Mediator 패턴",
        section: "디자인 패턴",
        file: "mediator.md",
        summary: "구성 요소 간의 통신과 협력 규칙을 중재자에게 모읍니다.",
    },
    Page {
        slug: "memento",
        title: "Memento 패턴",
        section: "디자인 패턴",
        file: "memento.md",
        summary: "객체의 내부 상태를 캡슐화해 저장하고 이전 상태로 복원합니다.",
    },
    Page {
        slug: "observer",
        title: "Observer 패턴",
        section: "디자인 패턴",
        file: "observer.md",
        summary: "상태가 바뀌면 등록된 관찰자들에게 변경 내용을 알립니다.",
    },
    Page {
        slug: "state",
        title: "State 패턴",
        section: "디자인 패턴",
        file: "state.md",
        summary: "현재 상태에 따라 동작과 허용되는 상태 전이를 변경합니다.",
    },
    Page {
        slug: "template-method",
        title: "Template Method 패턴",
        section: "디자인 패턴",
        file: "template-method.md",
        summary: "공통 알고리즘의 순서를 유지하면서 일부 단계를 교체합니다.",
    },
    Page {
        slug: "visitor",
        title: "Visitor 패턴",
        section: "디자인 패턴",
        file: "visitor.md",
        summary: "자료 구조와 그 구조에 수행할 연산을 분리합니다.",
    },
    Page {
        slug: "creational-patterns",
        title: "생성 패턴",
        section: "디자인 패턴",
        file: "creational-patterns.md",
        summary: "값과 자료 구조를 생성하는 책임을 조직하는 방법입니다.",
    },
    Page {
        slug: "fold",
        title: "Fold 패턴",
        section: "디자인 패턴",
        file: "fold.md",
        summary: "재귀적 자료 구조를 하나의 누적 결과로 접어 계산합니다.",
    },
    Page {
        slug: "prototype",
        title: "Prototype 패턴",
        section: "디자인 패턴",
        file: "prototype.md",
        summary: "등록된 원형 객체를 복제해 새 객체를 생성합니다.",
    },
    Page {
        slug: "abstract-factory",
        title: "Abstract Factory 패턴",
        section: "디자인 패턴",
        file: "abstract-factory.md",
        summary: "서로 호환되는 관련 제품군을 추상 인터페이스로 생성합니다.",
    },
    Page {
        slug: "factory-method",
        title: "Factory Method 패턴",
        section: "디자인 패턴",
        file: "factory-method.md",
        summary: "Creator의 생성 메서드로 구체 제품 선택을 분리합니다.",
    },
    Page {
        slug: "singleton",
        title: "Singleton 패턴",
        section: "디자인 패턴",
        file: "singleton.md",
        summary: "초기화된 전역 공유 인스턴스를 안전하게 제공합니다.",
    },
    Page {
        slug: "structural-patterns",
        title: "구조 패턴",
        section: "디자인 패턴",
        file: "structural-patterns.md",
        summary: "타입과 모듈의 구성 및 경계를 설계하는 패턴입니다.",
    },
    Page {
        slug: "adapter",
        title: "Adapter 패턴",
        section: "디자인 패턴",
        file: "adapter.md",
        summary: "기존 타입의 인터페이스를 클라이언트가 기대하는 API에 맞춥니다.",
    },
    Page {
        slug: "bridge",
        title: "Bridge 패턴",
        section: "디자인 패턴",
        file: "bridge.md",
        summary: "추상화와 구현을 분리해 각 축을 독립적으로 확장합니다.",
    },
    Page {
        slug: "composite",
        title: "Composite 패턴",
        section: "디자인 패턴",
        file: "composite.md",
        summary: "단일 객체와 객체의 트리를 같은 인터페이스로 다룹니다.",
    },
    Page {
        slug: "decorator",
        title: "Decorator 패턴",
        section: "디자인 패턴",
        file: "decorator.md",
        summary: "같은 인터페이스의 래퍼를 쌓아 객체의 동작을 확장합니다.",
    },
    Page {
        slug: "facade",
        title: "Facade 패턴",
        section: "디자인 패턴",
        file: "facade.md",
        summary: "여러 하위 시스템의 작업을 단순한 진입점으로 조정합니다.",
    },
    Page {
        slug: "flyweight",
        title: "Flyweight 패턴",
        section: "디자인 패턴",
        file: "flyweight.md",
        summary: "공유 가능한 상태를 재사용해 반복 객체의 메모리를 절약합니다.",
    },
    Page {
        slug: "proxy",
        title: "Proxy 패턴",
        section: "디자인 패턴",
        file: "proxy.md",
        summary: "실제 객체에 대한 접근과 생성 시점을 대리 객체로 제어합니다.",
    },
    Page {
        slug: "compose-structs",
        title: "독립적으로 빌리기 위한 구조체 분해",
        section: "디자인 패턴",
        file: "compose-structs.md",
        summary: "큰 구조체를 작은 타입으로 나누어 필드별 대여를 유연하게 합니다.",
    },
    Page {
        slug: "small-crates",
        title: "작은 크레이트를 선호하기",
        section: "디자인 패턴",
        file: "small-crates.md",
        summary: "응집도 높은 작은 크레이트로 의존성과 컴파일 범위를 관리합니다.",
    },
    Page {
        slug: "unsafe-modules",
        title: "unsafe를 작은 모듈에 가두기",
        section: "디자인 패턴",
        file: "unsafe-modules.md",
        summary: "안전하지 않은 구현을 좁은 경계에 격리하고 안전한 API를 제공합니다.",
    },
    Page {
        slug: "trait-bounds",
        title: "사용자 정의 트레이트로 복잡한 바운드 줄이기",
        section: "디자인 패턴",
        file: "trait-bounds.md",
        summary: "반복되는 타입 제약을 의미 있는 트레이트로 묶습니다.",
    },
    Page {
        slug: "ffi-patterns",
        title: "FFI 디자인 패턴",
        section: "디자인 패턴",
        file: "ffi-patterns.md",
        summary: "언어 간 경계에서 객체와 타입을 표현하는 설계 방법입니다.",
    },
    Page {
        slug: "ffi-object-apis",
        title: "FFI를 위한 객체 기반 API",
        section: "디자인 패턴",
        file: "ffi-object-apis.md",
        summary: "C 호출자에게 Rust 객체를 핸들로 노출하는 API를 구성합니다.",
    },
    Page {
        slug: "ffi-wrapper-types",
        title: "래퍼로 타입 통합하기",
        section: "디자인 패턴",
        file: "ffi-wrapper-types.md",
        summary: "외부 타입을 Rust 측 래퍼로 감싸 경계와 동작을 통제합니다.",
    },
    Page {
        slug: "refactoring",
        title: "리팩터링",
        section: "추가 자료",
        file: "refactoring.md",
        summary: "테스트를 보호막으로 삼아 작고 안전한 단계로 구조를 개선합니다.",
    },
];

const SOURCE_FILES: &[(&str, &str)] = &[
    ("introduction", "intro.md"),
    ("idioms", "idioms/index.md"),
    ("borrowed-types", "idioms/coercion-arguments.md"),
    ("constructor", "idioms/ctor.md"),
    ("default-trait", "idioms/default.md"),
    ("newtype", "patterns/behavioural/newtype.md"),
    ("builder", "patterns/creational/builder.md"),
    ("strategy", "patterns/behavioural/strategy.md"),
    ("raii", "patterns/behavioural/RAII.md"),
    ("anti-patterns", "anti_patterns/index.md"),
    ("clone-borrow-checker", "anti_patterns/borrow_clone.md"),
    ("functional-programming", "functional/paradigms.md"),
    ("translations", "translations.md"),
    ("design-resources", "additional_resources/index.md"),
    (
        "design-principles",
        "additional_resources/design-principles.md",
    ),
    ("deny-warnings", "anti_patterns/deny-warnings.md"),
    ("deref-polymorphism", "anti_patterns/deref.md"),
    ("functional-overview", "functional/index.md"),
    (
        "generics-as-type-classes",
        "functional/generics-type-classes.md",
    ),
    ("functional-optics", "functional/optics.md"),
    ("format-concat", "idioms/concat-format.md"),
    ("collections-deref", "idioms/deref.md"),
    ("destructor-finalization", "idioms/dtor-finally.md"),
    ("ffi-idioms", "idioms/ffi/intro.md"),
    ("ffi-errors", "idioms/ffi/errors.md"),
    ("ffi-accept-strings", "idioms/ffi/accepting-strings.md"),
    ("ffi-pass-strings", "idioms/ffi/passing-strings.md"),
    ("mem-take-replace", "idioms/mem-replace.md"),
    ("stack-dynamic-dispatch", "idioms/on-stack-dyn-dispatch.md"),
    ("option-iterator", "idioms/option-iter.md"),
    ("closure-capture", "idioms/pass-var-to-closure.md"),
    ("extensible-types", "idioms/priv-extend.md"),
    (
        "consumed-argument-error",
        "idioms/return-consumed-arg-on-error.md",
    ),
    ("rustdoc-init", "idioms/rustdoc-init.md"),
    ("temporary-mutability", "idioms/temporary-mutability.md"),
    ("design-patterns", "patterns/index.md"),
    ("behavioral-patterns", "patterns/behavioural/intro.md"),
    ("chain-of-responsibility", "patterns/behavioural/intro.md"),
    ("command", "patterns/behavioural/command.md"),
    ("interpreter", "patterns/behavioural/interpreter.md"),
    ("iterator", "patterns/behavioural/intro.md"),
    ("mediator", "patterns/behavioural/intro.md"),
    ("memento", "patterns/behavioural/intro.md"),
    ("observer", "patterns/behavioural/intro.md"),
    ("state", "patterns/behavioural/intro.md"),
    ("template-method", "patterns/behavioural/intro.md"),
    ("visitor", "patterns/behavioural/visitor.md"),
    ("creational-patterns", "patterns/creational/intro.md"),
    ("abstract-factory", "patterns/creational/intro.md"),
    ("factory-method", "patterns/creational/intro.md"),
    ("fold", "patterns/creational/fold.md"),
    ("prototype", "patterns/creational/intro.md"),
    ("singleton", "patterns/creational/intro.md"),
    ("structural-patterns", "patterns/structural/intro.md"),
    ("adapter", "patterns/structural/intro.md"),
    ("bridge", "patterns/structural/intro.md"),
    ("composite", "patterns/structural/intro.md"),
    ("decorator", "patterns/structural/intro.md"),
    ("facade", "patterns/structural/intro.md"),
    ("flyweight", "patterns/structural/intro.md"),
    ("proxy", "patterns/structural/intro.md"),
    ("compose-structs", "patterns/structural/compose-structs.md"),
    ("small-crates", "patterns/structural/small-crates.md"),
    ("unsafe-modules", "patterns/structural/unsafe-mods.md"),
    ("trait-bounds", "patterns/structural/trait-for-bounds.md"),
    ("ffi-patterns", "patterns/ffi/intro.md"),
    ("ffi-object-apis", "patterns/ffi/export.md"),
    ("ffi-wrapper-types", "patterns/ffi/wrappers.md"),
    ("refactoring", "refactoring/index.md"),
];

const STYLE: &str = r#"
:root {
  color-scheme: dark;
  --bg: #101317;
  --panel: #171c22;
  --panel-raised: #1d242c;
  --text: #e6e9ed;
  --muted: #a4adb8;
  --line: #2b333d;
  --accent: #f3a64a;
  --code: #111820;
}
:root[data-theme="light"] {
  color-scheme: light;
  --bg: #f5f7fa;
  --panel: #ffffff;
  --panel-raised: #eef1f5;
  --text: #202a35;
  --muted: #596774;
  --line: #d3dae2;
  --accent: #995500;
  --code: #f1f3f6;
}
:root[data-theme="sepia"] {
  color-scheme: light;
  --bg: #f4ecd8;
  --panel: #fbf5e8;
  --panel-raised: #efe4cc;
  --text: #433629;
  --muted: #776856;
  --line: #d6c6a9;
  --accent: #8a4b08;
  --code: #eee3cf;
}
:root[data-theme="ocean"] {
  color-scheme: dark;
  --bg: #0b1720;
  --panel: #11232e;
  --panel-raised: #183442;
  --text: #e0f2f6;
  --muted: #a1bbc4;
  --line: #2a4856;
  --accent: #56c2e0;
  --code: #0a1c26;
}
:root[data-theme="forest"] {
  color-scheme: dark;
  --bg: #101a14;
  --panel: #18251b;
  --panel-raised: #22332a;
  --text: #e3efe5;
  --muted: #a5bba9;
  --line: #31483a;
  --accent: #8bd19b;
  --code: #101e17;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--text); font: 16px/1.72 system-ui, -apple-system, "Segoe UI", sans-serif; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
.shell { min-height: 100vh; display: grid; grid-template-columns: 280px minmax(0, 1fr); align-items: start; }
aside { position: sticky; top: 0; height: 100vh; overflow-y: auto; background: var(--panel); border-right: 1px solid var(--line); padding: 27px 18px; }
.brand { display: block; color: var(--text); font-size: 18px; font-weight: 750; line-height: 1.3; margin: 0 8px 8px; }
.brand small { display: block; color: var(--muted); font-size: 12px; font-weight: 500; margin-top: 7px; }
.nav { margin-top: 25px; }
.nav a { display: block; color: var(--muted); border-radius: 7px; font-size: 14px; padding: 6px 9px; }
.nav a:hover, .nav a.active { background: var(--panel-raised); color: var(--text); text-decoration: none; }
.nav a.active-ancestor { color: var(--text); }
.nav-heading { align-items: center; display: flex; }
.nav-parent-link { flex: 1; }
.nav-toggle { background: transparent; border: 0; border-radius: 6px; color: var(--muted); cursor: pointer; font-size: 18px; height: 32px; width: 32px; }
.nav-toggle:hover { background: var(--panel-raised); color: var(--text); }
.nav-toggle::before { content: "›"; display: inline-block; transition: transform .15s ease; }
.nav-toggle[aria-expanded="true"]::before { transform: rotate(90deg); }
.nav-root > .nav-heading > .nav-parent-link { color: var(--text); font-weight: 650; margin-top: 10px; }
.nav-children { border-left: 1px solid var(--line); margin: 2px 0 6px 12px; padding-left: 8px; }
.toolbar { align-items: center; display: flex; gap: 8px; position: fixed; right: 18px; top: 16px; z-index: 10; }
.main-brand-logo { align-items: center; background: var(--panel-raised); border: 1px solid var(--line); border-radius: 8px; box-shadow: 0 4px 14px #0005; display: flex; height: 50px; left: max(calc(280px + clamp(22px, 7vw, 100px)), calc((100% + 280px - 850px) / 2)); padding: 4px 8px; position: fixed; top: 9px; z-index: 10; }
.main-brand-logo:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
.main-brand-logo img { display: block; height: 40px; width: auto; }
.toolbar-button { align-items: center; background: var(--panel-raised); border: 1px solid var(--line); border-radius: 8px; box-shadow: 0 4px 14px #0005; color: var(--text); cursor: pointer; display: flex; gap: 7px; line-height: 1.4; padding: 8px 11px; text-decoration: none; }
.toolbar-button:hover { border-color: var(--accent); text-decoration: none; }
.toolbar-button:focus-visible, .search-panel input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.toolbar-icon { font-size: 18px; line-height: 1; }
.theme-panel { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; box-shadow: 0 8px 28px #0007; display: grid; gap: 8px; padding: 12px; position: fixed; right: 18px; top: 66px; width: min(250px, calc(100vw - 36px)); z-index: 11; }
.theme-panel[hidden] { display: none; }
.theme-panel label { font-size: 14px; font-weight: 650; }
.theme-panel select { background: var(--bg); border: 1px solid var(--line); border-radius: 7px; color: var(--text); font: inherit; padding: 8px 10px; }
.theme-panel select:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.search-panel { align-items: center; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; box-shadow: 0 8px 28px #0007; display: flex; gap: 9px; padding: 12px; position: fixed; right: 18px; top: 66px; width: min(390px, calc(100vw - 36px)); z-index: 11; }
.search-panel[hidden] { display: none; }
.search-panel input, .search-page-form input { background: var(--bg); border: 1px solid var(--line); border-radius: 7px; color: var(--text); font: inherit; min-width: 0; padding: 8px 10px; }
.search-panel input { flex: 1; }
.visually-hidden { clip: rect(0, 0, 0, 0); clip-path: inset(50%); height: 1px; overflow: hidden; position: absolute; white-space: nowrap; width: 1px; }
.search-page-form { margin: 24px 0; }
.search-page-form label { display: block; font-weight: 650; margin-bottom: 8px; }
.search-page-form > div { display: flex; gap: 9px; }
.search-page-form input { flex: 1; }
.search-results { display: grid; gap: 10px; margin-top: 20px; }
.search-result { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; color: var(--text); display: block; padding: 14px 16px; }
.search-result:hover { border-color: var(--accent); text-decoration: none; }
.search-result strong { display: block; }
.search-result span { color: var(--muted); font-size: 14px; }
.page-navigation { border-top: 1px solid var(--line); display: grid; gap: 14px; grid-template-columns: repeat(2, minmax(0, 1fr)); margin-top: 42px; padding-top: 20px; }
.page-nav-link, .page-nav-placeholder { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; display: flex; flex-direction: column; justify-content: center; min-height: 78px; padding: 12px 15px; }
.page-nav-link { color: var(--text); }
.page-nav-link:hover { border-color: var(--accent); text-decoration: none; }
.page-nav-link.next { text-align: right; }
.page-nav-direction, .page-nav-placeholder { color: var(--muted); font-size: 13px; }
.page-nav-link strong { margin-top: 3px; }
.shell.nav-hidden { grid-template-columns: minmax(0, 1fr); }
.shell.nav-hidden aside { display: none; }
.shell.nav-hidden .main-brand-logo { left: max(clamp(22px, 7vw, 100px), calc((100% - 850px) / 2)); }
main { min-width: 0; padding: 64px clamp(22px, 7vw, 100px) 70px; }
.content { max-width: 850px; margin: auto; }
.eyebrow { color: var(--accent); font-size: 12px; font-weight: 700; letter-spacing: .13em; text-transform: uppercase; }
h1 { font-size: clamp(32px, 5vw, 50px); letter-spacing: -.035em; line-height: 1.15; margin: 9px 0 16px; }
.page-title { font-size: clamp(30px, 4vw, 42px); }
article h1 { font-size: clamp(26px, 3.2vw, 34px); letter-spacing: -.025em; }
h2 { border-bottom: 1px solid var(--line); font-size: 25px; margin-top: 42px; padding-bottom: 9px; }
h3 { font-size: 19px; margin-top: 28px; }
.lead { color: var(--muted); font-size: 18px; max-width: 720px; }
.page-meta, .notice { color: var(--muted); font-size: 13px; }
.notice { background: var(--panel-raised); border: 1px solid var(--line); border-radius: 10px; margin: 28px 0; padding: 14px 17px; }
.notice strong { color: var(--text); }
.cards { display: grid; gap: 14px; grid-template-columns: repeat(auto-fit, minmax(230px, 1fr)); margin: 30px 0; }
.card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; color: var(--text); display: block; padding: 18px; }
.card:hover { border-color: var(--accent); text-decoration: none; }
.card strong { display: block; margin-bottom: 6px; }
.card span { color: var(--muted); font-size: 14px; }
pre { background: var(--code); border: 1px solid var(--line); border-radius: 9px; overflow-x: auto; padding: 16px; }
code { font-family: "Cascadia Code", "SFMono-Regular", Consolas, monospace; font-size: .9em; }
:not(pre) > code { background: var(--panel-raised); border-radius: 4px; padding: 2px 5px; }
blockquote { border-left: 3px solid var(--accent); color: var(--muted); margin: 20px 0; padding: 3px 17px; }
li { margin: 6px 0; }
table { border-collapse: collapse; display: block; overflow-x: auto; }
th, td { border: 1px solid var(--line); padding: 8px 12px; }
th { background: var(--panel); }
footer { border-top: 1px solid var(--line); color: var(--muted); font-size: 13px; margin-top: 55px; padding-top: 18px; }
@media (max-width: 760px) {
  .shell { display: block; }
  aside { position: static; height: auto; overflow: visible; border-bottom: 1px solid var(--line); border-right: 0; padding: 62px 18px 15px; }
  .nav { margin-top: 13px; max-height: 55vh; overflow-y: auto; }
  .nav a { padding: 4px 7px; }
  main { padding-top: 64px; }
  .main-brand-logo, .shell.nav-hidden .main-brand-logo { left: 18px; }
  .page-navigation { grid-template-columns: 1fr; }
  .page-nav-link.next { text-align: left; }
}
@media (max-width: 460px) {
  .toolbar { gap: 6px; }
  .toolbar-button { padding: 8px; }
  .toolbar-button span:not(.toolbar-icon) { display: none; }
}
@media (max-width: 360px) {
  .main-brand-logo img { height: auto; width: clamp(0px, calc(100vw - 190px), 130px); }
}
"#;

const NAV_SCRIPT: &str = r##"
const layout = document.querySelector(".shell");
const menuButton = document.querySelector(".menu-toggle");
const searchButton = document.querySelector(".search-toggle");
const searchPanel = document.querySelector("#site-search");
const searchInput = document.querySelector("#site-search-input");
const themeButton = document.querySelector(".theme-picker-toggle");
const themePanel = document.querySelector("#theme-panel");
const themeSelect = document.querySelector("#theme-select");
const themeColors = {
  dark: "#101317",
  light: "#f5f7fa",
  sepia: "#f4ecd8",
  ocean: "#0b1720",
  forest: "#101a14",
};

menuButton.addEventListener("click", () => {
  const expanded = menuButton.getAttribute("aria-expanded") === "true";
  menuButton.setAttribute("aria-expanded", String(!expanded));
  layout.classList.toggle("nav-hidden", expanded);
});

const themeStorageKey = "rust-pattern-theme";
let savedTheme = null;
try {
  savedTheme = localStorage.getItem(themeStorageKey);
} catch (error) {
  console.error("테마 설정을 읽지 못했습니다.", error);
}

function applyTheme(theme) {
  if (!Object.prototype.hasOwnProperty.call(themeColors, theme)) {
    console.warn("지원하지 않는 테마 설정입니다. 기본 테마를 적용합니다.", theme);
    theme = "dark";
  }
  document.documentElement.dataset.theme = theme;
  themeSelect.value = theme;
  document.querySelector('meta[name="theme-color"]').setAttribute(
    "content",
    themeColors[theme]
  );
}

applyTheme(savedTheme || "dark");

function closeSearchPanel() {
  searchPanel.hidden = true;
  searchButton.setAttribute("aria-expanded", "false");
}

function closeThemePanel() {
  themePanel.hidden = true;
  themeButton.setAttribute("aria-expanded", "false");
}

themeButton.addEventListener("click", () => {
  const expanded = themeButton.getAttribute("aria-expanded") === "true";
  closeSearchPanel();
  themePanel.hidden = expanded;
  themeButton.setAttribute("aria-expanded", String(!expanded));
  if (!expanded) themeSelect.focus();
});

themeSelect.addEventListener("change", () => {
  const theme = themeSelect.value;
  applyTheme(theme);
  try {
    localStorage.setItem(themeStorageKey, theme);
  } catch (error) {
    console.error("테마 설정을 저장하지 못했습니다.", error);
  }
  closeThemePanel();
  themeButton.focus();
});

searchButton.addEventListener("click", () => {
  const expanded = searchButton.getAttribute("aria-expanded") === "true";
  closeThemePanel();
  searchButton.setAttribute("aria-expanded", String(!expanded));
  searchPanel.hidden = expanded;
  if (!expanded) searchInput.focus();
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    if (!searchPanel.hidden) {
      closeSearchPanel();
      searchButton.focus();
    } else if (!themePanel.hidden) {
      closeThemePanel();
      themeButton.focus();
    }
  }
});

document.querySelector(".nav").addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const button = target.closest(".nav-toggle");
  if (!button) return;
  const group = button.closest(".nav-group");
  const children = group.querySelector(":scope > .nav-children");
  const expanded = button.getAttribute("aria-expanded") === "true";
  button.setAttribute("aria-expanded", String(!expanded));
  children.hidden = expanded;
});
"##;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(error) = dotenvy::dotenv() {
        if !matches!(&error, dotenvy::Error::Io(io) if io.kind() == std::io::ErrorKind::NotFound) {
            return Err(error.into());
        }
    }
    let (host, port) = server_address(
        environment_or_default("HOST", "0.0.0.0")?,
        &environment_or_default("PORT", "3000")?,
    )?;

    let app = Router::new()
        .route("/", get(home))
        .route("/search", get(search))
        .route("/docs/{slug}", get(document))
        .route("/assets/cozysoft-logo.svg", get(cozysoft_logo))
        .route("/health", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    let address = listener.local_addr()?;
    println!("서버 수신 주소: {address}");
    if address.ip().is_unspecified() {
        let loopback = if address.is_ipv4() {
            "127.0.0.1"
        } else {
            "[::1]"
        };
        println!(
            "한국어 Rust 패턴 안내서: http://{loopback}:{}",
            address.port()
        );
        if address.is_ipv4() {
            println!(
                "스마트폰은 같은 Wi-Fi에서 http://<PC의 IPv4 주소>:{} 으로 접속하세요.",
                address.port()
            );
        }
    } else {
        println!("한국어 Rust 패턴 안내서: http://{address}");
    }
    axum::serve(listener, app).await?;
    Ok(())
}

async fn cozysoft_logo() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!("../public/assets/cozysoft-logo.svg"),
    )
}

async fn home() -> Html<String> {
    let cards = PAGES.iter().map(|page| {
        html! {
            a.card href=(format!("/docs/{}", page.slug)) {
                strong { (page.title) }
                span { (page.summary) }
            }
        }
    });

    let content = html! {
        div class="eyebrow" { "RUST DESIGN PATTERNS · 한국어" }
        h1 { "Rust 디자인 패턴" }
        p class="lead" {
            "Rust다운 관용구와 디자인 패턴을 한국어로 읽고, 각 선택의 장단점을 이해하기 위한 안내서입니다."
        }
        div class="notice" {
            strong { "번역 범위 안내. " }
            "원문의 51개 문서를 한국어 Markdown으로 번역했습니다. 코드 예제는 원문을 유지하고 설명과 안내를 한국어로 제공합니다. "
            a href="https://rust-unofficial.github.io/patterns/" { "영문 원문 보기" }
            "."
        }
        h2 { "문서 둘러보기" }
        div class="cards" { @for card in cards { (card) } }
        p class="page-meta" {
            "원작: rust-unofficial/patterns · "
            a href="https://github.com/rust-unofficial/patterns" { "MPL-2.0 라이선스 및 출처" }
        }
    };

    layout(None, "Rust 디자인 패턴 한국어 안내서", content)
}

async fn search(Query(params): Query<HashMap<String, String>>) -> Result<Html<String>, StatusCode> {
    let query = params.get("q").map(String::as_str).unwrap_or("").trim();
    let normalized_query = query.to_lowercase();
    let mut matches = Vec::new();

    if !normalized_query.is_empty() {
        for page in PAGES {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("content")
                .join("ko")
                .join(page.file);
            let markdown = tokio::fs::read_to_string(&path).await.map_err(|error| {
                eprintln!("검색 문서를 읽지 못했습니다 ({}): {error}", path.display());
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let searchable = format!("{} {} {}", page.title, page.summary, markdown);
            if searchable.to_lowercase().contains(&normalized_query) {
                matches.push(page);
            }
        }
    }

    let result_count = matches.len();
    let content = html! {
        div class="eyebrow" { "문서 탐색" }
        h1 { "문서 검색" }
        form class="search-page-form" action="/search" method="get" role="search" {
            label for="search-query" { "검색어" }
            div {
                input
                    id="search-query"
                    type="search"
                    name="q"
                    value=(query)
                    placeholder="문서 제목과 본문 검색";
                button class="toolbar-button" type="submit" { "검색" }
            }
        }
        @if query.is_empty() {
            p class="page-meta" { "검색어를 입력하면 51개 문서의 제목과 본문을 검색합니다." }
        } @else {
            p class="page-meta" {
                "“" (query) "” 검색 결과: " (result_count) "개"
            }
            @if matches.is_empty() {
                p { "검색 결과가 없습니다. 다른 검색어를 입력해 보세요." }
            } @else {
                div class="search-results" {
                    @for page in matches {
                        a class="search-result" href=(format!("/docs/{}", page.slug)) {
                            strong { (page.title) }
                            span { (page.summary) }
                        }
                    }
                }
            }
        }
    };

    Ok(layout(None, "문서 검색", content))
}

async fn document(Path(slug): Path<String>) -> Result<Html<String>, StatusCode> {
    let Some(page) = PAGES.iter().find(|page| page.slug == slug) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let source_path = source_path(page.slug).ok_or_else(|| {
        eprintln!("원문 경로를 찾을 수 없습니다: {}", page.slug);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let source_url =
        format!("https://github.com/rust-unofficial/patterns/blob/main/src/{source_path}");
    let (previous, next) = adjacent_pages(page.slug);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("content")
        .join("ko")
        .join(page.file);
    let markdown = tokio::fs::read_to_string(&path).await.map_err(|error| {
        eprintln!("문서 파일을 읽지 못했습니다 ({}): {error}", path.display());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let mut rendered = String::new();
    markdown_html::push_html(&mut rendered, Parser::new_ext(&markdown, options));

    let content = html! {
        div class="eyebrow" { (page.section) }
        h1 class="page-title" { (page.title) }
        p class="lead" { (page.summary) }
        article { (PreEscaped(rendered)) }
        nav class="page-navigation" aria-label="문서 이동" {
            @if let Some(previous) = previous {
                a class="page-nav-link previous"
                    href=(format!("/docs/{}", previous.slug))
                    aria-label=(format!("이전 페이지: {}", previous.title)) {
                    span class="page-nav-direction" { "← 이전 페이지" }
                    strong { (previous.title) }
                }
            } @else {
                div class="page-nav-placeholder" { "첫 페이지입니다" }
            }
            @if let Some(next) = next {
                a class="page-nav-link next"
                    href=(format!("/docs/{}", next.slug))
                    aria-label=(format!("다음 페이지: {}", next.title)) {
                    span class="page-nav-direction" { "다음 페이지 →" }
                    strong { (next.title) }
                }
            } @else {
                div class="page-nav-placeholder next" { "마지막 페이지입니다" }
            }
        }
        footer {
            "이 페이지의 원문: "
            a href=(source_url) target="_blank" rel="noopener noreferrer" {
                "GitHub에서 원문 보기 ↗"
            }
            " · "
            "번역 기반: "
            a href="https://github.com/rust-unofficial/patterns" { "Rust Design Patterns" }
            " · "
            a href="https://github.com/rust-unofficial/patterns/blob/main/LICENSE" {
                "MPL-2.0 라이선스"
            }
            "를 확인하세요."
        }
    };

    Ok(layout(Some(page.slug), page.title, content))
}

fn layout(active: Option<&str>, title: &str, content: Markup) -> Html<String> {
    let page = html! {
        (DOCTYPE)
        html lang="ko" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="theme-color" content="#101317";
                title { (title) " · Rust 패턴 한국어 안내서" }
                style { (PreEscaped(STYLE)) }
            }
            body {
                div class="toolbar" role="toolbar" aria-label="페이지 도구" {
                    button
                        class="toolbar-button menu-toggle"
                        type="button"
                        aria-label="목차 표시 전환"
                        aria-controls="site-navigation"
                        aria-expanded="true" {
                        span class="toolbar-icon" aria-hidden="true" { "☰" }
                        span { "목차" }
                    }
                    button
                        class="toolbar-button search-toggle"
                        type="button"
                        aria-label="검색 열기"
                        aria-controls="site-search"
                        aria-expanded="false" {
                        span class="toolbar-icon" aria-hidden="true" { "⌕" }
                        span { "검색" }
                    }
                    button
                        class="toolbar-button theme-picker-toggle"
                        type="button"
                        aria-label="테마 선택"
                        aria-controls="theme-panel"
                        aria-expanded="false" {
                        span class="toolbar-icon" aria-hidden="true" { "◐" }
                        span { "테마" }
                    }
                }
                div id="theme-panel" class="theme-panel" hidden {
                    label for="theme-select" { "테마 선택" }
                    select id="theme-select" {
                        option value="dark" { "어두운" }
                        option value="light" { "밝은" }
                        option value="sepia" { "세피아" }
                        option value="ocean" { "오션" }
                        option value="forest" { "포레스트" }
                    }
                }
                form id="site-search" class="search-panel" action="/search" method="get" role="search" hidden {
                    label class="visually-hidden" for="site-search-input" { "문서 검색" }
                    input
                        id="site-search-input"
                        type="search"
                        name="q"
                        placeholder="문서 제목과 본문 검색";
                    button class="toolbar-button" type="submit" { "검색" }
                }
                div class="shell" {
                    aside id="site-navigation" {
                        a class="brand" href="/" {
                            "Rust 패턴 안내서"
                            small { "한국어 Markdown 에디션" }
                        }
                        nav class="nav" aria-label="문서 목차" {
                            (render_toc(TOC, active, 0))
                        }
                    }
                    main {
                        a
                            class="main-brand-logo"
                            href="https://www.cozysoft.kr"
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label="Cozysoft 웹사이트 (새 탭에서 열림)" {
                            img src="/assets/cozysoft-logo.svg" alt="" width="110" height="32";
                        }
                        div class="content" { (content) }
                    }
                }
                script { (PreEscaped(NAV_SCRIPT)) }
            }
        }
    };
    Html(page.into_string())
}

fn render_toc(items: &[TocItem], active: Option<&str>, depth: usize) -> Markup {
    html! {
        @for item in items {
            @if item.children.is_empty() {
                a href=(format!("/docs/{}", item.slug)) class=(toc_class(item, active)) {
                    (item.title)
                }
            } @else {
                    div class=(if depth == 0 { "nav-group nav-root" } else { "nav-group" }) {
                        div class="nav-heading" {
                            a href=(format!("/docs/{}", item.slug))
                                class=(format!("nav-parent-link {}", toc_class(item, active))) {
                                (item.title)
                            }
                            button
                                type="button"
                                class="nav-toggle"
                                aria-expanded=(if group_is_open(item, active) { "true" } else { "false" })
                                aria-label=(format!("{} 하위 목차 표시 전환", item.title)) {}
                        }
                        div class="nav-children" hidden[!group_is_open(item, active)] {
                        (render_toc(item.children, active, depth + 1))
                    }
                }
            }
        }
    }
}

fn group_is_open(item: &TocItem, active: Option<&str>) -> bool {
    active.is_some_and(|slug| contains_page(item, slug))
}

fn adjacent_pages(slug: &str) -> (Option<&'static Page>, Option<&'static Page>) {
    fn collect_pages<'a>(items: &'a [TocItem], pages: &'a [Page], ordered: &mut Vec<&'a Page>) {
        for item in items {
            if let Some(page) = pages.iter().find(|page| page.slug == item.slug) {
                ordered.push(page);
            }
            collect_pages(item.children, pages, ordered);
        }
    }

    let mut ordered = Vec::with_capacity(PAGES.len());
    collect_pages(TOC, PAGES, &mut ordered);
    let Some(index) = ordered.iter().position(|page| page.slug == slug) else {
        return (None, None);
    };
    let previous = index.checked_sub(1).map(|previous| ordered[previous]);
    let next = ordered.get(index + 1).copied();
    (previous, next)
}

fn source_path(slug: &str) -> Option<&'static str> {
    SOURCE_FILES
        .iter()
        .find_map(|(page_slug, path)| (*page_slug == slug).then_some(*path))
}

fn toc_class(item: &TocItem, active: Option<&str>) -> &'static str {
    match active {
        Some(slug) if item.slug == slug => "active",
        Some(slug) if contains_page(item, slug) => "active-ancestor",
        _ => "",
    }
}

fn contains_page(item: &TocItem, slug: &str) -> bool {
    item.slug == slug || item.children.iter().any(|child| contains_page(child, slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_address_accepts_host_and_port() {
        assert_eq!(
            server_address("127.0.0.1".into(), "8080").unwrap(),
            ("127.0.0.1".into(), 8080)
        );
    }

    #[test]
    fn server_address_rejects_empty_host_and_invalid_port() {
        assert!(server_address(" ".into(), "3000").is_err());
        assert!(server_address("localhost".into(), "not-a-port").is_err());
        assert!(server_address("localhost".into(), "65536").is_err());
    }

    #[test]
    fn markdown_keeps_rust_code_blocks() {
        let markdown = "```rust\nlet answer = 42;\n```";
        let mut html = String::new();
        markdown_html::push_html(&mut html, Parser::new(markdown));
        assert!(html.contains("<code"));
        assert!(html.contains("let answer = 42;"));
    }

    #[test]
    fn every_navigation_page_has_a_markdown_file() {
        for page in PAGES {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("content")
                .join("ko")
                .join(page.file);
            assert!(path.is_file(), "missing document: {}", path.display());
        }
    }

    #[test]
    fn table_of_contents_titles_match_page_titles() {
        fn check_titles(items: &[TocItem]) {
            for item in items {
                let page = PAGES
                    .iter()
                    .find(|page| page.slug == item.slug)
                    .expect("table of contents entry must have a page");
                assert_eq!(item.title, page.title, "title mismatch: {}", item.slug);
                check_titles(item.children);
            }
        }

        check_titles(TOC);
    }

    #[test]
    fn every_page_has_one_original_source_reference() {
        assert_eq!(SOURCE_FILES.len(), PAGES.len());

        for page in PAGES {
            assert!(
                source_path(page.slug).is_some(),
                "missing original source reference: {}",
                page.slug
            );
            assert_eq!(
                SOURCE_FILES
                    .iter()
                    .filter(|(slug, _)| *slug == page.slug)
                    .count(),
                1,
                "page must have exactly one original source reference: {}",
                page.slug
            );
        }
    }

    #[test]
    fn table_of_contents_contains_each_page_once() {
        fn collect_slugs(items: &[TocItem], slugs: &mut Vec<&'static str>) {
            for item in items {
                slugs.push(item.slug);
                collect_slugs(item.children, slugs);
            }
        }

        let mut slugs = Vec::new();
        collect_slugs(TOC, &mut slugs);
        assert_eq!(slugs.len(), PAGES.len());

        for page in PAGES {
            assert_eq!(
                slugs.iter().filter(|slug| **slug == page.slug).count(),
                1,
                "page must appear exactly once in the table of contents: {}",
                page.slug
            );
        }
    }

    #[test]
    fn toc_groups_open_only_for_the_active_page_branch() {
        let patterns = TOC
            .iter()
            .find(|item| item.slug == "design-patterns")
            .expect("design patterns group exists");
        assert!(!group_is_open(patterns, None));
        assert!(group_is_open(patterns, Some("command")));
        assert!(!group_is_open(patterns, Some("constructor")));
    }

    #[test]
    fn adjacent_pages_follow_the_table_of_contents_order() {
        let (previous, next) = adjacent_pages("command");
        assert_eq!(
            previous.map(|page| page.slug),
            Some("chain-of-responsibility")
        );
        assert_eq!(next.map(|page| page.slug), Some("interpreter"));

        let (previous, next) = adjacent_pages("chain-of-responsibility");
        assert_eq!(previous.map(|page| page.slug), Some("behavioral-patterns"));
        assert_eq!(next.map(|page| page.slug), Some("command"));

        let (previous, next) = adjacent_pages("iterator");
        assert_eq!(previous.map(|page| page.slug), Some("interpreter"));
        assert_eq!(next.map(|page| page.slug), Some("mediator"));

        let (previous, next) = adjacent_pages("mediator");
        assert_eq!(previous.map(|page| page.slug), Some("iterator"));
        assert_eq!(next.map(|page| page.slug), Some("memento"));

        let (previous, next) = adjacent_pages("memento");
        assert_eq!(previous.map(|page| page.slug), Some("mediator"));
        assert_eq!(next.map(|page| page.slug), Some("newtype"));

        let (previous, next) = adjacent_pages("observer");
        assert_eq!(previous.map(|page| page.slug), Some("newtype"));
        assert_eq!(next.map(|page| page.slug), Some("raii"));

        let (previous, next) = adjacent_pages("state");
        assert_eq!(previous.map(|page| page.slug), Some("raii"));
        assert_eq!(next.map(|page| page.slug), Some("strategy"));

        let (previous, next) = adjacent_pages("template-method");
        assert_eq!(previous.map(|page| page.slug), Some("strategy"));
        assert_eq!(next.map(|page| page.slug), Some("visitor"));

        let (previous, next) = adjacent_pages("introduction");
        assert!(previous.is_none());
        assert_eq!(next.map(|page| page.slug), Some("translations"));

        let (previous, next) = adjacent_pages("refactoring");
        assert_eq!(previous.map(|page| page.slug), Some("design-principles"));
        assert!(next.is_none());
    }

    #[test]
    fn layout_has_an_accessible_navigation_toggle() {
        let rendered = layout(None, "테스트", html! { p { "내용" } }).0;

        assert!(rendered.contains("class=\"main-brand-logo\""));
        let main_start = rendered.find("<main>").unwrap();
        let logo_start = rendered.find("class=\"main-brand-logo\"").unwrap();
        let content_start = rendered.find("class=\"content\"").unwrap();
        assert!(main_start < logo_start && logo_start < content_start);
        assert!(rendered.contains("href=\"https://www.cozysoft.kr\""));
        assert!(rendered.contains("src=\"/assets/cozysoft-logo.svg\""));
        assert!(rendered.contains("target=\"_blank\""));
        assert!(rendered.contains("rel=\"noopener noreferrer\""));
        assert!(rendered.contains("class=\"toolbar-button menu-toggle\""));
        assert!(rendered.contains("class=\"toolbar-button search-toggle\""));
        assert!(rendered.contains("class=\"toolbar-button theme-picker-toggle\""));
        assert!(rendered.contains("aria-label=\"목차 표시 전환\""));
        assert!(rendered.contains("aria-label=\"검색 열기\""));
        assert!(rendered.contains("aria-controls=\"site-navigation\""));
        assert!(rendered.contains("aria-controls=\"site-search\""));
        assert!(rendered.contains("aria-controls=\"theme-panel\""));
        assert!(rendered.contains("aria-expanded=\"true\""));
        assert!(rendered.contains("id=\"site-navigation\""));
        assert!(rendered.contains("id=\"site-search\""));
        assert!(rendered.contains("id=\"theme-panel\""));
        assert!(rendered.contains("value=\"sepia\""));
        assert!(rendered.contains("value=\"ocean\""));
        assert!(rendered.contains("value=\"forest\""));
    }
}
