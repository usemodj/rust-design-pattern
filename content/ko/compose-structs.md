# 독립적으로 빌리기 위한 구조체 분해

## 설명

큰 구조체의 일부 필드를 서로 독립적으로 빌려 사용하고 싶을 때, 전체 구조체를 한꺼번에 사용하는 코드가 끼어들면 빌림 검사기가 허용하지 않는 경우가 있습니다. 이럴 때 책임과 데이터가 실제로 독립적인 작은 구조체들로 나누고, 상위 구조체가 이들을 다시 조합하도록 설계할 수 있습니다. 각 구성 요소를 필요한 곳에 따로 전달하면 전체 구조체를 빌리는 대신 필요한 부분만 다루도록 인터페이스를 좁힐 수 있습니다.

이 방식은 빌림 제약을 해결할 뿐 아니라, 큰 구조체 안에 함께 들어 있던 작은 기능 단위를 드러내 더 나은 설계로 이어지기도 합니다. 다만 구조체를 나누는 것 자체가 목적은 아닙니다. 분리한 타입이 의미 있는 개념과 책임을 나타내는지 살펴야 합니다.

## 동기와 원래 예제

다음 예제에서 `Database`는 연결 문자열, 제한 시간, 연결 풀 크기를 한 구조체에 보관합니다. `print_database`는 세 필드를 출력하기 위해 `&Database` 전체를 받습니다.

```rust,ignore
struct Database {
    connection_string: String,
    timeout: u32,
    pool_size: u32,
}

fn print_database(database: &Database) {
    println!("Connection string: {}", database.connection_string);
    println!("Timeout: {}", database.timeout);
    println!("Pool size: {}", database.pool_size);
}

fn main() {
    let mut db = Database {
        connection_string: "initial string".to_string(),
        timeout: 30,
        pool_size: 100,
    };

    let connection_string = &mut db.connection_string;
    print_database(&db);
    *connection_string = "new string".to_string();
}
```

`connection_string`에 대한 가변 참조가 살아 있는 동안 `print_database(&db)`가 구조체 전체를 불변 참조로 빌리려 합니다. 이후 가변 참조를 다시 사용하므로 두 빌림의 사용 구간이 겹칩니다. Rust는 같은 데이터에 대한 가변 참조와 불변 참조가 동시에 유효한 상태를 허용하지 않기 때문에 이 코드는 컴파일되지 않습니다.

원문 컴파일러 진단의 요지는 다음과 같습니다.

```ignore
let connection_string = &mut db.connection_string;
                        ------------------------- mutable borrow occurs here
print_database(&db);
               ^^^ immutable borrow occurs here
*connection_string = "new string".to_string();
------------------ mutable borrow later used here
```

여기서 문제는 구조체 필드가 본질적으로 따로 빌릴 수 없다는 뜻이 아닙니다. Rust는 서로 다른 필드에 대한 참조를 구별할 수 있습니다. 문제는 `print_database`의 매개변수가 `&Database`이므로 호출 시점에 전체 `db`가 빌린다는 점입니다.

## 작은 타입으로 분해하고 다시 조합하기

원문은 각 설정값을 별도의 새 타입으로 감싼 다음, 이 타입들을 다시 `Database`의 필드로 조합합니다.

```rust
// Database is now composed of three structs - ConnectionString, Timeout and PoolSize.
// Let's decompose it into smaller structs
#[derive(Debug, Clone)]
struct ConnectionString(String);

#[derive(Debug, Clone, Copy)]
struct Timeout(u32);

#[derive(Debug, Clone, Copy)]
struct PoolSize(u32);

// We then compose these smaller structs back into `Database`
struct Database {
    connection_string: ConnectionString,
    timeout: Timeout,
    pool_size: PoolSize,
}

// print_database can then take ConnectionString, Timeout and Poolsize struct instead
fn print_database(connection_str: ConnectionString, timeout: Timeout, pool_size: PoolSize) {
    println!("Connection string: {connection_str:?}");
    println!("Timeout: {timeout:?}");
    println!("Pool size: {pool_size:?}");
}

fn main() {
    // Initialize the Database with the three structs
    let mut db = Database {
        connection_string: ConnectionString("localhost".to_string()),
        timeout: Timeout(30),
        pool_size: PoolSize(100),
    };

    let connection_string = &mut db.connection_string;
    print_database(connection_string.clone(), db.timeout, db.pool_size);
    *connection_string = ConnectionString("new string".to_string());
}
```

