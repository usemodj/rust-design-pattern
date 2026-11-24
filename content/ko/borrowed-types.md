# 인자에는 빌린 타입 사용하기

## 설명

함수 인자의 타입을 정할 때 역참조 강제 변환(deref coercion)의 대상 타입을 사용하면 코드의 유연성을 높일 수 있습니다. 이렇게 하면 함수가 더 다양한 입력 타입을 받을 수 있습니다.

이 원칙은 슬라이스로 변환할 수 있는 타입이나 팻 포인터(fat pointer) 타입에만 한정되지 않습니다. 소유 타입 자체를 빌리는 것보다 **그 소유 타입에 대응하는 빌린 타입을 우선해서 사용**하는 편이 좋습니다. 예를 들면 `&String` 대신 `&str`, `&Vec<T>` 대신 `&[T]`, `&Box<T>` 대신 `&T`를 사용합니다.

소유 타입이 이미 간접 참조 계층을 제공한다면, 빌린 타입을 사용해 불필요한 간접 참조 계층을 피할 수 있습니다. 예를 들어 `String`에는 이미 간접 참조 계층이 있으므로 `&String`은 간접 참조를 두 번 거치게 됩니다. 함수가 `&str`을 받도록 하면 이 계층을 하나 줄일 수 있습니다. 함수를 호출할 때 `&String`은 필요한 경우 `&str`로 자동 강제 변환됩니다.

## 예제

다음 예제에서는 함수 인자로 `&String`을 사용하는 경우와 `&str`을 사용하는 경우의 차이를 살펴봅니다. 같은 원리는 `&Vec<T>`와 `&[T]`, `&Box<T>`와 `&T` 사이에도 적용됩니다.

문자열에 모음이 세 번 연속으로 나타나는지 확인한다고 가정해 봅시다. 확인을 위해 문자열의 소유권을 가질 필요는 없으므로 참조를 인자로 받으면 됩니다. 처음에는 다음과 같이 작성할 수 있습니다.

```rust
fn three_vowels(word: &String) -> bool {
    let mut vowel_count = 0;
    for c in word.chars() {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => {
                vowel_count += 1;
                if vowel_count >= 3 {
                    return true;
                }
            }
            _ => vowel_count = 0,
        }
    }
    false
}

fn main() {
    let ferris = "Ferris".to_string();
    let curious = "Curious".to_string();
    println!("{}: {}", ferris, three_vowels(&ferris));
    println!("{}: {}", curious, three_vowels(&curious));

    // This works fine, but the following two lines would fail:
    // println!("Ferris: {}", three_vowels("Ferris"));
    // println!("Curious: {}", three_vowels("Curious"));
}
```

`&String` 타입을 전달하고 있으므로 위 코드는 정상적으로 동작합니다. 하지만 마지막 두 줄의 주석을 해제하면 예제가 컴파일되지 않습니다. `&str` 타입은 `&String` 타입으로 강제 변환되지 않기 때문입니다. 함수의 인자 타입을 바꾸면 이 문제를 해결할 수 있습니다.

```rust, ignore
fn three_vowels(word: &str) -> bool {
```

이렇게 하면 두 형태의 호출이 모두 컴파일되며 같은 결과를 출력합니다.

```bash
Ferris: false
Curious: true
```

하지만 여기서 끝이 아닙니다. `"Ferris"`처럼 `'static` 수명을 갖는 문자열 슬라이스를 입력으로 사용할 일은 없다고 생각할 수도 있습니다. 그런 특수한 경우를 제외하더라도 `&str`을 사용하면 `&String`보다 더 유연한 함수를 만들 수 있습니다.

이번에는 문장 하나를 받아, 그 문장의 단어 중 모음이 세 번 연속으로 나타나는 단어가 있는지 확인한다고 합시다. 앞에서 정의한 함수를 재사용해 문장의 각 단어를 전달하면 됩니다.

```rust
fn three_vowels(word: &str) -> bool {
    let mut vowel_count = 0;
    for c in word.chars() {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => {
                vowel_count += 1;
                if vowel_count >= 3 {
                    return true;
                }
            }
            _ => vowel_count = 0,
        }
    }
    false
}

fn main() {
    let sentence_string =
        "Once upon a time, there was a friendly curious crab named Ferris".to_string();
    for word in sentence_string.split(' ') {
        if three_vowels(word) {
            println!("{word} has three consecutive vowels!");
        }
    }
}
```

함수 인자가 `&str`이면 다음과 같이 출력됩니다.

```bash
curious has three consecutive vowels!
```

반면 함수 인자가 `&String`이면 이 예제는 동작하지 않습니다. `split`이 반환하는 문자열 조각은 `&String`이 아니라 `&str`이기 때문입니다. `&str`을 `&String`으로 바꾸려면 새 문자열 할당이 필요하며, 이는 암시적으로 일어나지 않습니다. 반대로 `String`을 `&str`로 변환하는 일은 비용이 적고 함수 호출 시 자동으로 처리됩니다.

## 참고 자료

- [Rust 언어 레퍼런스: 타입 강제 변환](https://doc.rust-lang.org/reference/type-coercions.html)
- `String`과 `&str`을 함수에서 사용하는 방법에 대한 [Herman J. Radtke III의 2015년 블로그 연재](https://web.archive.org/web/20201112023149/https://hermanradtke.com/2015/05/03/string-vs-str-in-rust-functions.html)
- [`String`과 `&str` 중 무엇을 사용해야 할까요?](https://archive.ph/LBpD0) — Steve Klabnik의 블로그 글
