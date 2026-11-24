# Abstract Factory 패턴

## 의도

Abstract Factory(추상 팩토리) 패턴은 서로 관련되어 함께 사용되는 객체들의 제품군을 구체 타입을 직접 지정하지 않고 생성하는 방법을 제공합니다. 클라이언트는 제품군마다 제공되는 생성 인터페이스에 의존하고, 실행 환경이나 설정에 따라 선택된 구체 팩토리가 일관된 제품들을 만듭니다.

예를 들어 Windows용 버튼과 체크박스, macOS용 버튼과 체크박스가 있다고 합시다. 클라이언트가 제품을 직접 생성하면 서로 다른 스타일의 위젯을 섞을 수 있습니다. Abstract Factory는 한 팩토리가 같은 계열의 위젯을 함께 만들도록 구성해 제품군의 일관성을 유지합니다.

## 구성 요소

- **추상 제품(Abstract Product):** 제품군에 속하는 각 제품 종류가 제공할 공통 동작을 정의합니다. 예를 들어 `Button`과 `Checkbox`가 있습니다.
- **구체 제품(Concrete Product):** 공통 제품 트레이트를 특정 변형에 맞게 구현합니다. 예를 들어 Windows 버튼과 macOS 버튼이 있습니다.
- **추상 팩토리(Abstract Factory):** 제품군의 각 제품을 생성하는 메서드를 선언합니다.
- **구체 팩토리(Concrete Factory):** 추상 팩토리를 구현하고 특정 변형에 속하는 제품들을 생성합니다.
- **클라이언트(Client):** 구체 제품이나 구체 팩토리 대신 추상 인터페이스를 사용합니다.

애플리케이션 초기화 단계에서 설정이나 실행 환경을 보고 구체 팩토리 하나를 선택한 뒤, 나머지 코드는 추상 팩토리와 제품 인터페이스만 사용하게 할 수 있습니다.

## Rust 예제: 플랫폼별 UI 위젯 제품군

Rust에는 클래스 상속이 없으므로 제품과 팩토리의 공통 API를 트레이트로 표현합니다. 다음 예제에서는 `WidgetFactory`가 버튼과 체크박스를 모두 생성하며, Windows와 macOS 팩토리가 각 플랫폼에 맞는 제품군을 반환합니다.

`src/lib.rs`:

```rust
pub trait Button {
    fn render(&self) -> &'static str;
}

pub trait Checkbox {
    fn render(&self) -> &'static str;
}

pub trait WidgetFactory {
    fn create_button(&self) -> Box<dyn Button>;
    fn create_checkbox(&self) -> Box<dyn Checkbox>;
}

struct WindowsButton;
struct WindowsCheckbox;
struct MacButton;
struct MacCheckbox;

impl Button for WindowsButton {
    fn render(&self) -> &'static str {
        "Windows 버튼"
    }
}

impl Checkbox for WindowsCheckbox {
    fn render(&self) -> &'static str {
        "Windows 체크박스"
    }
}

impl Button for MacButton {
    fn render(&self) -> &'static str {
        "macOS 버튼"
    }
}

impl Checkbox for MacCheckbox {
    fn render(&self) -> &'static str {
        "macOS 체크박스"
    }
}

pub struct WindowsFactory;
pub struct MacFactory;

impl WidgetFactory for WindowsFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(WindowsButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(WindowsCheckbox)
    }
}

impl WidgetFactory for MacFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(MacButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(MacCheckbox)
    }
}

pub struct SettingsScreen {
    factory: Box<dyn WidgetFactory>,
}

impl SettingsScreen {
    pub fn new(factory: Box<dyn WidgetFactory>) -> Self {
        Self { factory }
    }

    pub fn render(&self) -> Vec<&'static str> {
        let button = self.factory.create_button();
        let checkbox = self.factory.create_checkbox();
        vec![button.render(), checkbox.render()]
    }
}

#[cfg(test)]
mod tests {
    use super::{MacFactory, SettingsScreen, WidgetFactory, WindowsFactory};

    #[test]
    fn windows_factory_creates_a_matching_product_family() {
        let factory = WindowsFactory;

        assert_eq!(factory.create_button().render(), "Windows 버튼");
        assert_eq!(factory.create_checkbox().render(), "Windows 체크박스");
    }

    #[test]
    fn mac_factory_creates_a_matching_product_family() {
        let factory = MacFactory;

        assert_eq!(factory.create_button().render(), "macOS 버튼");
        assert_eq!(factory.create_checkbox().render(), "macOS 체크박스");
    }

    #[test]
    fn client_uses_any_factory_through_the_same_interface() {
        let factories: [Box<dyn WidgetFactory>; 2] =
            [Box::new(WindowsFactory), Box::new(MacFactory)];
        let expected = [
            vec!["Windows 버튼", "Windows 체크박스"],
            vec!["macOS 버튼", "macOS 체크박스"],
        ];

        for (factory, expected_widgets) in factories.into_iter().zip(expected) {
            let screen = SettingsScreen::new(factory);
            assert_eq!(screen.render(), expected_widgets);
        }
    }
}
```

