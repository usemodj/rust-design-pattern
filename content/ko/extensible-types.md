# `#[non_exhaustive]`와 비공개 필드로 확장성 확보하기

## 설명

라이브러리 작성자가 기존 공개 구조체에 공개 필드를 추가하거나 공개 열거형에 새 변형을 추가하면서도 하위 호환성을 유지하고 싶은 경우가 있습니다. Rust에서는 이를 위해 두 가지 방법을 사용할 수 있습니다.

- 구조체, 열거형 또는 열거형의 개별 변형에 `#[non_exhaustive]`를 붙입니다. 이 속성을 적용할 수 있는 위치와 동작에 대한 자세한 내용은 [Rust 레퍼런스](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)를 참고하세요.
- 구조체에 비공개 필드를 추가해 외부 코드가 구조체를 직접 생성하거나 모든 필드를 열거해 패턴 매칭하지 못하게 합니다.

## 예제

아래 구조체와 열거형은 다른 크레이트에 공개됩니다. 라이브러리 작성자는 나중에 구조체의 필드나 열거형의 변형을 추가할 수 있습니다.

외부 크레이트에서 `#[non_exhaustive]` 구조체를 분해할 때는 `..`를 사용해야 합니다. 열거형도 미래에 추가될 변형을 고려해 와일드카드 분기를 포함해야 합니다. 개별 변형에 `#[non_exhaustive]`가 붙은 경우 그 변형의 필드를 분해할 때도 `..`가 필요합니다.

```rust
mod a {
    // Public struct.
    #[non_exhaustive]
    pub struct S {
        pub foo: i32,
    }

    #[non_exhaustive]
    pub enum AdmitMoreVariants {
        VariantA,
        VariantB,
        #[non_exhaustive]
        VariantC {
            a: String,
        },
    }
}

fn print_matched_variants(s: a::S) {
    // Because S is `#[non_exhaustive]`, it cannot be named here and
    // we must use `..` in the pattern.
    let a::S { foo: _, .. } = s;

    let some_enum = a::AdmitMoreVariants::VariantA;
    match some_enum {
        a::AdmitMoreVariants::VariantA => println!("it's an A"),
        a::AdmitMoreVariants::VariantB => println!("it's a b"),

        // .. required because this variant is non-exhaustive as well
        a::AdmitMoreVariants::VariantC { a, .. } => println!("it's a c"),

        // The wildcard match is required because more variants may be
        // added in the future
        _ => println!("it's a new variant"),
    }
}
```

## 대안: 구조체의 비공개 필드

`#[non_exhaustive]`는 크레이트 경계를 넘어 사용하는 경우에만 효과가 있습니다. 같은 크레이트 안에서 확장성을 확보하려면 비공개 필드를 사용하는 방법을 고려할 수 있습니다.

구조체에 필드를 추가하는 일은 대체로 하위 호환성을 유지하는 변경입니다. 하지만 사용자가 구조체 패턴에서 모든 필드를 직접 나열해 분해하고 있다면 새 필드가 추가될 때 그 코드가 깨집니다. 일부 필드만 지정하고 `..`를 쓰는 패턴이라면 필드를 추가해도 호환성이 유지됩니다. 필드 중 하나 이상을 비공개로 만들면 외부 사용자는 `..` 패턴을 사용해야 하므로, 이후 필드를 추가할 수 있는 여지를 남깁니다.

이 방식의 단점은 실제 데이터로 필요하지 않은 필드를 구조체에 추가해야 할 수 있다는 점입니다. `()` 타입 필드는 런타임 오버헤드가 없으며, 필드 이름에 `_` 접두사를 붙이면 사용되지 않는 필드 경고를 피할 수 있습니다.

```rust
pub struct S {
    pub a: i32,
    // Because `b` is private, you cannot match on `S` without using `..` and `S`
    //  cannot be directly instantiated or matched against
    _b: (),
}
```

## 논의

구조체에 `#[non_exhaustive]`를 적용하면 하위 호환성을 유지하면서 필드를 추가할 수 있습니다. 모든 필드가 공개되어 있어도 외부 사용자는 구조체 리터럴로 값을 직접 만들 수 없습니다. 이는 새 필드를 빠뜨렸을 때 컴파일 오류로 알리는 장점이 될 수 있지만, 사용자가 새 필드를 알아차리지 못한 채 기존 코드를 계속 사용하는 편이 더 나은지도 고려해야 합니다.

`#[non_exhaustive]`는 열거형의 개별 변형에도 적용할 수 있습니다. 이 경우 해당 변형은 `#[non_exhaustive]` 구조체와 같은 방식으로 다뤄야 하며, 외부 사용자는 나중에 추가될 수 있는 필드에 대비해야 합니다.

이 속성은 신중하게 사용하세요. 필드나 변형을 추가할 때 메이저 버전을 올리는 편이 더 나은 경우가 많습니다. 라이브러리와 별개로 변화할 수 있는 외부 자원을 모델링하는 상황처럼, 메이저 버전을 올리지 않고도 변화에 대응해야 할 때 적절할 수 있지만 일반적인 확장 수단으로 무조건 적용할 기능은 아닙니다.

## 단점

`#[non_exhaustive]`는 API 사용을 불편하게 만들 수 있습니다. 특히 알 수 없는 열거형 변형까지 처리하도록 강제할 때 그렇습니다. 메이저 버전을 올리지 않고도 이런 종류의 확장이 실제로 필요한 경우에만 사용해야 합니다.

열거형에 적용하면 외부 사용자는 와일드카드 분기를 작성해야 합니다. 알 수 없는 변형에 대해 합리적으로 취할 동작이 없다면, 거의 발생하지 않을 코드 경로를 만들거나 해당 분기에서 패닉을 일으키게 될 수 있습니다. 이런 경우에는 오히려 컴파일 시점에 오류가 나도록 메이저 버전을 올리는 편이 더 나을 수도 있습니다. `#[non_exhaustive]`는 사용자에게 “그 밖의 경우”를 처리하도록 요구하지만, 그 경우에 적절한 동작이 거의 없는 상황이 많습니다.

## 참고 자료

- [`#[non_exhaustive]` 속성을 열거형과 구조체에 도입한 RFC](https://github.com/rust-lang/rfcs/blob/master/text/2008-non-exhaustive.md)
