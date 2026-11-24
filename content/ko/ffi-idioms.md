# FFI 관용구

FFI(Foreign Function Interface)는 Rust와 다른 언어의 함수 호출 규약 및 메모리 표현을 연결합니다. `unsafe` 코드와 명시적인 ABI 계약이 필요하므로 별도의 주의가 요구됩니다.

이 장에서는 다음 주제를 다룹니다.

- [FFI에서 오류 처리하기](/docs/ffi-errors): 정수 오류 코드와 센티널 반환값으로 오류 전달
- [FFI에서 문자열 받기](/docs/ffi-accept-strings): C 문자열을 가능한 한 빌린 값으로 다루기
- [FFI로 문자열 전달하기](/docs/ffi-pass-strings): 수명과 소유권을 지키며 문자열 넘기기

외부 포인터의 유효성, 정렬, 초기화 여부, 널 종료, 호출 중 변경 가능성은 호출자와 API 사이의 계약으로 명시해야 합니다. `unsafe` 블록을 작게 유지하고, 모든 `unsafe` 연산의 전제 조건을 문서화하세요.