`SettingsScreen`은 어떤 플랫폼 팩토리를 받았는지 알지 못합니다. 버튼과 체크박스의 공통 인터페이스를 통해 위젯을 생성하고 그 동작을 사용합니다. 애플리케이션은 시작할 때 설정에 맞는 `WindowsFactory` 또는 `MacFactory`를 선택해 `SettingsScreen`에 전달하면 됩니다.

이 예제의 `Box<dyn Trait>`은 런타임에 서로 다른 구체 팩토리와 제품을 같은 인터페이스로 다루기 위한 동적 디스패치입니다. 각 제품을 힙에 할당하며 메서드 호출도 동적으로 전달됩니다. 팩토리 선택이 컴파일 시점에 정해지고 정적 디스패치가 더 적절하다면, 제네릭이나 연관 타입으로 표현할 수도 있습니다.

## Rust 예제: 라이트·다크 UI 테마 제품군

Abstract Factory는 플랫폼이 아닌 테마를 기준으로 제품군을 나누는 데도 사용할 수 있습니다. 아래 `ThemeFactory`는 버튼 스타일과 패널 스타일을 함께 생성합니다. 화면은 두 구체 테마 중 무엇이 선택되었는지 알지 못하며, 같은 테마 팩토리에서 받은 스타일들을 적용합니다.

이 예제는 색상 설정을 값 객체로 모델링합니다. 실제 UI에서는 컴포넌트의 크기, 테두리, 상태별 색상 등도 스타일 타입에 추가할 수 있습니다. 아래 색상은 패턴 구조를 설명하기 위한 예시이므로 실제 서비스에서는 목표 플랫폼과 접근성 기준에 맞춰 검증해야 합니다.

`src/lib.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct ButtonStyle {
    pub background: &'static str,
    pub foreground: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PanelStyle {
    pub background: &'static str,
    pub border: &'static str,
}

pub trait ThemeFactory {
    fn button_style(&self) -> ButtonStyle;
    fn panel_style(&self) -> PanelStyle;
}

pub struct LightThemeFactory;
pub struct DarkThemeFactory;

impl ThemeFactory for LightThemeFactory {
    fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            background: "#1565C0",
            foreground: "#FFFFFF",
        }
    }

    fn panel_style(&self) -> PanelStyle {
        PanelStyle {
            background: "#FFFFFF",
            border: "#CFD8DC",
        }
    }
}

impl ThemeFactory for DarkThemeFactory {
    fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            background: "#90CAF9",
            foreground: "#102A43",
        }
    }

    fn panel_style(&self) -> PanelStyle {
        PanelStyle {
            background: "#202124",
            border: "#5F6368",
        }
    }
}

pub struct LoginScreen {
    theme: Box<dyn ThemeFactory>,
}

impl LoginScreen {
    pub fn new(theme: Box<dyn ThemeFactory>) -> Self {
        Self { theme }
    }

    pub fn styles(&self) -> (ButtonStyle, PanelStyle) {
        (self.theme.button_style(), self.theme.panel_style())
    }
}

#[cfg(test)]
mod tests {
    use super::{DarkThemeFactory, LightThemeFactory, LoginScreen, ThemeFactory};

    #[test]
    fn light_factory_creates_a_cohesive_light_theme() {
        let theme = LightThemeFactory;
        let button = theme.button_style();
        let panel = theme.panel_style();

        assert_eq!(button.background, "#1565C0");
        assert_eq!(button.foreground, "#FFFFFF");
        assert_eq!(panel.background, "#FFFFFF");
        assert_eq!(panel.border, "#CFD8DC");
    }

    #[test]
    fn dark_factory_creates_a_cohesive_dark_theme() {
        let theme = DarkThemeFactory;
        let button = theme.button_style();
        let panel = theme.panel_style();

        assert_eq!(button.background, "#90CAF9");
        assert_eq!(button.foreground, "#102A43");
        assert_eq!(panel.background, "#202124");
        assert_eq!(panel.border, "#5F6368");
    }

    #[test]
    fn screen_can_switch_theme_families_without_changing_its_api() {
        let themes: [Box<dyn ThemeFactory>; 2] =
            [Box::new(LightThemeFactory), Box::new(DarkThemeFactory)];
        let expected_panel_backgrounds = ["#FFFFFF", "#202124"];

        for (theme, expected_background) in themes.into_iter().zip(expected_panel_backgrounds) {
            let screen = LoginScreen::new(theme);
            let (button, panel) = screen.styles();

            assert_eq!(panel.background, expected_background);
            assert_ne!(button.background, panel.background);
        }
    }
}
```