이제 `print_database`는 `&Database` 대신 필요한 세 값을 개별 매개변수로 받습니다. `timeout`과 `pool_size`는 `Copy` 타입이므로 값으로 전달해도 원래 필드를 계속 사용할 수 있고, `ConnectionString`은 `Clone`을 구현하므로 예제에서는 가변 참조로 사용 중인 필드의 복제본을 전달합니다. 그 뒤 원래 연결 문자열을 새 값으로 바꿉니다.

이 예제는 작은 도메인 타입으로 데이터를 나누는 구조를 보여 주기 위한 것입니다. `String`을 복제하는 비용과 의미를 고려해야 하며, 단순히 빌림 검사기 오류를 피하려고 무조건 `Clone`을 추가하라는 뜻은 아닙니다. 함수가 소유권을 가져갈 필요가 없다면 `&ConnectionString`처럼 빌린 참조를 매개변수로 받는 편이 더 적절할 수 있습니다. 문제를 해결하려고 무분별하게 복제하는 일은 [빌림 검사기를 만족시키기 위한 Clone](/docs/clone-borrow-checker) 안티 패턴도 참고하세요.

## 적용할 때 고려할 점

이 기법은 구조체 필드를 여러 곳에서 독립적으로 사용해야 하거나, 필드 그룹이 별도의 책임·수명·불변 조건을 가진 경우에 유용합니다. 예를 들어 데이터베이스 설정을 연결 정보와 풀 정책으로 묶으면 API가 각 역할을 명시할 수 있습니다.

하지만 모든 빌림 충돌에 새 구조체를 도입해야 하는 것은 아닙니다. 먼저 함수가 정말 구조체 전체를 받아야 하는지 확인하고, 필요한 필드만 매개변수로 받도록 좁히거나, 참조가 살아 있는 범위를 조정할 수 있는지 살펴보세요. 구조체의 서로 다른 필드는 따로 빌릴 수 있으므로, 작은 함수가 필요한 필드만 빌리도록 표현하면 타입을 추가하지 않고도 충분한 경우가 있습니다.

## 장점

- 서로 독립적인 필드를 나누면 빌림 검사기의 제약을 더 세밀한 인터페이스로 해결할 수 있습니다.
- 큰 구조체 안의 기능 단위와 책임을 발견해 더 응집도 높은 설계를 만드는 데 도움이 됩니다.
- 각 구성 요소를 별도로 전달하고 재사용하기 쉬워집니다.

## 단점

- 타입과 코드가 늘어나 더 장황해질 수 있습니다.
- 분리된 작은 구조체가 실제 도메인 개념이나 독립적인 책임을 나타내지 않는다면 좋은 추상화가 아닙니다. 이는 구조를 다시 검토하라는 신호일 수 있습니다.
- 값으로 전달하려고 `Clone`을 추가하면 데이터 복사 비용이 생기고, 원본과 복제본이 서로 다른 값이 될 수 있습니다.

## 논의

이 패턴은 빌림 검사기가 있는 Rust에서 특히 두드러집니다. 다른 언어에서도 기능을 더 작은 단위로 나누면 코드가 명료해질 수 있지만, Rust에서는 소유권과 빌림 규칙 때문에 데이터 구조와 함수 경계가 빌림 가능성에 직접 영향을 줍니다.

Rust의 빌림 검사기는 `a.b`와 `a.c`처럼 서로 다른 필드가 구별된다는 점을 알고 있으므로, 독립적인 필드를 따로 빌릴 수 있습니다. 반면 `&a`를 전달하면 구조체 전체를 빌립니다. 따라서 이 패턴의 핵심은 단순히 필드마다 래퍼 타입을 만드는 것이 아니라, 필요한 데이터만 다루도록 구조와 API의 경계를 설계하는 데 있습니다.

## 참고 자료

- [Rust Design Patterns 원문: Struct decomposition for independent borrowing](https://rust-unofficial.github.io/patterns/patterns/structural/compose-structs.html)
- [The Rust Programming Language: 참조와 빌림](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [The Rust Reference: 필드 접근 표현식](https://doc.rust-lang.org/reference/expressions/field-expr.html)
- [Rust 컴파일러 오류 E0502: 서로 다른 가변성으로 다시 빌리기](https://doc.rust-lang.org/error_codes/E0502.html)
- [빌림 검사기를 만족시키기 위한 Clone 안티 패턴](/docs/clone-borrow-checker)
- [`mem::take`와 `mem::replace` 관용구](/docs/mem-take-replace)
