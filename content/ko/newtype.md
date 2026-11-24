# Newtype 패턴

타입 별칭만으로는 부족하고, 어떤 타입이 다른 타입과 비슷하게 동작하게 만들거나 컴파일 시점에 특정 동작을 강제하고 싶을 때가 있습니다. 예를 들어 보안상의 이유로 `String`의 `Display` 구현이 비밀번호를 그대로 출력하지 않도록 바꾸고 싶을 수 있습니다.

이런 경우 **타입 안전성**과 **캡슐화**를 제공하는 Newtype 패턴을 사용할 수 있습니다.

## 설명

필드가 하나인 튜플 구조체로 타입을 감싸 불투명한 래퍼를 만듭니다. 이렇게 하면 기존 타입의 별칭이 아니라 별개의 새 타입이 됩니다.

```rust
struct UserId(u64);
type UserIdAlias = u64;
```

`UserId`는 내부에 `u64`를 저장하지만 `u64`와 다른 타입입니다. 반면 `UserIdAlias`는 `u64`의 다른 이름일 뿐이므로 두 타입을 구분하지 못합니다. 새 타입의 필드는 별도로 공개하지 않는 한 비공개이므로, 외부 코드는 내부 값을 직접 다루지 않고 공개된 인터페이스만 사용하게 할 수 있습니다.

## 예제: 비밀번호 출력 감추기

다음 예제는 `String`을 `Password`로 감싸고, `Display` 트레이트를 새 타입에 맞게 구현합니다. 원래 문자열은 그대로 출력되지만 `Password` 값은 별표로 표시됩니다.

```rust
use std::fmt::Display;

// Create Newtype Password to override the Display trait for String
struct Password(String);

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "****************")
    }
}

fn main() {
    let unsecured_password: String = "ThisIsMyPassword".to_string();
    let secured_password: Password = Password(unsecured_password.clone());
    println!("unsecured_password: {unsecured_password}");
    println!("secured_password: {secured_password}");
}
```

```shell
unsecured_password: ThisIsMyPassword
secured_password: ****************
```

이 예제에서 Newtype은 `Display`를 통해 출력되는 표현을 바꿉니다. 값을 암호화하거나 메모리에서 보호하는 것은 아니므로, 실제 비밀번호 보안 기능 자체를 제공한다고 오해해서는 안 됩니다.

## 동기

Newtype을 사용하는 주된 이유는 **추상화**입니다. 여러 타입 사이에 구현 세부 사항을 공유하면서도 각 타입이 외부에 제공하는 인터페이스를 정밀하게 제어할 수 있습니다. API에서 구현 타입을 직접 노출하지 않고 Newtype을 사용하면, 내부 구현을 바꾸더라도 공개 인터페이스를 유지해 하위 호환성을 지킬 수 있습니다.

Newtype은 단위를 구분하는 데도 사용할 수 있습니다. 예를 들어 둘 다 내부적으로 `f64`를 사용하더라도 `Miles`와 `Kilometres`를 별개의 타입으로 정의하면, 두 값을 잘못 섞어 쓰는 일을 컴파일 시점에 방지할 수 있습니다. 타입 별칭으로 `type Miles = f64;`처럼 작성하면 이름은 달라 보여도 실제 타입은 같으므로 이런 보호를 얻을 수 없습니다.

## 장점

- **타입 구분:** 래퍼 타입과 감싼 타입은 서로 호환되는 타입이 아닙니다. (`type` 별칭과 다릅니다.) 따라서 Newtype을 사용하는 코드에서 두 타입을 혼동해 전달할 수 없습니다.
- **런타임 비용 없음:** Newtype은 감싼 타입 위에 타입 안전성과 추상화를 제공하는 제로 비용 추상화입니다. 별도의 런타임 래퍼 비용을 발생시키지 않습니다.
- **캡슐화:** 필드가 기본적으로 비공개이므로, 공개 API에서 내부 타입을 감추고 사용자가 내부 값에 직접 접근하지 못하도록 할 수 있습니다.

## 단점

타입 별칭과 비교했을 때의 단점은 Newtype에 대한 특별한 언어 지원이 없다는 점입니다. 감싼 타입에서 제공하고 싶은 메서드마다 전달 메서드(pass-through method)를 작성하고, 래퍼에서도 사용하고 싶은 트레이트마다 별도의 `impl`을 작성해야 하므로 반복 코드가 많아질 수 있습니다.

## 논의

Newtype은 Rust 코드에서 흔히 사용됩니다. 추상화를 제공하거나 단위를 표현하는 것이 대표적인 용도지만, 다음과 같은 목적으로도 활용할 수 있습니다.

- **기능 제한:** 감싼 타입이 제공하는 함수나 트레이트 중 일부만 공개해 사용할 수 있는 기능을 줄입니다.
- **복사 의미를 이동 의미로 변경:** 감싼 타입이 `Copy`를 구현하더라도 래퍼는 `Copy`를 구현하지 않게 하여 값이 암묵적으로 복사되지 않도록 할 수 있습니다.
- **더 구체적인 공개 타입 제공:** 내부 타입을 감추고 구체적인 API 타입을 제공할 수 있습니다. 예를 들어 다음과 같이 정의할 수 있습니다.

```rust,ignore
pub struct Foo(Bar<T1, T2>);
```

여기서 `Bar`는 공개된 제네릭 타입이고 `T1`, `T2`는 내부 구현에 사용하는 타입일 수 있습니다. 모듈 사용자는 `Foo`가 내부적으로 `Bar`를 사용한다는 사실을 알 필요가 없습니다. 특히 감추는 것은 `Bar`라는 타입뿐 아니라 `T1`, `T2`가 무엇이고 `Bar`와 함께 어떻게 사용되는지에 관한 구현 세부 사항입니다.

## 참고 자료

- [The Rust Programming Language: 고급 타입 - 타입 안전성과 추상화를 위한 Newtype 패턴](https://doc.rust-lang.org/book/ch19-04-advanced-types.html?highlight=newtype#using-the-newtype-pattern-for-type-safety-and-abstraction)
- [Haskell의 Newtype](https://wiki.haskell.org/Newtype)
- [타입 별칭](https://doc.rust-lang.org/stable/book/ch19-04-advanced-types.html#creating-type-synonyms-with-type-aliases)
- [derive_more](https://crates.io/crates/derive_more): Newtype에서 여러 기본 트레이트를 파생할 수 있는 크레이트
- [The Newtype Pattern In Rust](https://web.archive.org/web/20230519162111/https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
