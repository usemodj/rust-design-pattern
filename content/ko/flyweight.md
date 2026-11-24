# Flyweight 패턴

## 의도

Flyweight(플라이웨이트) 패턴은 여러 객체가 공통으로 사용하는 데이터를 공유해 메모리 사용량을 줄입니다. 객체의 상태를 다음 두 부분으로 나누는 것이 핵심입니다.

- **내재 상태(intrinsic state):** 객체들 사이에서 공유할 수 있는 변하지 않는 데이터입니다.
- **외재 상태(extrinsic state):** 각 객체마다 달라 객체 외부나 개별 인스턴스에 보관해야 하는 데이터입니다.

예를 들어 숲의 수많은 나무는 종, 색, 질감은 같을 수 있지만 각 나무의 위치는 다릅니다. 나무 종류 정보를 한 번 만들어 공유하고 위치만 개별 나무에 보관하면 중복된 종류 데이터를 줄일 수 있습니다.

## 구성 요소

- **플라이웨이트(Flyweight):** 여러 객체가 공유하는 내재 상태를 보관합니다.
- **플라이웨이트 팩토리(Flyweight Factory):** 같은 내재 상태의 플라이웨이트가 중복 생성되지 않도록 찾아 재사용하거나 새로 만듭니다.
- **컨텍스트(Context):** 공유 플라이웨이트와 개별 외재 상태를 결합해 하나의 논리적 객체를 나타냅니다.
- **클라이언트(Client):** 각 인스턴스에 고유한 외재 상태를 관리하고 공유 플라이웨이트를 사용합니다.

## Rust에서 표현하기

Rust에서는 공유할 불변 데이터를 `Rc<T>`로 여러 인스턴스가 참조하도록 구성할 수 있습니다. 스레드 사이에서 공유해야 한다면 `Arc<T>`를 사용할 수 있습니다. 해시 맵을 이용한 팩토리는 내재 상태의 값이 기존 항목과 같을 때 기존 포인터를 반환합니다.

이 구조에서 플라이웨이트는 공유되므로 일반적으로 불변으로 다루는 편이 안전합니다. 개별 객체의 위치처럼 달라지는 데이터는 플라이웨이트 안에 넣지 않고 각 컨텍스트에 저장해야 합니다.

## Rust 예제: 나무 종류를 공유하는 숲

`TreeType`은 여러 나무가 공유하는 종, 색, 질감 정보를 보유합니다. `Tree`는 개별 좌표를 소유하면서 `Rc<TreeType>`을 공유합니다. `TreeFactory`는 같은 종류의 나무를 요청하면 기존 플라이웨이트를 재사용합니다.

`src/lib.rs`:

```rust
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TreeType {
    species: String,
    color: String,
    texture: String,
}

impl TreeType {
    pub fn species(&self) -> &str {
        &self.species
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn texture(&self) -> &str {
        &self.texture
    }
}

#[derive(Default)]
pub struct TreeFactory {
    types: HashMap<TreeType, Rc<TreeType>>,
}

impl TreeFactory {
    pub fn get_or_create(
        &mut self,
        species: impl Into<String>,
        color: impl Into<String>,
        texture: impl Into<String>,
    ) -> Rc<TreeType> {
        let key = TreeType {
            species: species.into(),
            color: color.into(),
            texture: texture.into(),
        };

        if let Some(existing) = self.types.get(&key) {
            return Rc::clone(existing);
        }

        let tree_type = Rc::new(key.clone());
        self.types.insert(key, Rc::clone(&tree_type));
        tree_type
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }
}

struct Tree {
    x: i32,
    y: i32,
    tree_type: Rc<TreeType>,
}

impl Tree {
    fn describe(&self) -> String {
        format!(
            "{} at ({}, {}), color={}, texture={}",
            self.tree_type.species(),
            self.x,
            self.y,
            self.tree_type.color(),
            self.tree_type.texture()
        )
    }
}

#[derive(Default)]
pub struct Forest {
    trees: Vec<Tree>,
    tree_factory: TreeFactory,
}

impl Forest {
    pub fn plant_tree(
        &mut self,
        x: i32,
        y: i32,
        species: impl Into<String>,
        color: impl Into<String>,
        texture: impl Into<String>,
    ) {
        let tree_type = self.tree_factory.get_or_create(species, color, texture);
        self.trees.push(Tree { x, y, tree_type });
    }

    pub fn tree_count(&self) -> usize {
        self.trees.len()
    }

    pub fn unique_type_count(&self) -> usize {
        self.tree_factory.type_count()
    }

    pub fn describe_trees(&self) -> Vec<String> {
        self.trees.iter().map(Tree::describe).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Forest;
    use std::rc::Rc;

    #[test]
    fn trees_with_the_same_intrinsic_state_share_one_flyweight() {
        let mut forest = Forest::default();
        forest.plant_tree(10, 20, "oak", "green", "rough");
        forest.plant_tree(30, 40, "oak", "green", "rough");

        assert_eq!(forest.tree_count(), 2);
        assert_eq!(forest.unique_type_count(), 1);
        assert!(Rc::ptr_eq(
            &forest.trees[0].tree_type,
            &forest.trees[1].tree_type
        ));
    }

    #[test]
    fn different_intrinsic_states_use_different_flyweights() {
        let mut forest = Forest::default();
        forest.plant_tree(0, 0, "oak", "green", "rough");
        forest.plant_tree(5, 5, "pine", "dark green", "needle-like");

        assert_eq!(forest.unique_type_count(), 2);
        assert!(!Rc::ptr_eq(
            &forest.trees[0].tree_type,
            &forest.trees[1].tree_type
        ));
    }

    #[test]
    fn each_tree_keeps_its_own_extrinsic_position() {
        let mut forest = Forest::default();
        forest.plant_tree(10, 20, "oak", "green", "rough");
        forest.plant_tree(30, 40, "oak", "green", "rough");

        assert_eq!(
            forest.describe_trees(),
            vec![
                "oak at (10, 20), color=green, texture=rough",
                "oak at (30, 40), color=green, texture=rough",
            ]
        );
    }
}
```

