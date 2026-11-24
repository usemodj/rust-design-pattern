# `Option` 반복하기

## 설명

`Option`은 항목이 0개이거나 1개인 컨테이너로 볼 수 있습니다. 특히 `IntoIterator` 트레이트를 구현하므로, 이런 형태의 입력을 처리해야 하는 제네릭 코드에서 반복자로 사용할 수 있습니다.

## 예제

### `extend`로 항목 추가하기

`Option`은 `IntoIterator`를 구현하므로 [`extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html#tymethod.extend)의 인자로 전달할 수 있습니다. `Some(value)`는 그 값을 하나의 항목으로 추가하고, `None`은 추가할 항목이 없는 반복자처럼 동작합니다. 아래 코드는 `turing`의 값을 벡터 끝에 추가합니다.

```rust
let turing = Some("Turing");
let mut logicians = vec!["Curry", "Kleene", "Markov"];

logicians.extend(turing);

// equivalent to
if let Some(turing_inner) = turing {
    logicians.push(turing_inner);
}
```

`extend`를 사용하는 코드는 `Some`인지 확인한 뒤 `push`하는 코드와 같은 결과를 냅니다. 이미 반복자 기반으로 여러 값을 처리하는 흐름 안에 선택적인 항목을 추가할 때 간결하게 표현할 수 있습니다.

### `chain`으로 반복자 연결하기

기존 반복자의 끝에 `Option`을 덧붙이고 싶다면 [`chain`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain)에 전달할 수 있습니다. 다음 예제는 논리학자 목록을 순회한 뒤 `Some("Turing")`의 항목도 이어서 출력합니다. `turing.iter()`는 `Some`이면 한 항목을, `None`이면 항목이 없는 반복자를 만듭니다.

```rust
let turing = Some("Turing");
let logicians = vec!["Curry", "Kleene", "Markov"];

for logician in logicians.iter().chain(turing.iter()) {
    println!("{logician} is a logician");
}
```

`Option`이 항상 `Some`이라면 `Some(value).into_iter()`를 사용하는 것보다 값에 [`std::iter::once`](https://doc.rust-lang.org/std/iter/fn.once.html)를 적용하는 편이 더 관용적입니다. `once`는 정확히 하나의 항목을 내놓는다는 의도를 분명하게 나타냅니다.

`Option`은 `IntoIterator`를 구현하므로 `for` 루프에서 직접 순회할 수도 있습니다. 이는 `if let Some(..)`으로 값을 확인하는 것과 같은 방식으로 동작합니다. 대부분의 경우에는 반복자 하나를 순회하기 위해 `for`를 쓰기보다 `if let Some(value) = option`을 사용하는 편이 더 읽기 쉽습니다.

## 참고 자료

- [`std::iter::once`](https://doc.rust-lang.org/std/iter/fn.once.html): 정확히 하나의 항목을 반환하는 반복자이며, `Some(foo).into_iter()`보다 의도가 분명합니다.
- [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map): `Option`을 반환하는 변환 함수에 특화된 `Iterator::map`입니다.
- [`ref_slice` 크레이트](https://crates.io/crates/ref_slice): `Option`을 0개 또는 1개 항목을 담은 슬라이스로 변환하는 함수를 제공합니다.
- [`Option<T>` 문서](https://doc.rust-lang.org/std/option/enum.Option.html)