`LoginScreen`은 `ThemeFactory`만 알고 있으며, 테마 변경은 생성 시 다른 팩토리를 전달하는 것으로 끝납니다. 따라서 버튼만 다크 테마로 바꾸고 패널은 라이트 테마로 남기는 식의 우발적인 혼합을 피할 수 있습니다.

## 테스트 코드

각 `src/lib.rs` 예제는 독립적인 Rust 라이브러리로 실행할 수 있으며, 단위 테스트로 다음 동작을 확인합니다.

1. 플랫폼별 위젯 예제에서 Windows와 macOS 팩토리가 각각 자신의 버튼과 체크박스 제품군을 생성합니다.
2. 라이트·다크 테마 예제에서 각 팩토리가 해당 테마의 버튼과 패널 스타일을 생성합니다.
3. 두 클라이언트 화면 모두 구체 팩토리 타입을 알지 못한 채 같은 인터페이스로 서로 다른 제품군을 사용할 수 있습니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

테스트는 각 팩토리 구현이 제품군의 일관성을 지키는지 확인합니다. 트레이트 타입만으로는 팩토리가 서로 다른 제품군을 잘못 섞어 반환하는 것까지 자동으로 막지 않으므로, 새 구체 팩토리를 추가할 때도 제품군 조합 테스트를 유지하는 것이 좋습니다.

## Factory Method와의 차이

Factory Method는 보통 한 종류의 제품을 생성하는 메서드를 구체 Creator가 구현하도록 해 생성 방식을 다르게 합니다. Abstract Factory는 여러 종류의 관련 제품을 생성하는 메서드들을 한 팩토리 인터페이스에 묶어, 제품군 전체의 호환성을 유지하는 데 초점을 둡니다. 두 패턴은 함께 사용될 수 있습니다. Abstract Factory의 각 생성 메서드가 내부적으로 Factory Method를 사용할 수도 있습니다.

## 장점과 단점

### 장점

- 클라이언트가 구체 제품 타입에 직접 의존하지 않습니다.
- 한 팩토리가 관련 제품을 함께 생성하므로, 같은 변형에 속한 제품군을 일관되게 선택할 수 있습니다.
- 새 제품 변형을 추가할 때 새 구체 팩토리와 제품을 추가하고 초기화 단계에서 선택하도록 구성할 수 있습니다.
- 테스트에서 다른 팩토리를 주입해 클라이언트 동작을 확인할 수 있습니다.

### 단점

- 제품 종류가 늘어날수록 추상 팩토리 인터페이스와 모든 구체 팩토리 구현을 함께 수정해야 합니다.
- 제품 트레이트, 팩토리 트레이트, 구체 제품과 팩토리가 추가되어 구조가 복잡해집니다.
- `Box<dyn Trait>`을 사용하면 힙 할당과 동적 디스패치 비용이 생깁니다.
- 제품군의 변형이나 조합이 거의 없고 생성 로직도 단순하다면, `match`나 일반 생성 함수가 더 간결할 수 있습니다.

## 참고 자료

- [생성 패턴 개요](/docs/creational-patterns)
- [Abstract Factory 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/abstract-factory)
- [Abstract Factory 패턴 — 위키백과](https://en.wikipedia.org/wiki/Abstract_factory_pattern)
- [Rust의 트레이트 문서: 공통 동작 정의](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust Reference: 트레이트 객체](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Factory Method 패턴](/docs/factory-method): 구체 Creator에 제품 생성을 위임하는 관련 생성 패턴