두 `oak` 나무는 같은 `Rc<TreeType>`을 참조하지만 좌표는 각각의 `Tree`에 저장됩니다. 테스트는 포인터가 실제로 공유되는지, 서로 다른 내재 상태는 별도 플라이웨이트를 사용하는지, 외재 상태인 위치는 나무마다 유지되는지 확인합니다.

## 테스트 코드

위 `src/lib.rs`의 단위 테스트는 다음 동작을 검증합니다.

1. 내재 상태가 같은 나무가 하나의 플라이웨이트를 공유합니다.
2. 내재 상태가 다른 나무는 서로 다른 플라이웨이트를 사용합니다.
3. 종류 정보를 공유해도 각 나무의 좌표는 독립적으로 유지됩니다.

프로젝트 루트에서 다음 명령으로 테스트를 실행합니다.

```console
cargo test
```

## 장점과 단점

### 장점

- 많은 객체가 큰 공통 데이터를 반복해서 보유하는 경우 메모리 사용량을 줄일 수 있습니다.
- 공유 데이터와 인스턴스별 데이터를 분리해 어떤 상태가 공통인지 드러냅니다.
- 팩토리로 공유 객체 생성을 모아 중복 생성을 통제할 수 있습니다.

### 단점

- 팩토리와 해시 맵, 참조 카운트가 추가되어 작은 객체나 적은 인스턴스에서는 오히려 비용이 커질 수 있습니다.
- 외재 상태와 내재 상태를 잘못 나누면 객체 사용이 복잡해집니다.
- 이 예제의 `Rc<T>`는 단일 스레드에서만 사용할 수 있습니다. 다중 스레드에서 공유해야 한다면 `Arc<T>`를 고려해야 합니다.
- 팩토리가 강한 참조를 계속 보유하면 한 번 등록된 모든 종류가 살아남습니다. 종류가 무한히 늘 수 있는 시스템에서는 캐시의 수명과 정리 정책도 설계해야 합니다.

Flyweight는 실제로 중복 상태가 많고 메모리 사용량이 문제가 될 때 적용하는 것이 좋습니다. 공유 포인터와 해시 조회 비용이 절감되는 메모리보다 커질 수 있으므로, 필요성을 측정하고 적용하세요.

## 참고 자료

- [구조 패턴 개요](/docs/structural-patterns)
- [Flyweight 패턴 — Refactoring Guru](https://refactoring.guru/design-patterns/flyweight)
- [Flyweight 패턴 — 위키백과](https://en.wikipedia.org/wiki/Flyweight_pattern)
- [`std::rc::Rc` 문서](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [`std::sync::Arc` 문서](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [`std::collections::HashMap` 문서](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [Composite 패턴](/docs/composite): 객체를 트리로 구성해 같은 인터페이스로 다루는 패턴
